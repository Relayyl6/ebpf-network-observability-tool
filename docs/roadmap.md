# Roadmap — 12-Week Phase Plan

> Rationale: [design.md](design.md). Decisions: [decisions.md](decisions.md).
> Verifier risks: [verifier-notes.md](verifier-notes.md).

## Phase 1 — Foundation (Weeks 1–2)
- Kernel networking deep dive
- Verifier fundamentals
- **Deliverable**: first BPF program loads

## Phase 2 — Flow Tracking (Weeks 3–4)
- Map design
- TC ingress + egress skeleton
- **Deliverable**: TC hooks load, flow map populates

## Phase 3 — PID Binding (Weeks 5–6)
- kprobe on `tcp_v4_connect`
- `PID_BINDINGS` map
- Process lifecycle via `sched_process_exit`
- **Deliverable**: per-process flows tracked

## Phase 4 — Anomaly Detection (Weeks 7–8)
- Binary flags in kernel
- Ring buffer events
- **Deliverable**: anomaly events reach userspace

## Phase 5 — Userspace & Binary (Weeks 9–10)
- aya-rs harness
- Event loop
- Single static binary
- **Deliverable**: `./userspace` runs end-to-end

## Phase 6 — Testing & Docs (Weeks 11–12)
- Load tests
- Verifier limit profiling
- Blog post + runbook
- **Deliverable**: 1h stability, documentation complete

## Open Questions
- [ ] Flow lifetime: outlive process?
- [ ] PID binding GC strategy?
- [ ] Prometheus exporter in MVP or later?

## Cross-References
- [design.md](design.md)
- [decisions.md](decisions.md)
- [testing.md](testing.md)
- [verifier-notes.md](verifier-notes.md)
