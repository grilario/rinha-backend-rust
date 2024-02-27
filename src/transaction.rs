use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response, Result},
    Json,
};
use serde::Serialize;

use crate::{AppState, Client, IncomingTransaction, TrasactionType};

#[derive(Debug, Serialize, sqlx::FromRow)]
struct OutputTransaction {
    #[serde(rename = "limite")]
    max_limit: i32,
    #[serde(rename = "saldo")]
    balance: i32,
}

pub async fn transaction(
    Path(id): Path<u32>,
    State(state): State<AppState>,
    Json(payload): Json<IncomingTransaction>,
) -> Result<Response> {
    if payload.description.len() > 10 || payload.description.is_empty() {
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, "").into_response());
    }

    let client = sqlx::query_as::<_, Client>("select * from client where id = $1")
        .bind(id as i32)
        .fetch_one(&state.pool)
        .await
        .map_err(|err| {
            eprint!("{}", err);

            (StatusCode::NOT_FOUND, "").into_response()
        })?;

    let value = match payload.r#type {
        TrasactionType::Credit => payload.value,
        TrasactionType::Debit => -payload.value,
    };

    let Some(result) = sqlx::query_as::<_, OutputTransaction>(
        r#"
update client set balance = balance + $1
where id = $2 and $1 + max_limit + balance >= 0
returning max_limit, balance
    "#,
    )
    .bind(value)
    .bind(client.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|err| {
        eprintln!("Error: {} \nRequest: {:?}", err, payload);

        (StatusCode::BAD_REQUEST, "").into_response()
    })?
    else {
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, "").into_response());
    };

    sqlx::query(
        "insert into transaction (client_id, value, type, description) values ($1, $2, $3, $4)",
    )
    .bind(client.id)
    .bind(payload.value)
    .bind(&payload.r#type)
    .bind(&payload.description)
    .execute(&state.pool)
    .await
    .map_err(|err| {
        eprintln!("Error: {} \nRequest: {:?}", err, payload);

        (StatusCode::BAD_REQUEST, "").into_response()
    })?;

    Ok((StatusCode::OK, Json(result)).into_response())
}
