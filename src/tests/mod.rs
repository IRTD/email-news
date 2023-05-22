#![allow(unused_imports)]
#![allow(dead_code)]

mod health_check;
mod subscription;

use crate::configuration::{DatabaseSettings, Settings};
use crate::telemetry::*;

use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let sub_name = "test".to_string();
    let default_filter_level = "debug".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let sub = get_subscriber(sub_name, default_filter_level, std::io::stdout);
        init_subscriber(sub);
    } else {
        let sub = get_subscriber(sub_name, default_filter_level, std::io::sink);

        init_subscriber(sub);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// Spin up an instance of our App for tests to run on.
// This will bind to a random port and return the Address it binded to (i.e. 127.0.0.1:xxxx)
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut config = Settings::get().expect("Failed to load Settings");
    config.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_database(&config.database).await;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");

    let port = listener.local_addr().unwrap().port();
    let server = crate::startup::run(listener, db_pool.clone()).expect("Failed to bind to address");

    let _ = tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut db_conn = PgConnection::connect(&config.no_db_connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    db_conn
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create Database");

    let pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to connect Pool to Postgres");

    sqlx::migrate!("./docker-instances/migrations")
        .run(&pool)
        .await
        .expect("Failed to execute Migration");

    pool
}
