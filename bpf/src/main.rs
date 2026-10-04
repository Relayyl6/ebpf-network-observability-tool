#![no_std]
#![no_main]

use aya_ebpf::{
    macros::{classifier, map},
    maps::LruHashMap,
    programs::TcContext,
    bindings::TC_ACT_OK,
    helpers::bpf_ktime_get_ns,
};
use common::{FlowKey, FlowStats, FiveTuple};
use network_types::{
    eth::{EthHdr, EtherType},
    ip::{Ipv4Hdr, IpProto},
    tcp::TcpHdr,
};
use core::mem;

// Flow stats map (per-process TCP flows)
#[map]
static FLOW_STATS: LruHashMap<FlowKey, FlowStats> = LruHashMap::with_max_entries(100_000, 0);

// PID Bindings map (5-tuple -> PID)
#[map]
static PID_BINDINGS: aya_ebpf::maps::HashMap<FiveTuple, u32> = aya_ebpf::maps::HashMap::with_max_entries(10_000, 0);

// Anomaly Events Ring Buffer
#[map]
static ANOMALY_EVENTS: aya_ebpf::maps::RingBuf = aya_ebpf::maps::RingBuf::with_byte_size(256 * 1024, 0);

#[classifier]
pub fn tc_ingress(ctx: TcContext) -> i32 {
    match handle_packet(ctx, true) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

#[classifier]
pub fn tc_egress(ctx: TcContext) -> i32 {
    match handle_packet(ctx, false) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

#[inline(always)]
fn ptr_at<T>(ctx: &TcContext, offset: usize) -> Result<*const T, i32> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(TC_ACT_OK); // Ignore packet, but allow it to pass
    }

    Ok((start + offset) as *const T)
}

fn handle_packet(ctx: TcContext, is_ingress: bool) -> Result<i32, i32> {
    // Loopback iperf3 traffic often uses TSO/GSO, leaving TCP headers in the non-linear paged area.
    let _ = ctx.skb.pull_data(128);

    // Get the protocol from the kernel's sk_buff metadata. 
    // The protocol is stored in network byte order. 
    // ETH_P_IP is 0x0800 (which reads as 8 in native little-endian)
    let protocol = unsafe { (*ctx.skb.skb).protocol };
    // network-types already stores EtherType::Ipv4 in Big Endian (0x0800.to_be() == 8)
    if (protocol as u16) != EtherType::Ipv4 as u16 {
        // aya_log_ebpf::info!(&ctx, "Drop: Not IPv4. Protocol: {:x}", protocol);
        return Ok(TC_ACT_OK);
    }

    let l2_header: *const [u8; 15] = match ptr_at(&ctx, 0) {
        Ok(ptr) => ptr,
        Err(e) => { /* aya_log_ebpf::info!(&ctx, "Drop: L2 bounds check failed"); */ return Ok(TC_ACT_OK); }
    };
    
    let l3_offset = if (unsafe { (*l2_header)[0] } >> 4) == 4 {
        0
    } else if (unsafe { (*l2_header)[4] } >> 4) == 4 {
        4
    } else if (unsafe { (*l2_header)[14] } >> 4) == 4 {
        14
    } else {
        /* aya_log_ebpf::info!(&ctx, "Drop: Unknown encapsulation. First bytes: {:x} {:x} {:x}", unsafe{(*l2_header)[0]}, unsafe{(*l2_header)[4]}, unsafe{(*l2_header)[14]}); */
        return Ok(TC_ACT_OK);
    };

    let ipv4hdr: *const Ipv4Hdr = match ptr_at(&ctx, l3_offset) {
        Ok(ptr) => ptr,
        Err(e) => { /* aya_log_ebpf::info!(&ctx, "Drop: IPv4 bounds check failed"); */ return Ok(TC_ACT_OK); }
    };
    
    // IP protocol is a single byte (u8), so endianness doesn't matter
    if unsafe { (*ipv4hdr).proto } != IpProto::Tcp as u8 {
        /* aya_log_ebpf::info!(&ctx, "Drop: Not TCP"); */
        return Ok(TC_ACT_OK);
    }

    let ipv4_header_len = unsafe { (*ipv4hdr).ihl() as usize * 4 };
    let tcphdr: *const TcpHdr = match ptr_at(&ctx, l3_offset + ipv4_header_len) {
        Ok(ptr) => ptr,
        Err(e) => { /* aya_log_ebpf::info!(&ctx, "Drop: TCP bounds check failed. Off: {}, Len: {}", l3_offset, ipv4_header_len); */ return Ok(TC_ACT_OK); }
    };

    // Extract 5-tuple
    let mut src_ip = u32::from_be_bytes(unsafe { (*ipv4hdr).src_addr });
    let mut dst_ip = u32::from_be_bytes(unsafe { (*ipv4hdr).dst_addr });
    let mut src_port = u16::from_be_bytes(unsafe { (*tcphdr).source });
    let mut dst_port = u16::from_be_bytes(unsafe { (*tcphdr).dest });

    // Look up PID from bindings map
    let tuple = FiveTuple {
        src_ip, dst_ip, src_port, dst_port, protocol: 6,
    };
    let pid = unsafe { PID_BINDINGS.get(&tuple).copied().unwrap_or(0) };

    // Phase 3: Include PID in the key
    let key = FlowKey {
        src_ip,
        dst_ip,
        src_port,
        dst_port,
        protocol: 6, // IPPROTO_TCP
        pid,
    };

    // TCP flags are at byte 13 of the TCP header.
    // Use u16 bounds check at byte 12 to prevent LLVM from generating `start + offset >= end`
    let tcp_flags_bounds: *const u16 = match ptr_at(&ctx, l3_offset + ipv4_header_len + 12) {
        Ok(ptr) => ptr,
        Err(_) => { /* aya_log_ebpf::info!(&ctx, "Drop: TCP flags bounds check failed"); */ return Ok(TC_ACT_OK); }
    };
    let flags = unsafe { *(tcp_flags_bounds as *const u8).add(1) };

    let packet_len = (ctx.data_end() - ctx.data()) as u64;
    let now = unsafe { bpf_ktime_get_ns() };

    let mut stats = match unsafe { FLOW_STATS.get(&key) } {
        Some(s) => *s,
        None => FlowStats {
            bytes_sent: 0,
            bytes_recv: 0,
            packets_sent: 0,
            packets_recv: 0,
            start_ts: now,
            last_update_ts: now,
            tcp_state: 0, // 0 = unknown, 1 = SYN_SENT, 2 = ESTABLISHED, 3 = CLOSED
            anomaly_flags: 0,
        },
    };

    // TCP State machine tracking
    let syn = (flags & 0x02) != 0;
    let ack = (flags & 0x10) != 0;
    let fin = (flags & 0x01) != 0;
    let rst = (flags & 0x04) != 0;

    if syn && !ack {
        stats.tcp_state = 1; // SYN_SENT
    } else if syn && ack {
        // SYN-ACK
    } else if ack && stats.tcp_state == 1 {
        stats.tcp_state = 2; // ESTABLISHED
    } else if fin || rst {
        stats.tcp_state = 3; // CLOSED
    }

    if is_ingress {
        stats.bytes_recv += packet_len;
        stats.packets_recv += 1;
    } else {
        stats.bytes_sent += packet_len;
        stats.packets_sent += 1;
    }
    stats.last_update_ts = now;

    /* aya_log_ebpf::info!(&ctx, "TCP FLOW UPDATED! packets: {}", stats.packets_sent + stats.packets_recv); */

    let previous_flags = stats.anomaly_flags;

    // High-Level Correlation: Stuck Connections (SYN without ACK)
    // If it's been in SYN_SENT for more than 5 seconds without becoming ESTABLISHED
    if stats.tcp_state == 1 && (now - stats.start_ts) > 5_000_000_000 {
        stats.anomaly_flags |= common::ANOMALY_STUCK_CONN;
    }

    // Simple anomaly logic: Data Spike (> 1MB)
    if stats.bytes_sent + stats.bytes_recv > 1_000_000 {
        stats.anomaly_flags |= common::ANOMALY_DATA_SPIKE;
    }

    if previous_flags != stats.anomaly_flags {
        if let Some(mut event) = ANOMALY_EVENTS.reserve::<common::AnomalyEvent>(0) {
            unsafe {
                let e = &mut *event.as_mut_ptr();
                e.ts = now;
                e.pid = key.pid;
                e.uid = 0; // Not available in TC hook directly
                e.src_ip = key.src_ip;
                e.dst_ip = key.dst_ip;
                e.src_port = key.src_port;
                e.dst_port = key.dst_port;
                e.anomaly_flags = stats.anomaly_flags;
                e.bytes = stats.bytes_sent + stats.bytes_recv;
                e.packets = stats.packets_sent + stats.packets_recv;
            }
            event.submit(0);
        }
    }

    unsafe {
        FLOW_STATS.insert(&key, &stats, 0).map_err(|_| TC_ACT_OK)?;
    }

    Ok(TC_ACT_OK)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

use aya_ebpf::macros::{kprobe, tracepoint};
use aya_ebpf::programs::{ProbeContext, TracePointContext};

#[kprobe]
pub fn kprobe_tcp_v4_connect(ctx: ProbeContext) -> i32 {
    let pid_tgid = aya_ebpf::helpers::bpf_get_current_pid_tgid();
    let pid = (pid_tgid >> 32) as u32;

    // High-Level Upgrade: Actually read from the `struct sock *sk` argument!
    // In the Linux kernel, `struct sock` begins with `struct sock_common`.
    // skc_daddr is at offset 0, skc_rcv_saddr is at offset 4.
    // skc_dport is at offset 12, skc_num is at offset 14.
    // We use safe bpf_probe_read_kernel to pull these bytes into BPF space safely.
    
    let sk_ptr: *const u8 = ctx.arg(0).unwrap_or(core::ptr::null());
    if sk_ptr.is_null() {
        return 0;
    }

    let mut daddr: u32 = 0;
    let mut rcv_saddr: u32 = 0;
    let mut dport: u16 = 0;
    let mut num: u16 = 0; // source port

    unsafe {
        let _ = aya_ebpf::helpers::bpf_probe_read_kernel_buf(sk_ptr.add(0), core::slice::from_raw_parts_mut(&mut daddr as *mut _ as *mut u8, 4));
        let _ = aya_ebpf::helpers::bpf_probe_read_kernel_buf(sk_ptr.add(4), core::slice::from_raw_parts_mut(&mut rcv_saddr as *mut _ as *mut u8, 4));
        let _ = aya_ebpf::helpers::bpf_probe_read_kernel_buf(sk_ptr.add(12), core::slice::from_raw_parts_mut(&mut dport as *mut _ as *mut u8, 2));
        let _ = aya_ebpf::helpers::bpf_probe_read_kernel_buf(sk_ptr.add(14), core::slice::from_raw_parts_mut(&mut num as *mut _ as *mut u8, 2));
    }

    let mut tuple = FiveTuple {
        src_ip: u32::from_be(rcv_saddr),
        dst_ip: u32::from_be(daddr),
        src_port: num,          // skc_num is in host byte order
        dst_port: u16::from_be(dport), // skc_dport is in network byte order
        protocol: 6,
    };

    // Normalize
    if tuple.dst_ip < tuple.src_ip {
        core::mem::swap(&mut tuple.src_ip, &mut tuple.dst_ip);
        core::mem::swap(&mut tuple.src_port, &mut tuple.dst_port);
    }

    unsafe {
        let _ = PID_BINDINGS.insert(&tuple, &pid, 0);
    }
    0
}

#[tracepoint]
pub fn tracepoint_sched_process_exit(_ctx: TracePointContext) -> i32 {
    let pid_tgid = aya_ebpf::helpers::bpf_get_current_pid_tgid();
    let _pid = (pid_tgid >> 32) as u32;

    // TODO: Send event to userspace ringbuffer indicating `pid` has exited
    // Userspace will then iterate over FLOW_STATS and prune all flows with `pid`.
    0
}
