# eBPF Network Observability Tool

This project is a high-performance, low-overhead network observability engine built with **eBPF (Extended Berkeley Packet Filter)** and **Rust**. 
It acts as a kernel-level security and monitoring platform that intercepts network traffic at the lowest possible level before it even reaches the operating system's networking stack.

### What is the goal of this project?
The goal is to track every single TCP connection on a Linux machine in real-time without slowing down the server. 
It maps raw network packets directly to the process ID (PID) that created them, allowing you to answer the critical security question: *"Which specific application on my server is communicating with this IP address?"*

It also features a sliding-window **Anomaly Engine** that can detect suspicious behavior (like massive data exfiltration spikes or stuck SYN-floods) and emit structured JSON alerts to userspace within microseconds.

## Quick Links
- [Design](docs/design.md) — mental model, constraints, architecture
- [Architecture](docs/architecture.md) — component diagram and data flow
- [Data Model](docs/data-model.md) — flow_key, flow_stats, maps
- [Verifier Notes](docs/verifier-notes.md) — the walls you will hit
- [Decisions](docs/decisions.md) — trade-off matrix (locked-in choices)
- [Roadmap](docs/roadmap.md) — 12-week phase plan
- [Testing](docs/testing.md) — load + integration strategy
- [Runbook](docs/runbook.md) — operating the tool in production
- [Glossary](docs/glossary.md) — shared vocabulary
- [AGENTS.md](AGENTS.md) — how AI coding agents should work in this repo

## What It Does
- Attaches **TC ingress + egress** hooks for packet-level visibility
- Attaches a **kprobe on `tcp_v4_connect`** to bind 5-tuple → PID
- Attaches a **tracepoint on `sched_process_exit`** to garbage-collect flows
- Sets anomaly **flags in the kernel** (`HIGH_LOSS`, `STUCK_CONN`, `DATA_SPIKE`)
- Emits **ring buffer events** to a Rust userspace consumer
- Applies **thresholds and correlation** in userspace
- Ships as a **single static binary** (aya-rs + embedded BPF bytecode)

## Running on Windows (WSL)
Because eBPF requires a Linux kernel, Windows users must run this inside WSL (Windows Subsystem for Linux).

**1. Initialize the Environment**
Open a WSL terminal and run the setup script to install LLVM, Clang, and generic Linux tools:
```bash
bash scripts/lab-setup.sh
source $HOME/.cargo/env
rustup toolchain install nightly --component rust-src
```

**2. Install bpf-linker**
Since WSL runs a custom Microsoft kernel, compiling `bpf-linker` from scratch can fail due to missing C++ LLVM shared libraries. The safest method is to download the pre-compiled binary using `cargo-binstall`:
```bash
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
cargo binstall -y bpf-linker
```

**3. Build and Run**
Use the absolute path to `cargo` so `sudo` doesn't lose the Rust environment variables. 
To see the internal heartbeat logs and alerts, you MUST prefix the command with `RUST_LOG=info`.

```bash
sudo -E RUST_LOG=info ~/.cargo/bin/cargo run --package xtask -- run
```

### Understanding the Run Command
The execution command is heavily chained because we are building a kernel payload and running it with root privileges in one shot. Here is the breakdown of exactly what it does:

* `sudo -E`: Runs the command as root (required to load eBPF programs). The `-E` preserves your user environment variables (like your `PATH` and `RUST_LOG`) so Cargo can find your toolchains.
* `RUST_LOG=info`: Sets the logging level. By default, `env_logger` hides info-level prints. We need this to see the continuous heartbeat `Snapshot` logs. (Alternatives: `RUST_LOG=debug` or `RUST_LOG=trace` for deeper verifier logs).
* `~/.cargo/bin/cargo`: We use the absolute path to cargo because some Linux distributions aggressively strip the `PATH` variable when switching to the `root` user, even with `-E`.
* `run --package xtask -- run`: This tells Cargo to execute our custom `xtask` orchestrator. The orchestrator compiles the eBPF kernel code to bytecode, then compiles the userspace code, and finally launches the userspace binary.
* `-- --iface lo`: The double dashes (`--`) tell Cargo to pass the remaining arguments directly to our userspace application instead of Cargo itself. 

### CLI Arguments & Network Interfaces
When you run an eBPF network tool, it doesn't just listen to all traffic globally. It physically attaches its hooks (via Traffic Control) to a specific piece of networking hardware on your machine, known as a **Network Interface**.

If you attach the tool to the wrong interface, it will see zero traffic. The `userspace` binary allows you to specify the interface using the `-i` or `--iface` argument:

- **`eth0` (Ethernet - Default):** 
  This is the standard name for a hardwired ethernet connection. In WSL, Microsoft creates a virtual `eth0` adapter to bridge your Linux environment to your Windows host. Use this to monitor standard outgoing web traffic or external database connections.
- **`lo` (Loopback):** 
  This is the internal, virtual network interface. When a program on your computer talks to another program on the *same* computer (e.g., `127.0.0.1` or `localhost`), the traffic never leaves the motherboard. Use `--iface lo` when monitoring local microservices or running local stress tests.
- **`wlan0` (Wireless LAN):** 
  This is the standard Linux name for a Wi-Fi card. If you deploy this tool on a native Linux laptop connected to Wi-Fi, all your internet traffic flows through `wlan0`. If you leave the default as `eth0`, the tool will see nothing.

*Example:* To attach to a Wi-Fi interface with debug logging:
`sudo -E RUST_LOG=debug ~/.cargo/bin/cargo run --package xtask -- run -- --iface wlan0`

**4. Trigger Anomalies (Stress Test)**
The stress testing script uses `iperf3` to blast gigabits of traffic locally to `127.0.0.1`. Localhost traffic traverses the **loopback** interface (`lo`), NOT `eth0`. 

To observe the stress test, you must explicitly attach the orchestrator to `lo`:
```bash
# Terminal 1: Start orchestrator on loopback
sudo -E RUST_LOG=info ~/.cargo/bin/cargo run --package xtask -- run -- --iface lo
```

Then, in a second WSL terminal tab, unleash the synthetic traffic:
```bash
# Terminal 2: Trigger traffic flood
bash scripts/stress.sh
```

## How it Works (Anomaly Detection)

When you run the stress test (`scripts/stress.sh`), `iperf3` blasts up to 30 Gigabits of data through the loopback interface. 

Our eBPF Kernel program inspects every single packet in microseconds. It tracks the exact byte counts for every active TCP flow (the 5-tuple).
Once a flow's total data volume crosses the `1MB` threshold, it triggers a `DATA_SPIKE` kernel anomaly.

The kernel safely serializes this anomaly event over an eBPF Ring Buffer into Userspace. The Rust Userspace application receives it asynchronously, parses it into JSON, and evaluates it through a Sliding Window Correlator.

If a single process (or flow) triggers too many anomalies within a 60-second window, the system escalates it into a high-level security alert!

### Example Output

When running under load, you will see output like this:

```json
[2026-10-04T12:15:28Z WARN  userspace::anomaly] CRITICAL: PID 0 has triggered 1297 anomalies in the last 60 seconds! Potential exfiltration or attack.
{"ts":20720080943888,"pid":0,"src_ip":"127.0.0.1","dst_ip":"127.0.0.1","src_port":48688,"dst_port":25562,"anomaly_flags":2,"bytes":4096,"packets":32}

[2026-10-04T12:15:30Z INFO  userspace::flow_map] Snapshot: 12516 active TCP flows (135907456 total bytes)

[2026-10-04T12:15:33Z WARN  userspace::anomaly] CRITICAL: PID 0 has triggered 1640 anomalies in the last 60 seconds! Potential exfiltration or attack.
{"ts":20723701782376,"pid":0,"src_ip":"127.0.0.1","dst_ip":"127.0.0.1","src_port":51156,"dst_port":39012,"anomaly_flags":2,"bytes":384,"packets":3}
```

* **ts**: Timestamp in kernel nanoseconds.
* **pid**: The Process ID associated with the socket. *(Note: For short-lived loopback stress tests where the socket connects faster than the eBPF kprobe can intercept it, or for specific ephemeral fast-paths, this may fallback to 0).*
* **src_ip / dst_ip**: Extracted from the IPv4 header directly from kernel memory.
* **src_port / dst_port**: Extracted from the TCP header.
* **anomaly_flags**: Bitmask describing the rule that was triggered (e.g., `2` for `DATA_SPIKE`).
* **Snapshot**: Periodically prints the total active state residing inside the kernel's eBPF Hash Maps without interrupting traffic flow.

## Requirements
- Linux kernel ≥ 5.8 (WSL2 is natively supported)
- Rust nightly + `rust-src` (for building the kernel eBPF bytecode)
- `bpf-linker`

## Status
**Done** — All 6 Phases Complete.
The observability platform is fully built, hardened, and documented.

## Related Documents
- Agent workflow: [AGENTS.md](AGENTS.md)
- Design rationale: [docs/design.md](docs/design.md)
- Operational guide: [docs/runbook.md](docs/runbook.md)

## Debugging & Edge Cases

### 1. WSL and Null Encapsulation (The 4-Byte Loopback Header)
In native Linux, loopback traffic either uses a 0-byte header (raw IP) or a 14-byte Ethernet header. However, inside Windows Subsystem for Linux (WSL), the virtual networking stack frequently injects a 4-byte `AF_INET` (Null encapsulation) header! 
If your eBPF program assumes standard Ethernet encapsulation and blindly skips 14 bytes into the packet, it will read garbage memory and silently drop the flows.
**Solution:** Do a dynamic signature scan of the first 15 bytes. Look for the `0x45` signature (the standard IPv4 Version 4 + IHL 5 byte) at offset 0, offset 4, and offset 14. This allows your eBPF program to dynamically self-adjust and work flawlessly across native Linux, WSL, Wi-Fi, Ethernet, and VPN tunnels alike!

### 2. Loopback Interfaces and "Fake" Ethernet Headers
When attaching TC (Traffic Control) hooks to the loopback interface (`lo`), the Linux kernel aggressively optimizes local traffic. Instead of generating a full Ethernet header, it often prepends a "fake" zeroed-out header.
If your eBPF program attempts to manually parse `ethhdr->ether_type` by reading raw memory on `lo`, it will frequently read `0x0000` and incorrectly drop the packet.
**Solution:** Do not rely on raw Ethernet bytes for protocol detection on loopback. Always read the kernel-provided `skb->protocol` metadata field.

### 3. Endianness and `skb->protocol`
Network protocols transmit data in **Big-Endian** (network byte order), but x86_64 CPUs operate in **Little-Endian**. 
When inspecting `skb->protocol`, the value is preserved in network byte order. For example, IPv4 is defined as `0x0800`. Because your CPU reads this backward in memory as `0x0008`, if you print `skb->protocol` as an integer, it will output exactly `8`!
Always use `u16::from_be()` to safely convert network-byte values to native CPU endianness before comparing them to standard constants.

### 4. TSO/GSO and Paged Memory
When testing locally with `iperf3`, the kernel uses TCP Segmentation Offload (TSO / GSO) to send massive 64KB "jumbo" packets. In eBPF, `skb->data` only points to the linear memory area. For huge packets, the TCP headers are often pushed into the non-linear paged memory area.
If you attempt a bounds check (e.g., `start + offset > end`), it will fail and your program will drop the packet.
**Solution:** Always call `ctx.skb.pull_data(128)` at the start of your TC hook to force the kernel to pull the packet headers out of paged memory and into the linear area.

### 5. The eBPF Verifier and LLVM Optimizations
The eBPF verifier is notoriously strict about pointer arithmetic. To safely read a single byte (e.g., `start + 1 > end`), LLVM will often mathematically simplify the expression to `start >= end`. 
The verifier does not understand algebraic simplifications for pointers and will reject the program with `Permission denied (os error 13)`.
**Solution:** Trick LLVM by forcing a 2-byte bounds check (e.g., casting to `*const u16`), which prevents the `start >= end` optimization, allowing you to safely read the underlying 1-byte value.
