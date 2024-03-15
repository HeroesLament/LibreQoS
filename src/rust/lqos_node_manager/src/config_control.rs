use crate::{auth_guard::AuthGuard, cache_control::NoCache};
use default_net::get_interfaces;
use lqos_bus::{bus_request, BusRequest, BusResponse};
use lqos_config::{Tunables, Config};
use rocket::{fs::NamedFile, serde::{json::Json, Serialize}};

// Note that NoCache can be replaced with a cache option
// once the design work is complete.
#[get("/config")]
pub async fn config_page<'a>(_auth: AuthGuard) -> NoCache<Option<NamedFile>> {
  NoCache::new(NamedFile::open("static/config.html").await.ok())
}

#[get("/api/list_nics")]
pub async fn get_nic_list<'a>(
  _auth: AuthGuard,
) -> NoCache<Json<Vec<(String, String, String)>>> {
  let result = get_interfaces()
    .iter()
    .map(|eth| {
      let mac = if let Some(mac) = &eth.mac_addr {
        mac.to_string()
      } else {
        String::new()
      };
      (eth.name.clone(), format!("{:?}", eth.if_type), mac)
    })
    .collect();

  NoCache::new(Json(result))
}

#[get("/api/config")]
pub async fn get_current_lqosd_config(
  _auth: AuthGuard,
) -> NoCache<Json<Config>> {
  let config = lqos_config::load_config().unwrap();
  println!("{config:#?}");
  NoCache::new(Json(config))
}

#[post("/api/lqos_tuning/<period>", data = "<tuning>")]
pub async fn update_lqos_tuning(
  auth: AuthGuard,
  period: u64,
  tuning: Json<Tunables>,
) -> Json<String> {
  if auth != AuthGuard::Admin {
    return Json("Error: Not authorized".to_string());
  }

  // Send the update to the server
  bus_request(vec![BusRequest::UpdateLqosDTuning(period, (*tuning).clone())])
    .await
    .unwrap();

  // For now, ignore the reply.

  Json("OK".to_string())
}

#[derive(Serialize, Clone, Default)]
#[serde(crate = "rocket::serde")]
pub struct LqosStats {
  pub bus_requests_since_start: u64,
  pub time_to_poll_hosts_us: u64,
  pub high_watermark: (u64, u64),
  pub tracked_flows: u64,
  pub rtt_events_per_second: u64,
}

#[get("/api/stats")]
pub async fn stats() -> NoCache<Json<LqosStats>> {
  for msg in bus_request(vec![BusRequest::GetLqosStats]).await.unwrap() {
    if let BusResponse::LqosdStats { bus_requests, time_to_poll_hosts, high_watermark, tracked_flows, rtt_events_per_second } = msg {
      return NoCache::new(Json(LqosStats {
        bus_requests_since_start: bus_requests,
        time_to_poll_hosts_us: time_to_poll_hosts,
        high_watermark,
        tracked_flows,
        rtt_events_per_second,
      }));
    }
  }
  NoCache::new(Json(LqosStats::default()))
}