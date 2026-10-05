# Data Model — Keys, Values, Maps

> Rationale: [design.md](design.md). Architecture: [architecture.md](architecture.md).
> Decisions: [decisions.md](decisions.md).
> 
> *Note: Since we use Rust for both kernel (`bpf`) and user space, all structures are defined in `common/src/lib.rs` and marked with `#[repr(C)]` for stable memory layout.*

## `FlowKey` (map key)

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FlowKey {
  pub src_ip: u32,
  pub dst_ip: u32,
  pub src_port: u16,
  pub dst_port: u16,
  pub protocol: u8,   // IPPROTO_TCP
  pub pid: u32,       // owning process
}
```

- Size: 17 bytes (packed), 18–20 with alignment.
- Cache-line friendly.
- PID is in the key for per-process isolation.

## `FlowStats` (map value)

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FlowStats {
  pub bytes_sent: u64,
  pub bytes_recv: u64,
  pub packets_sent: u64,
  pub packets_recv: u64,
  pub start_ts: u64,
  pub last_update_ts: u64,
  pub tcp_state: u32,
  pub anomaly_flags: u32,   // bitfield
}
```

- Size: ~56 bytes.
- Fits comfortably on eBPF stack (limit ~512 bytes).

## Anomaly Flags

```rust
pub const ANOMALY_HIGH_LOSS: u32 = 0x1;
pub const ANOMALY_STUCK_CONN: u32 = 0x2;
pub const ANOMALY_DATA_SPIKE: u32 = 0x4;
```

## `FiveTuple` (pid binding key)

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FiveTuple {
  pub src_ip: u32,
  pub dst_ip: u32,
  pub src_port: u16,
  pub dst_port: u16,
  pub protocol: u8,
}
```

Value: `u32 pid`.

## `AnomalyEvent` (ring buffer)

```rust
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
```

## Maps

| Map | Type | Key | Value | Size |
|-----|------|-----|-------|------|
| `FLOW_STATS` | `LruHashMap` | `FlowKey` | `FlowStats` | 100K |
| `PID_BINDINGS` | `HashMap` | `FiveTuple` | `u32` | 10K |
| `ANOMALY_EVENTS` | `RingBuf` | — | `AnomalyEvent` | 256 KB |


## Cross-References
- [design.md](design.md)
- [architecture.md](architecture.md)
- [decisions.md](decisions.md)
- [verifier-notes.md](verifier-notes.md)
