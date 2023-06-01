use crate::domain::SubscriberEmail;
use crate::email::EmailClient;
use crate::routes::*;

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(
    listener: TcpListener,
    db_conn: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(db_conn);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/subscriptions", web::post().to(subscribe))
            .route("/health", web::get().to(health_check))
            .app_data(connection.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
