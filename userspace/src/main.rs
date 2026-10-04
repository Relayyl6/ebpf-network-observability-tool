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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    env_logger::init();
    
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
                ringbuf::drain(&mut anomaly_events_map, &mut engine);

                // 2. Snapshot
                flow_map::snapshot(&flow_stats);
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Exiting...");
                break;
            }
        }
    }

    Ok(())
}
