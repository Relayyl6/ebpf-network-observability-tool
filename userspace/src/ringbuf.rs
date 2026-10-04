use aya::maps::{RingBuf, MapData};
use common::AnomalyEvent;
use crate::anomaly::AnomalyEngine;

pub fn drain(ringbuf: &mut RingBuf<MapData>, engine: &mut AnomalyEngine) {
    while let Some(item) = ringbuf.next() {
        if item.len() < core::mem::size_of::<AnomalyEvent>() {
            continue;
        }
        let event = unsafe { core::ptr::read_unaligned(item.as_ptr() as *const AnomalyEvent) };
        engine.process_event(&event);
    }
}
