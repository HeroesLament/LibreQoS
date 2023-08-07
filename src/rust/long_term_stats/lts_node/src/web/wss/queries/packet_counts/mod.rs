//! Packet-per-second data queries
mod packet_row;
use self::packet_row::PacketRow;
use super::time_period::InfluxTimePeriod;
use crate::web::wss::{influx_query_builder::InfluxQueryBuilder, send_response};
use axum::extract::ws::WebSocket;
use pgdb::sqlx::{Pool, Postgres};
use tracing::instrument;
use wasm_pipe_types::{PacketHost, Packets};

fn add_by_direction(direction: &str, down: &mut Vec<Packets>, up: &mut Vec<Packets>, row: &PacketRow) {
    match direction {
        "down" => {
            down.push(Packets {
                value: row.avg,
                date: row.time.format("%Y-%m-%d %H:%M:%S").to_string(),
                l: row.min,
                u: row.max - row.min,
            });
        }
        "up" => {
            up.push(Packets {
                value: row.avg,
                date: row.time.format("%Y-%m-%d %H:%M:%S").to_string(),
                l: row.min,
                u: row.max - row.min,
            });
        }
        _ => {}
    }
}

#[instrument(skip(cnn, socket, key, period))]
pub async fn send_packets_for_all_nodes(
    cnn: &Pool<Postgres>,
    socket: &mut WebSocket,
    key: &str,
    period: InfluxTimePeriod,
) -> anyhow::Result<()> {
    let node_status = pgdb::node_status(cnn, key).await?;
    let mut nodes = Vec::<PacketHost>::new();
    InfluxQueryBuilder::new(period.clone())
        .with_measurement("packets")
        .with_fields(&["min", "max", "avg"])
        .with_groups(&["host_id", "min", "max", "avg", "direction", "_field"])
        .execute::<PacketRow>(cnn, key)
        .await?
        .into_iter()
        .for_each(|row| {
            if let Some(node) = nodes.iter_mut().find(|n| n.node_id == row.host_id) {
                add_by_direction(&row.direction, &mut node.down, &mut node.up, &row);
            } else {
                let mut down = Vec::new();
                let mut up = Vec::new();

                add_by_direction(&row.direction, &mut down, &mut up, &row);

                let node_name = if let Some(node) = node_status.iter().find(|n| n.node_id == row.host_id) {
                    node.node_name.clone()
                } else {
                    row.host_id.clone()
                };

                nodes.push(PacketHost {
                    node_id: row.host_id,
                    node_name,
                    down,
                    up,
                });
            }
        });
    send_response(socket, wasm_pipe_types::WasmResponse::PacketChart { nodes }).await;
    Ok(())
}

#[instrument(skip(cnn, socket, key, period))]
pub async fn send_packets_for_node(
    cnn: &Pool<Postgres>,
    socket: &mut WebSocket,
    key: &str,
    period: InfluxTimePeriod,
    node_id: &str,
    node_name: &str,
) -> anyhow::Result<()> {
    let node =
        get_packets_for_node(cnn, key, node_id.to_string(), node_name.to_string(), period).await?;

    send_response(
        socket,
        wasm_pipe_types::WasmResponse::PacketChart { nodes: vec![node] },
    )
    .await;
    Ok(())
}

/// Requests packet-per-second data for a single shaper node.
///
/// # Arguments
/// * `cnn` - A connection pool to the database
/// * `key` - The organization's license key
/// * `node_id` - The ID of the node to query
/// * `node_name` - The name of the node to query
pub async fn get_packets_for_node(
    cnn: &Pool<Postgres>,
    key: &str,
    node_id: String,
    node_name: String,
    period: InfluxTimePeriod,
) -> anyhow::Result<PacketHost> {
    let rows = InfluxQueryBuilder::new(period.clone())
        .with_measurement("packets")
        .with_host_id(&node_id)
        .with_field("min")
        .with_field("max")
        .with_field("avg")
        .execute::<PacketRow>(cnn, key)
        .await;

    match rows {
        Err(e) => {
            tracing::error!("Error querying InfluxDB (packets by node): {}", e);
            Err(anyhow::Error::msg("Unable to query influx"))
        }
        Ok(rows) => {
            // Parse and send the data
            //println!("{rows:?}");

            let mut down = Vec::new();
            let mut up = Vec::new();

            // Fill download
            for row in rows.iter().filter(|r| r.direction == "down") {
                down.push(Packets {
                    value: row.avg,
                    date: row.time.format("%Y-%m-%d %H:%M:%S").to_string(),
                    l: row.min,
                    u: row.max - row.min,
                });
            }

            // Fill upload
            for row in rows.iter().filter(|r| r.direction == "up") {
                up.push(Packets {
                    value: row.avg,
                    date: row.time.format("%Y-%m-%d %H:%M:%S").to_string(),
                    l: row.min,
                    u: row.max - row.min,
                });
            }

            Ok(PacketHost {
                node_id,
                node_name,
                down,
                up,
            })
        }
    }
}