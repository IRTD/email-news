use email_news::configuration::Settings;
use email_news::email::EmailClient;
use email_news::startup::run;
use email_news::telemetry::*;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("email-news".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let config = Settings::get().expect("Failed to load Settings");
    let db_conn = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(config.database.with_db());

    let addr = config.addr();
    let listener = TcpListener::bind(addr)?;
    let sender_email = config
        .email_client
        .sender()
        .expect("Failed to parse Email - Invalid Email");

    let email_client = EmailClient::new(
        sender_email,
        config.email_client.base_url,
        config.email_client.auth_token,
    );

    run(listener, db_conn, email_client)?.await?;

    Ok(())
}
