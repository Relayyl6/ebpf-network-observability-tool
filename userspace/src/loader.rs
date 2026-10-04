use aya::Ebpf;
use aya::programs::{tc, SchedClassifier, TcAttachType, KProbe, TracePoint};
use log::info;
use anyhow::Context;

pub fn load_and_attach(bpf: &mut Ebpf, iface: &str) -> anyhow::Result<()> {
    // Add clsact qdisc to the interface to allow TC attach
    if let Err(e) = tc::qdisc_add_clsact(iface) {
        log::warn!("Failed to add clsact qdisc (it might already exist): {}", e);
    }
    
    // TC Ingress
    let prog_ingress: &mut SchedClassifier = bpf.program_mut("tc_ingress").unwrap().try_into()?;
    prog_ingress.load()?;
    prog_ingress.attach(iface, TcAttachType::Ingress).context("Failed to attach tc_ingress")?;
    info!("Attached TC ingress program to {}", iface);

    // TC Egress
    let prog_egress: &mut SchedClassifier = bpf.program_mut("tc_egress").unwrap().try_into()?;
    prog_egress.load()?;
    prog_egress.attach(iface, TcAttachType::Egress).context("Failed to attach tc_egress")?;
    info!("Attached TC egress program to {}", iface);

    // Kprobe tcp_v4_connect
    let kprobe: &mut KProbe = bpf.program_mut("kprobe_tcp_v4_connect").unwrap().try_into()?;
    kprobe.load()?;
    kprobe.attach("tcp_v4_connect", 0).context("Failed to attach kprobe")?;
    info!("Attached kprobe to tcp_v4_connect");

    // Tracepoint sched_process_exit
    let tracepoint: &mut TracePoint = bpf.program_mut("tracepoint_sched_process_exit").unwrap().try_into()?;
    tracepoint.load()?;
    tracepoint.attach("sched", "sched_process_exit").context("Failed to attach tracepoint")?;
    info!("Attached tracepoint to sched_process_exit");

    Ok(())
}
