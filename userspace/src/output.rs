use common::AnomalyEvent;
use serde::Serialize;
use std::net::Ipv4Addr;

#[derive(Serialize)]
pub struct JsonAnomalyEvent {
    pub ts: u64,
    pub pid: u32,
    pub src_ip: String,
    pub dst_ip: String,
    pub src_port: u16,
    pub dst_port: u16,
    pub anomaly_flags: u32,
    pub bytes: u64,
    pub packets: u64,
}

impl From<&AnomalyEvent> for JsonAnomalyEvent {
    fn from(event: &AnomalyEvent) -> Self {
        Self {
            ts: event.ts,
            pid: event.pid,
            src_ip: Ipv4Addr::from(event.src_ip).to_string(),
            dst_ip: Ipv4Addr::from(event.dst_ip).to_string(),
            src_port: event.src_port,
            dst_port: event.dst_port,
            anomaly_flags: event.anomaly_flags,
            bytes: event.bytes,
            packets: event.packets,
        }
    }
}

pub fn emit_anomaly(event: &AnomalyEvent) {
    let json_event = JsonAnomalyEvent::from(event);
    if let Ok(json_str) = serde_json::to_string(&json_event) {
        println!("{}", json_str);
    }
}
