pub mod loader;
pub mod ringbuf;
pub mod flow_map;
pub mod anomaly;
pub mod output;

use aya::Ebpf;
use clap::Parser;
use log::info;
use std::time::Duration;
use aya::maps::{HashMap, RingBuf, MapData};
use common::{FlowKey, FlowStats};

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
}

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::sync::broadcast;
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::json;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    env_logger::init();
    
    // Broadcast channel for WebSocket
    let (tx, _rx) = broadcast::channel(100);
    let app_state = AppState { tx: tx.clone() };

    // Start WebSocket Server
    tokio::spawn(async move {
        let app = Router::new()
            .route("/ws", get(ws_handler))
            .with_state(app_state);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
        log::info!("WebSocket Server listening on ws://127.0.0.1:3030/ws");
        axum::serve(listener, app).await.unwrap();
    });

    // Bump RLIMIT_MEMLOCK to allow BPF map creation on older WSL kernels
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        info!("Failed to set rlimit, but continuing anyway.");
    }
    
    // Load eBPF object file (Note: BPF target directory is inside the bpf/ package folder)
    #[cfg(debug_assertions)]
    let mut bpf = Ebpf::load(aya::include_bytes_aligned!("../../bpf/target/bpfel-unknown-none/debug/bpf"))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Ebpf::load(aya::include_bytes_aligned!("../../bpf/target/bpfel-unknown-none/release/bpf"))?;

    info!("Loaded eBPF program");

    // Phase 5: Delegated Loader
    loader::load_and_attach(&mut bpf, &opt.iface)?;

    // Initialize eBPF logger to receive logs from kernel (if the map exists)
    if let Err(e) = aya_log::EbpfLogger::init(&mut bpf) {
        log::debug!("EbpfLogger not initialized (probably because no logs are emitted in kernel): {}", e);
    }

    // Get maps by taking ownership to satisfy the borrow checker
    let flow_stats: HashMap<MapData, FlowKey, FlowStats> = HashMap::try_from(bpf.take_map("FLOW_STATS").unwrap())?;
    let mut anomaly_events_map: RingBuf<MapData> = RingBuf::try_from(bpf.take_map("ANOMALY_EVENTS").unwrap())?;
    
    let mut engine = anomaly::AnomalyEngine::new();
    let mut interval = tokio::time::interval(Duration::from_millis(500));

    info!("Starting snapshot & ringbuffer event loop...");
    loop {
        tokio::select! {
            _ = interval.tick() => {
                // 1. Drain ring buffer
                ringbuf::drain(&mut anomaly_events_map, &mut engine, &tx);

                // 2. Snapshot
                flow_map::snapshot(&flow_stats, &tx);
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Exiting...");
                break;
            }
        }
    }

    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    // Stream messages from broadcast channel to the websocket
    while let Ok(msg) = rx.recv().await {
        if socket.send(Message::Text(msg)).await.is_err() {
            break; // Client disconnected
        }
    }
}
