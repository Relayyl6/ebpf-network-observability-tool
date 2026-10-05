# Architecture — Component Diagram & Data Flow

> Rationale: [design.md](design.md). Data schemas: [data-model.md](data-model.md).
> Decisions: [decisions.md](decisions.md).

## Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         KERNEL SPACE                        │
│                                                             │
│  ┌──────────────┐   ┌──────────────┐   ┌────────────────┐   │
│  │ TC ingress   │   │ TC egress    │   │ kprobe         │   │
│  │ (count rx)   │   │ (count tx)   │   │ tcp_v4_connect │   │
│  └──────┬───────┘   └──────┬───────┘   └───────┬────────┘   │
│         │                  │                   │            │
│         ▼                  ▼                   ▼            │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              BPF MAPS                               │    │
│  │  FLOW_STATS (LRU_HASH)       PID_BINDINGS (HASH)    │    │
│  └─────────────────────────────────────────────────────┘    │
│         │                                                   │
│         ▼                                                   │
│  ┌──────────────┐   ┌──────────────────────────────────┐    │
│  │ ringbuf      │◄──│ anomaly flags set in FlowStats   │    │
│  │ (events)     │   └──────────────────────────────────┘    │
│  └──────┬───────┘                                           │
│         │                                                   │
│  ┌──────▼───────┐                                           │
│  │ tracepoint   │  sched_process_exit → orphan flows        │
│  └──────────────┘                                           │
└─────────┬───────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────┐
│                        USERSPACE (Rust)                     │
│                                                             │
│  ┌──────────────┐   ┌──────────────┐   ┌────────────────┐   │
│  │ main.rs      │   │ ringbuf.rs   │   │ flow_map.rs    │   │
│  │ (aya load)   │   │ (drain)      │   │ (snapshot)     │   │
│  └──────┬───────┘   └──────┬───────┘   └───────┬────────┘   │
│         │                  │                   │            │
│         ▼                  ▼                   ▼            │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              anomaly.rs (thresholds)                │    │
│  └──────────────────────┬──────────────────────────────┘    │
│                         ▼                                   │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              output.rs (stdout / JSON / WebSocket)  │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Data Flow

1. **Connection setup** — kprobe on `tcp_v4_connect` writes `(5-tuple → pid)`
   into `PID_BINDINGS`. See [data-model.md](data-model.md#five_tuple).
2. **Packet rx** — TC ingress reads `PID_BINDINGS`, updates
   `FLOW_STATS[FlowKey].bytes_recv / packets_recv`.
3. **Packet tx** — TC egress updates `bytes_sent / packets_sent`.
4. **Anomaly check** — inside TC hooks, binary flags are set on `FlowStats`.
5. **Event emission** — when a flag transitions 0→1, push `AnomalyEvent`
   onto ring buffer.
6. **Userspace drain** — `ringbuf.rs` reads events, `anomaly.rs` applies
   thresholds, `output.rs` emits.
7. **Flow map snapshot** — every 100ms, `flow_map.rs` iterates
   `FLOW_STATS` for aggregate views.
8. **Process exit** — tracepoint on `sched_process_exit` marks flows
   orphaned or deletes them.

## Userspace Modules (in `userspace/src/`)

| Module | Responsibility |
|--------|----------------|
| `main.rs` | CLI, wiring, aya program load, event loop, Axum WebSocket server |
| `ringbuf.rs` | Ring buffer drain (non-blocking) |
| `flow_map.rs` | Periodic snapshot of `FLOW_STATS` |
| `anomaly.rs` | Thresholds, correlation, scoring |
| `output.rs` | stdout / JSON |

## Dependencies

- `aya` — BPF program loading
- `aya-log` — logging from BPF
- `clap` — CLI
- `serde` + `serde_json` — output
- `tokio` — standard async executor for aya userspace
- `axum` — WebSocket Server for UI Dashboard

## Cross-References
- [design.md](design.md)
- [data-model.md](data-model.md)
- [decisions.md](decisions.md)
- [verifier-notes.md](verifier-notes.md)
- [runbook.md](runbook.md)
