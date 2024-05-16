use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use serde::Serialize;
use once_cell::sync::Lazy;
use crate::ChangeAnnouncement;

pub static FLOW_COUNT: AtomicUsize = AtomicUsize::new(0);
pub static SHAPED_DEVICE_COUNT: AtomicUsize = AtomicUsize::new(0);

pub static TOTAL_BITS_PER_SECOND: (AtomicU64, AtomicU64) = (AtomicU64::new(0), AtomicU64::new(0));
pub static SHAPED_BITS_PER_SECOND: (AtomicU64, AtomicU64) = (AtomicU64::new(0), AtomicU64::new(0));
pub static PACKETS_PER_SECOND: (AtomicU64, AtomicU64) = (AtomicU64::new(0), AtomicU64::new(0));

pub async fn track_changes(mut receiver: Receiver<ChangeAnnouncement>) {
    while let Some(msg) = receiver.recv().await {
        match msg {
            ChangeAnnouncement::FlowCount(count) => FLOW_COUNT.store(count, Relaxed),
            ChangeAnnouncement::ShapedDeviceCount(count) => SHAPED_DEVICE_COUNT.store(count, Relaxed),
            ChangeAnnouncement::ThroughputUpdate { bytes_per_second, shaped_bytes_per_second, packets_per_second } => {
                TOTAL_BITS_PER_SECOND.0.store(bytes_per_second.0 * 8, Relaxed);
                TOTAL_BITS_PER_SECOND.1.store(bytes_per_second.1 * 8, Relaxed);
                SHAPED_BITS_PER_SECOND.0.store(shaped_bytes_per_second.0 * 8, Relaxed);
                SHAPED_BITS_PER_SECOND.1.store(shaped_bytes_per_second.1 * 8, Relaxed);
                PACKETS_PER_SECOND.0.store(packets_per_second.0, Relaxed);
                PACKETS_PER_SECOND.1.store(packets_per_second.1, Relaxed);
                
                let mut lock = THROUGHPUT_RING_BUFFER.lock().unwrap();
                lock.push(bytes_per_second, shaped_bytes_per_second);
            }
        }
    }
}

pub static THROUGHPUT_RING_BUFFER: Lazy<Mutex<ThroughputRingBuffer>> = Lazy::new(|| Mutex::new(ThroughputRingBuffer::new()));

#[derive(Clone, Copy, Serialize)]
pub struct ThroughputEntry {
    bps: [u64; 2],
    shaped: [u64; 2],
}

pub struct ThroughputRingBuffer {
    head: usize,
    entries: Vec<ThroughputEntry>,
}

impl ThroughputRingBuffer {
    fn new() -> Self {
        let entries = vec![ThroughputEntry{ bps: [0,0], shaped: [0,0] }; 300];
        Self {
            head: 0,
            entries,
        }
    }
    
    fn push(&mut self, bps: (u64, u64), shaped: (u64, u64)) {
        let entry = ThroughputEntry {
            bps: [bps.0 * 8, bps.1 * 8],
            shaped: [shaped.0 * 8, shaped.1 * 8]
        };
        self.entries[self.head] = entry;
        self.head += 1;
        self.head %= 300;
    }
    
    pub fn fetch(&self) -> Vec<ThroughputEntry> {
        let mut result = Vec::with_capacity(300);
        for i in self.head .. 300 {
            result.push(self.entries[i]);
        }
        for i in 0 .. self.head {
            result.push(self.entries[i]);
        }
        result
    }
}