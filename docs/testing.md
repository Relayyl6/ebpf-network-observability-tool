# Testing — Load & Integration

> Roadmap: [roadmap.md](roadmap.md). Runbook: [runbook.md](runbook.md).

## Integration
- `tests/integration/` — spawn tool, generate traffic, assert events
- Use `iperf3` for synthetic flows

## Load
- `scripts/stress.sh` — 10K, 100K, 1M flows
- Measure: CPU%, latency (`perf stat`), ring buffer drop rate

## Edge Cases
- Map full → LRU eviction
- Ring buffer full → event loss
- Process exits mid-flow → orphan handling
- Long-running (1h+) → memory stability

## Verifier Profiling
- `bpftool prog stat` — instruction count
- `bpftool prog show` — attach state

## Cross-References
- [roadmap.md](roadmap.md)
- [runbook.md](runbook.md)
- [verifier-notes.md](verifier-notes.md)
