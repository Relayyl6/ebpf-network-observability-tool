# Glossary

| Term | Meaning |
|------|---------|
| **TC** | Traffic Control — qdisc hook, sees sk_buff |
| **XDP** | eXpress Data Path — driver-level RX hook |
| **kprobe** | Kernel probe on function entry |
| **tracepoint** | Stable kernel trace hook |
| **sk_buff** | Kernel socket buffer struct |
| **CO-RE** | Compile Once, Run Everywhere |
| **LRU_HASH** | BPF map with auto-eviction |
| **RINGBUF** | Single-writer kernel→user buffer |
| **Verifier** | Kernel safety prover for BPF |
| **aya-rs** | Rust library for eBPF, loading logic and maps |
| **flow_key** | 5-tuple + PID (see [data-model.md](data-model.md)) |
| **flow_stats** | Per-flow metrics (see [data-model.md](data-model.md)) |
| **anomaly_flags** | Bitfield: HIGH_LOSS, STUCK_CONN, DATA_SPIKE |

## Cross-References
- [design.md](design.md)
- [data-model.md](data-model.md)
- [verifier-notes.md](verifier-notes.md)
