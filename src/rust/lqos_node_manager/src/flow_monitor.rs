use lqos_bus::{bus_request, BusRequest, BusResponse, FlowbeeData};
use rocket::serde::json::Json;
use crate::cache_control::NoCache;

#[get("/api/flows/dump_all")]
pub async fn all_flows_debug_dump() -> NoCache<Json<Vec<FlowbeeData>>> {
  let responses =
    bus_request(vec![BusRequest::DumpActiveFlows]).await.unwrap();
  let result = match &responses[0] {
    BusResponse::AllActiveFlows(flowbee) => flowbee.to_owned(),
    _ => Vec::new(),
  };

  NoCache::new(Json(result))
}

#[get("/api/flows/count")]
pub async fn count_flows() -> NoCache<Json<u64>> {
  let responses =
    bus_request(vec![BusRequest::CountActiveFlows]).await.unwrap();
  let result = match &responses[0] {
    BusResponse::CountActiveFlows(count) => *count,
    _ => 0,
  };

  NoCache::new(Json(result))
}