//! Provides a globally accessible vector of all flows. This is used to store
//! all flows for the purpose of tracking and data-services.

use super::{flow_analysis::FlowAnalysis, RttData};
use lqos_sys::flowbee_data::{FlowbeeData, FlowbeeKey};
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};
use serde::Serialize;
use lqos_utils::units::DownUpOrder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct AsnId(pub u32);

pub static ALL_FLOWS: Lazy<Mutex<HashMap<FlowbeeKey, (FlowbeeLocalData, FlowAnalysis)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));


/// Condensed representation of the FlowbeeData type. This contains
/// only the information we want to keep locally for analysis purposes,
/// adds RTT data, and uses Rust-friendly typing.
#[derive(Debug, Clone, Serialize)]
pub struct FlowbeeLocalData {
    /// Time (nanos) when the connection was established
    pub start_time: u64,
    /// Time (nanos) when the connection was last seen
    pub last_seen: u64,
    /// Bytes transmitted
    pub bytes_sent: DownUpOrder<u64>,
    /// Packets transmitted
    pub packets_sent: DownUpOrder<u64>,
    /// Rate estimate
    pub rate_estimate_bps: DownUpOrder<u32>,
    /// TCP Retransmission count (also counts duplicates)
    pub tcp_retransmits: DownUpOrder<u16>,
    /// Has the connection ended?
    /// 0 = Alive, 1 = FIN, 2 = RST
    pub end_status: u8,
    /// Raw IP TOS
    pub tos: u8,
    /// Raw TCP flags
    pub flags: u8,
    /// Recent RTT median
    pub rtt: [RttData; 2],
    /// Throughput Buffer
    pub throughput_buffer: Vec<DownUpOrder<u64>>,
}

impl From<&FlowbeeData> for FlowbeeLocalData {
    fn from(data: &FlowbeeData) -> Self {
        Self {
            start_time: data.start_time,
            last_seen: data.last_seen,
            bytes_sent: data.bytes_sent,
            packets_sent: data.packets_sent,
            rate_estimate_bps: data.rate_estimate_bps,
            tcp_retransmits: data.tcp_retransmits,
            end_status: data.end_status,
            tos: data.tos,
            flags: data.flags,
            rtt: [RttData::from_nanos(0); 2],
            throughput_buffer: vec![ data.bytes_sent ],
        }
    }
}