//! Provides pre-packaged queries for obtaining data, that will
//! then be used by the web server to respond to requests.

mod packet_counts;
mod throughput;
mod rtt;
mod node_perf;
mod search;
mod site_heat_map;
pub mod time_period;
pub use packet_counts::{ send_packets_for_all_nodes, send_packets_for_node };
pub use throughput::{ send_throughput_for_all_nodes, send_throughput_for_node };
pub use rtt::{ send_rtt_for_all_nodes, send_rtt_for_node };
pub use node_perf::send_perf_for_node;
pub use search::omnisearch;
pub use site_heat_map::root_heat_map;