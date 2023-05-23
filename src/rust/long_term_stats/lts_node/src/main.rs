mod web;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start the logger
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn"),
    );

    // Get the database connection pool
    let pool = pgdb::get_connection_pool(5).await;
    if pool.is_err() {
        log::error!("Unable to connect to the database");
        log::error!("{pool:?}");
        return Err(anyhow::Error::msg("Unable to connect to the database"));
    }
    let pool = pool.unwrap();

    // Start the webserver
    log::info!("Starting the webserver");
    let _ = tokio::spawn(web::webserver(pool)).await;

    Ok(())
}
