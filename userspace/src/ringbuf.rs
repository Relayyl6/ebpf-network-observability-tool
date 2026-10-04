use aya::maps::{RingBuf, MapData};
use common::AnomalyEvent;
use crate::anomaly::AnomalyEngine;
use tokio::sync::broadcast;

pub fn drain(ringbuf: &mut RingBuf<MapData>, engine: &mut AnomalyEngine, tx: &broadcast::Sender<String>) {
    while let Some(item) = ringbuf.next() {
        if item.len() < core::mem::size_of::<AnomalyEvent>() {
            continue;
        }
        let event = unsafe { core::ptr::read_unaligned(item.as_ptr() as *const AnomalyEvent) };
        engine.process_event(&event, tx);
    }
}
