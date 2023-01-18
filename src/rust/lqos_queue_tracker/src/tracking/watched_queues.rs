use crate::queue_structure::QUEUE_STRUCTURE;
use lazy_static::*;
use lqos_bus::TcHandle;
use parking_lot::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static! {
    pub(crate) static ref WATCHED_QUEUES: RwLock<Vec<WatchedQueue>> = RwLock::new(Vec::new());
}

pub(crate) struct WatchedQueue {
    circuit_id: String,
    expires_unix_time: u64,
    download_class: TcHandle,
    upload_class: TcHandle,
}

impl WatchedQueue {
    pub(crate) fn get(&self) -> (&str, TcHandle, TcHandle) {
        (&self.circuit_id, self.download_class, self.upload_class)
    }

    pub(crate) fn refresh_timer(&mut self) {
        self.expires_unix_time = expiration_in_the_future();
    }
}

pub fn expiration_in_the_future() -> u64 {
    unix_now() + 10
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn add_watched_queue(circuit_id: &str) {
    let max = unsafe { lqos_sys::libbpf_num_possible_cpus() } * 2;
    {
        let read_lock = WATCHED_QUEUES.read();
        if read_lock
            .iter()
            .find(|q| q.circuit_id == circuit_id)
            .is_some()
        {
            return; // No duplicates, please
        }

        if read_lock.len() > max as usize {
            return; // Too many watched pots
        }
    }

    if let Some(queues) = &QUEUE_STRUCTURE.read().maybe_queues {
        if let Some(circuit) = queues
            .iter()
            .find(|c| c.circuit_id.is_some() && c.circuit_id.as_ref().unwrap() == circuit_id)
        {
            let new_watch = WatchedQueue {
                circuit_id: circuit.circuit_id.as_ref().unwrap().clone(),
                expires_unix_time: expiration_in_the_future(),
                download_class: circuit.class_id,
                upload_class: circuit.up_class_id,
            };

            WATCHED_QUEUES.write().push(new_watch);
        }
    } else {
        log::warn!("No circuit ID of {circuit_id}");
    }
}

pub(crate) fn expire_watched_queues() {
    let mut lock = WATCHED_QUEUES.write();
    let now = unix_now();
    lock.retain(|w| w.expires_unix_time > now);
}

pub fn still_watching(circuit_id: &str) {
    let mut lock = WATCHED_QUEUES.write();
    if let Some(q) = lock.iter_mut().find(|q| q.circuit_id == circuit_id) {
        q.refresh_timer();
    }
}