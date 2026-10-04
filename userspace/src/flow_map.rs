use aya::maps::{HashMap, MapData};
use common::{FlowKey, FlowStats};
use log::info;
use tokio::sync::broadcast;
use serde_json::json;

pub fn snapshot(flow_stats: &HashMap<MapData, FlowKey, FlowStats>, tx: &broadcast::Sender<String>) {
    let mut active_flows = 0;
    let mut total_bytes = 0;
    
    for item in flow_stats.iter() {
        if let Ok((_key, stats)) = item {
            active_flows += 1;
            total_bytes += stats.bytes_sent + stats.bytes_recv;
        }
    }
    
    // Broadcast to websocket
    let _ = tx.send(json!({
        "type": "snapshot",
        "data": {
            "active_flows": active_flows,
            "total_bytes": total_bytes
        }
    }).to_string());

    info!("Snapshot: {} active TCP flows ({} total bytes)", active_flows, total_bytes);
}
