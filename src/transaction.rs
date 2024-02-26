use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response, Result},
    Json,
};
use chrono::Utc;

use crate::{AppState, Client, IncomingTransaction, TrasactionType};

pub async fn transaction(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(payload): Json<IncomingTransaction>,
) -> Result<Response> {
    let client = sqlx::query_as::<_, Client>("select * from client where id = $1")
        .bind(id as i32)
        .fetch_one(&state.pool)
        .await
        .map_err(|err| {
            eprint!("{}", err);

            (StatusCode::NOT_FOUND, "").into_response()
        })?;

    match payload.r#type {
        TrasactionType::Credit => {
            sqlx::query("update client set balance = $1 where id = $2")
                .bind(client.balance + payload.value)
                .bind(client.id)
                .execute(&state.pool)
                .await
                .map_err(|err| {
                    eprint!("{}", err);

                    (StatusCode::NOT_FOUND, "").into_response()
                })?;
        }
        TrasactionType::Debit => {
            if client.limit - (client.balance + payload.value) < 0 {
                return Ok((StatusCode::UNPROCESSABLE_ENTITY, "").into_response());
            }

            sqlx::query("update client set balance = $1 where id = $2")
                .bind(client.balance - payload.value)
                .bind(client.id)
                .execute(&state.pool)
                .await
                .map_err(|err| {
                    eprint!("{}", err);

                    (StatusCode::NOT_FOUND, "").into_response()
                })?;
        }
    }
    sqlx::query(
        "insert into transaction (client_id, value, type, description) values ($1, $2, $3, $4)",
    )
    .bind(client.id)
    .bind(payload.value)
    .bind(payload.r#type)
    .bind(payload.description)
    .execute(&state.pool)
    .await
    .map_err(|err| {
        eprint!("{}", err);

        (StatusCode::NOT_FOUND, "").into_response()
    })?;

    Ok((StatusCode::OK, "").into_response())
}
