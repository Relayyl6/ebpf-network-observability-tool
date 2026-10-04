# Runbook — Operating the Tool

> Design: [design.md](design.md). Architecture: [architecture.md](architecture.md).

## Start
```bash
# Since we use xtask, you can run:
sudo cargo run --package xtask -- run
# OR directly
sudo ./target/release/userspace --iface eth0
```

## Debug Verifier Rejections
1. Check `dmesg | tail`
2. Run `cargo build --package bpf` to ensure it compiles.
3. See [verifier-notes.md](verifier-notes.md) for error table

## Inspect Maps
```bash
bpftool map show
bpftool map dump name FLOW_STATS
```

## Ring Buffer Full
- Increase size in Rust `RingBuf` configuration.
- Reduce event emission (filter in kernel)
- Profile userspace drain (check tokio task starvation)

## Detach
```bash
bpftool prog detach ...
# or Ctrl-C the userspace process (Aya handles detaching cleanly on exit)
```

## Cross-References
- [architecture.md](architecture.md)
- [testing.md](testing.md)
- [verifier-notes.md](verifier-notes.md)
