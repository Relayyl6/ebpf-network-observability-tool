# Week-by-Week Build Plan: eBPF Network Observability Tool

I'll structure this as a phased breakdown. Each phase has a clear learning objective and verifier challenge:

### **Phase 1: Foundation & Kernel Networking Mental Model (Weeks 1–2)**
... (See README for architectural details based on this plan)

**Week 1: Kernel Networking Deep Dive**
- Study Linux TCP state machine and netfilter hook points
- Set up lab environment (Linux VM, bpftool, llvm-objdump)

**Week 2: eBPF Verifier Fundamentals**
- Deep dive: verifier rules
- Study aya-rs architecture and its verifier guidance
- Build confidence reading verifier rejection logs

### **Phase 2: TCP Flow Tracking Infrastructure (Weeks 3–4)**
**Week 3: Map Design & Data Structure Layout**
- Define flow key and flow stats maps

**Week 4: BPF Program Skeleton & Safe Concurrency**
- Write the core attachment point (e.g., TC ingress + egress hooks)

### **Phase 3: Per-Process Binding & Process Tracking (Weeks 5–6)**
**Week 5: PID Binding & Process Tracking**
- Attach kprobes to `tcp_v4_connect` to bind flow to PID at connection time

**Week 6: Process Lifecycle Management**
- Detect process exits via `sched_process_exit`

### **Phase 4: Anomaly Detection Logic (Weeks 7–8)**
**Week 7: Anomaly Criteria & Kernel-Side Detection**
- Implement bitfield flags in kernel (HIGH_LOSS, STUCK_CONN, etc.)

**Week 8: Ring Buffer Event Emission**
- Design ring buffer event schema
- Emit events when anomaly flags set

### **Phase 5: Userspace Orchestration & Single Binary (Weeks 9–10)**
**Week 9: aya-rs Integration & Userspace State Machine**
- Build event loop reading maps and draining ring buffers

**Week 10: Single Binary & Deployment**
- Embed BPF bytecode in Rust binary

### **Phase 6: Testing, Hardening & Documentation (Weeks 11–12)**
**Week 11: Load Testing & Verifier Limits**
- Generate synthetic load
- Handle edge cases

**Week 12: Documentation & Portfolio Narrative**
- Document anomaly detection trade-offs
