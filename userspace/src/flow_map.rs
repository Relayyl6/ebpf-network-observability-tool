use aya::maps::{HashMap, MapData};
use common::{FlowKey, FlowStats};
use log::info;

pub fn snapshot(flow_stats: &HashMap<MapData, FlowKey, FlowStats>) {
    let mut active_flows = 0;
    let mut total_bytes = 0;
    
    for item in flow_stats.iter() {
        if let Ok((_key, stats)) = item {
            active_flows += 1;
            total_bytes += stats.bytes_sent + stats.bytes_recv;
        }
    }
    info!("Snapshot: {} active TCP flows ({} total bytes)", active_flows, total_bytes);
}
