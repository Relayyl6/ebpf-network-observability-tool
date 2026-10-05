# Design — eBPF Network Observability Tool

> This document is the **mental model** and rationale. For component layout
> see [architecture.md](architecture.md). For locked-in choices see
> [decisions.md](decisions.md). For verifier constraints see
> [verifier-notes.md](verifier-notes.md).

## 1. Core Constraint

The eBPF verifier is not a compiler — it is a **safety proof**. Every line
of kernel-space code is a negotiation. You cannot:

- Use unbounded loops
- Dereference pointers without proving validity
- Allocate memory dynamically
- Call arbitrary kernel functions

You *can*:

- Use bounded loops (`#pragma unroll` or Rust Iterators that are bounded)
- Inspect `sk_buff` via CO-RE
- Use pre-allocated BPF maps
- Call a whitelist of BPF helpers

**Consequence for design**: keep the kernel path minimal and latency-critical;
push complexity to userspace. This is why the architecture is **hybrid**.

See [verifier-notes.md](verifier-notes.md) for the specific walls.

## 2. Anomaly Detection Strategy

We use **Option C: Hybrid**.

```
Kernel  : Set binary flags (HIGH_LOSS, STUCK_CONN, DATA_SPIKE) in flow_stats
Userspace: Read flags, apply multi-flow correlation and thresholds
```

Rationale: fast kernel signaling + flexible userspace logic.
See [decisions.md](decisions.md#anomaly-detection).

## 3. Hook Strategy

We use **TC + kprobe hybrid**:

| Hook | Purpose |
|------|---------|
| TC ingress | Count received packets/bytes |
| TC egress  | Count sent packets/bytes |
| kprobe `tcp_v4_connect` | Bind 5-tuple → PID at connection time |
| tracepoint `sched_process_exit` | Mark flows orphaned / GC |

Rationale: TC sees all traffic but lacks PID context; kprobe provides PID
at connection setup. See [decisions.md](decisions.md#hook-strategy).

## 4. Map Strategy

Two maps:

1. **`FLOW_STATS`** — `BPF_MAP_TYPE_LRU_HASH`
   - Key: `FlowKey` (5-tuple + PID)
   - Value: `FlowStats`
   - ~100K entries
2. **`PID_BINDINGS`** — `BPF_MAP_TYPE_HASH`
   - Key: `FiveTuple`
   - Value: `u32 pid`
   - ~10K entries

See [data-model.md](data-model.md) for full schemas.

## 5. Signaling Path

- Kernel → userspace: **ring buffer** (`BPF_MAP_TYPE_RINGBUF`) via `aya-ebpf`
- Userspace → consumer: stdout / JSON / optional Prometheus
- See [architecture.md](architecture.md#data-flow)

## 6. Userspace Model

- Rust `tokio` driven polling, 10ms poll cycles
- Drain ring buffer → apply thresholds → snapshot flow map → emit
- See [architecture.md](architecture.md#userspace)

## 7. Verifier Risk Map

| Phase | Risk | Gate |
|-------|------|------|
| 1–2 | Low | First BPF program loads |
| 3–4 | Medium | TC + kprobe coexist |
| 5–6 | High | Kernel-side flags compile |
| 7–8 | Medium | Ring buffer backpressure |
| 9–10 | Low | Single binary loads |
| 11–12 | Low | 1h stability |

See [roadmap.md](roadmap.md) and [verifier-notes.md](verifier-notes.md).


## Cross-References
- [README.md](../README.md)
- [architecture.md](architecture.md)
- [data-model.md](data-model.md)
- [decisions.md](decisions.md)
- [verifier-notes.md](verifier-notes.md)
- [roadmap.md](roadmap.md)
- [glossary.md](glossary.md)
