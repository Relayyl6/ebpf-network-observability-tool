# AGENTS.md — Guide for AI Coding Agents

This repository is a **phased eBPF networking project**. Before making
changes, read [README.md](README.md) and [docs/design.md](docs/design.md).

## Ground Rules for Agents

1. **Never propose unbounded loops in BPF code.** See
   [docs/verifier-notes.md#bounded-loops](docs/verifier-notes.md#bounded-loops).
2. **Never dereference a pointer without a bounds check.** See
   [docs/verifier-notes.md#pointer-arithmetic](docs/verifier-notes.md#pointer-arithmetic).
3. **Keep kernel-side structs under ~200 bytes on stack.** See
   [docs/verifier-notes.md#stack-depth](docs/verifier-notes.md#stack-depth).
4. **Do not add new BPF maps without updating** [docs/data-model.md](docs/data-model.md).
5. **Do not change map key/value layouts without updating**
   [docs/data-model.md](docs/data-model.md) and bumping a schema version.
6. **Anomaly logic belongs in userspace unless it is a binary flag.**
   See [docs/decisions.md](docs/decisions.md) and [docs/design.md](docs/design.md).
7. **Every new userspace module must be referenced in** [docs/architecture.md](docs/architecture.md).

## Agent Workflow

When asked to implement a phase:

1. Read the phase in [docs/roadmap.md](docs/roadmap.md).
2. Read the relevant section in [docs/design.md](docs/design.md).
3. Check [docs/decisions.md](docs/decisions.md) for locked-in choices.
4. Write BPF code in `bpf/src/`, shared structures in `common/src/lib.rs`, userspace in `userspace/src/`.
5. Add verifier notes to [docs/verifier-notes.md](docs/verifier-notes.md)
   if you discover a new constraint.
6. Update [docs/architecture.md](docs/architecture.md) if you add a
   component or data path.
7. Run `scripts/stress.sh` and report results in [docs/testing.md](docs/testing.md).

## What Agents Should Not Do

- Do not silently change the anomaly detection strategy. It is **hybrid**:
  flags in kernel, thresholds in userspace. See [docs/decisions.md](docs/decisions.md).
- Do not replace ring buffer with perf buffer without a documented reason.
  See [docs/decisions.md#ring-vs-perf](docs/decisions.md#ring-vs-perf).
- Do not introduce tokio async into userspace without a benchmark.
  See [docs/decisions.md#sync-vs-async](docs/decisions.md#sync-vs-async).
- Do not add dependencies to `Cargo.toml` without listing them in
  [docs/architecture.md#dependencies](docs/architecture.md#dependencies).

## Cross-References
- [README.md](README.md)
- [docs/design.md](docs/design.md)
- [docs/architecture.md](docs/architecture.md)
- [docs/data-model.md](docs/data-model.md)
- [docs/verifier-notes.md](docs/verifier-notes.md)
- [docs/decisions.md](docs/decisions.md)
- [docs/roadmap.md](docs/roadmap.md)
- [docs/testing.md](docs/testing.md)
- [docs/runbook.md](docs/runbook.md)
- [docs/glossary.md](docs/glossary.md)
