use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use actix_web::web::Form;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::types::uuid;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let subscriber_email = SubscriberEmail::parse(form.email)?;
        let subscriber_name = SubscriberName::parse(form.name)?;
        Ok(NewSubscriber {
            email: subscriber_email,
            name: subscriber_name,
        })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let new_subscriber = match form.into_inner().try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let result = insert_subscriber(&new_subscriber, &pool);
    match result.await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscription in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    new_subscriber: &NewSubscriber,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
