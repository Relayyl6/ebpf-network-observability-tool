# Decisions — Locked-In Trade-offs

> Rationale: [design.md](design.md). Architecture: [architecture.md](architecture.md).

## Anomaly Detection
**Hybrid** — flags in kernel, thresholds in userspace.
See [design.md#2](design.md#2-anomaly-detection-strategy).

## Hook Strategy
**TC + kprobe** — TC for traffic, kprobe for PID binding.
See [design.md#3](design.md#3-hook-strategy).

## Ring vs Perf
**Ring buffer** — simpler userspace, per-map ordering, kernel ≥ 5.8.

## Sync vs Async
**Tokio Async via Aya** — Since `aya-rs` is highly optimized for asynchronous `tokio`, we embrace it from the start for ring buffer consumption.

## Flow Lifetime
**Outlive process** — Flows are stored in an `LruHashMap`. They outlive the process to ensure we capture all data sent by short-lived exfiltration scripts (like `curl`). The kernel automatically evicts the oldest flows when the map reaches its limit (100,000 entries).

## PID Binding GC
**Deferred (Future LruHashMap)** — The `tracepoint_sched_process_exit` hook can detect process death, but iterating over large maps in userspace is expensive. For now, GC is deferred. A future optimization will simply change `PID_BINDINGS` to an `LruHashMap` to leverage kernel-native auto-eviction.
## Map Types
- `FLOW_STATS`: **LRU_HASH** (auto-evict)
- `PID_BINDINGS`: **HASH** (manual GC)

## Cross-References
- [design.md](design.md)
- [architecture.md](architecture.md)
- [data-model.md](data-model.md)
- [verifier-notes.md](verifier-notes.md)
