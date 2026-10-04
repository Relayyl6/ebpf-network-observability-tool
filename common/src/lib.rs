#![no_std]

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FlowKey {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,
    pub pid: u32,
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for FlowKey {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FlowStats {
    pub bytes_sent: u64,
    pub bytes_recv: u64,
    pub packets_sent: u64,
    pub packets_recv: u64,
    pub start_ts: u64,
    pub tcp_state: u32,
    pub anomaly_flags: u32,
    pub last_update_ts: u64,
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for FlowStats {}

pub const ANOMALY_HIGH_LOSS: u32 = 0x1;
pub const ANOMALY_STUCK_CONN: u32 = 0x2;
pub const ANOMALY_DATA_SPIKE: u32 = 0x4;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FiveTuple {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for FiveTuple {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AnomalyEvent {
    pub ts: u64,
    pub pid: u32,
    pub uid: u32,
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub anomaly_flags: u32,
    pub bytes: u64,
    pub packets: u64,
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for AnomalyEvent {}
