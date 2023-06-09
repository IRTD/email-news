use email_news::configuration::Settings;
use email_news::startup::run;
use email_news::telemetry::*;
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("email-news".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let config = Settings::get().expect("Failed to load Settings");
    let db_conn = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(&config.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres");

    let addr = config.addr();
    let listener = TcpListener::bind(addr)?;

    run(listener, db_conn)?.await?;

    Ok(())
}
