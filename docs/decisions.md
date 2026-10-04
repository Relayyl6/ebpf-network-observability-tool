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
**Open** — should flows outlive their process? Track in
[roadmap.md](roadmap.md#open-questions).

## PID Binding GC
**Open** — GC on flow close, or keep for replay detection?

## Map Types
- `FLOW_STATS`: **LRU_HASH** (auto-evict)
- `PID_BINDINGS`: **HASH** (manual GC)

## Cross-References
- [design.md](design.md)
- [architecture.md](architecture.md)
- [data-model.md](data-model.md)
- [verifier-notes.md](verifier-notes.md)
