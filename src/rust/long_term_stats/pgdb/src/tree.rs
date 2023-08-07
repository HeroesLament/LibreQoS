use crate::license::StatsHostError;
use sqlx::{FromRow, Pool, Postgres, Row};
use itertools::Itertools;

#[derive(Debug, FromRow)]
pub struct TreeNode {
    pub site_name: String,
    pub index: i32,
    pub parent: i32,
    pub site_type: String,
    pub max_down: i32,
    pub max_up: i32,
    pub current_down: i32,
    pub current_up: i32,
    pub current_rtt: i32,
}

pub async fn get_site_tree(
    cnn: &Pool<Postgres>,
    key: &str,
    host_id: &str,
) -> Result<Vec<TreeNode>, StatsHostError> {
    sqlx::query_as::<_, TreeNode>("SELECT site_name, index, parent, site_type, max_down, max_up, current_down, current_up, current_rtt FROM site_tree WHERE key = $1 AND host_id=$2")
        .bind(key)
        .bind(host_id)
        .fetch_all(cnn)
        .await
        .map_err(|e| StatsHostError::DatabaseError(e.to_string()))
}

pub async fn get_site_info(
    cnn: &Pool<Postgres>,
    key: &str,
    site_name: &str,
) -> Result<TreeNode, StatsHostError> {
    sqlx::query_as::<_, TreeNode>("SELECT site_name, index, parent, site_type, max_down, max_up, current_down, current_up, current_rtt FROM site_tree WHERE key = $1 AND site_name=$2")
        .bind(key)
        .bind(site_name)
        .fetch_one(cnn)
        .await
        .map_err(|e| StatsHostError::DatabaseError(e.to_string()))
}

pub async fn get_site_id_from_name(
    cnn: &Pool<Postgres>,
    key: &str,
    site_name: &str,
) -> Result<i32, StatsHostError> {
    if site_name == "root" {
        return Ok(0);
    }
    let site_id_db = sqlx::query("SELECT index FROM site_tree WHERE key = $1 AND site_name=$2")
        .bind(key)
        .bind(site_name)
        .fetch_one(cnn)
        .await
        .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
    let site_id: i32 = site_id_db
        .try_get("index")
        .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
    Ok(site_id)
}

pub async fn get_parent_list(
    cnn: &Pool<Postgres>,
    key: &str,
    site_name: &str,
) -> Result<Vec<(String, String)>, StatsHostError> {
    let mut result = Vec::new();

    // Get the site index
    let site_id_db = sqlx::query("SELECT index FROM site_tree WHERE key = $1 AND site_name=$2")
        .bind(key)
        .bind(site_name)
        .fetch_one(cnn)
        .await
        .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
    let mut site_id: i32 = site_id_db
        .try_get("index")
        .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;

    // Get the parent list
    while site_id != 0 {
        let parent_db = sqlx::query(
            "SELECT site_name, parent, site_type FROM site_tree WHERE key = $1 AND index=$2",
        )
        .bind(key)
        .bind(site_id)
        .fetch_one(cnn)
        .await
        .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
        let parent: String = parent_db
            .try_get("site_name")
            .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
        let site_type: String = parent_db
            .try_get("site_type")
            .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
        site_id = parent_db
            .try_get("parent")
            .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
        result.push((site_type, parent));
    }

    Ok(result)
}

pub async fn get_child_list(
    cnn: &Pool<Postgres>,
    key: &str,
    site_name: &str,
) -> Result<Vec<(String, String, String)>, StatsHostError> {
    let mut result = Vec::new();

    // Get the site index
    let site_id_db = sqlx::query("SELECT index FROM site_tree WHERE key = $1 AND site_name=$2")
        .bind(key)
        .bind(site_name)
        .fetch_one(cnn)
        .await
        .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
    let site_id: i32 = site_id_db
        .try_get("index")
        .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;

    // Add child sites
    let child_sites = sqlx::query(
        "SELECT DISTINCT site_name, parent, site_type FROM site_tree WHERE key=$1 AND parent=$2",
    )
    .bind(key)
    .bind(site_id)
    .fetch_all(cnn)
    .await
    .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;

    for child in child_sites {
        let child_name: String = child
            .try_get("site_name")
            .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
        let child_type: String = child
            .try_get("site_type")
            .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
        result.push((child_type, child_name.clone(), child_name));
    }

    // Add child shaper nodes
    let child_circuits = sqlx::query(
        "SELECT circuit_id, circuit_name FROM shaped_devices WHERE key=$1 AND parent_node=$2",
    )
    .bind(key)
    .bind(site_name)
    .fetch_all(cnn)
    .await
    .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;

    for child in child_circuits {
        let child_name: String = child
            .try_get("circuit_name")
            .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
        let child_id: String = child
            .try_get("circuit_id")
            .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
        result.push(("circuit".to_string(), child_id, child_name));
    }

    result.sort_by(|a, b| a.2.cmp(&b.2));
    let result = result.into_iter().dedup_by(|a,b| a.2 == b.2).collect();    

    Ok(result)
}

pub async fn get_circuit_parent_list(
    cnn: &Pool<Postgres>,
    key: &str,
    circuit_id: &str,
) -> Result<Vec<(String, String)>, StatsHostError> {
    let mut result = Vec::new();

    // Get the site name to start at
    let site_name: String =
        sqlx::query("SELECT parent_node FROM shaped_devices WHERE key = $1 AND circuit_id= $2")
            .bind(key)
            .bind(circuit_id)
            .fetch_one(cnn)
            .await
            .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?
            .get(0);

    // Get the site index
    let site_id_db = sqlx::query("SELECT index FROM site_tree WHERE key = $1 AND site_name=$2")
        .bind(key)
        .bind(site_name)
        .fetch_one(cnn)
        .await
        .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
    let mut site_id: i32 = site_id_db
        .try_get("index")
        .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;

    // Get the parent list
    while site_id != 0 {
        let parent_db = sqlx::query(
            "SELECT site_name, parent, site_type FROM site_tree WHERE key = $1 AND index=$2",
        )
        .bind(key)
        .bind(site_id)
        .fetch_one(cnn)
        .await
        .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
        let parent: String = parent_db
            .try_get("site_name")
            .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
        let site_type: String = parent_db
            .try_get("site_type")
            .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
        site_id = parent_db
            .try_get("parent")
            .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;
        result.push((site_type, parent));
    }

    Ok(result)
}

#[derive(Debug, FromRow)]
pub struct SiteOversubscription {
    pub dlmax: i64,
    pub dlmin: i64,
    pub devicecount: i64,
}

pub async fn get_oversubscription(cnn: &Pool<Postgres>, key: &str, site_name: &str) -> Result<SiteOversubscription, StatsHostError> {
    let site_id = get_site_id_from_name(cnn, key, site_name).await
        .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;

    const SQL: &str = "WITH RECURSIVE children
    (index, site_name, level, parent) AS (
        SELECT index, site_name, 0, parent FROM site_tree WHERE key=$1 and index = $2
        UNION ALL
        SELECT
            st.index,
            st.site_name,
            children.level + 1,
            children.parent
            FROM site_tree st, children
            WHERE children.index = st.parent AND children.level < 5 AND key=$3
    ),
    devices (circuit_id, download_max_mbps, download_min_mbps) AS (
        SELECT DISTINCT
        circuit_id,
        download_max_mbps,
        download_min_mbps
    FROM shaped_devices WHERE key=$4
    AND parent_node IN (SELECT site_name FROM children)
    AND circuit_name NOT LIKE '%(site)'
    )
    
    SELECT 
        SUM(download_max_mbps) AS dlmax,
            SUM(download_min_mbps) AS dlmin,
            COUNT(circuit_id) AS devicecount
    FROM devices;";

    let rows = sqlx::query_as::<_, SiteOversubscription>(SQL)
        .bind(key)
        .bind(site_id)
        .bind(key)
        .bind(key)
        .fetch_one(cnn)
        .await
        .map_err(|e| StatsHostError::DatabaseError(e.to_string()))?;

    Ok(rows)
}