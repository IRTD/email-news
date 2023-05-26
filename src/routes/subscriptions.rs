use crate::domain::Subscriber;

use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding new Subscriber",
    skip(form, db_conn),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subscribe(form: web::Form<FormData>, db_conn: web::Data<PgPool>) -> impl Responder {
    let sub = match Subscriber::new(form.name.clone(), form.email.clone()) {
        Ok(n) => n,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(db_conn.get_ref(), &sub).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(name = "Saving new Subscriber details in Database", skip(sub, db_conn))]
async fn insert_subscriber(db_conn: &PgPool, sub: &Subscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        sub.email(),
        sub.name(),
        Utc::now(),
    )
    .execute(db_conn)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
