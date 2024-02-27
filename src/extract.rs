use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response, Result},
    Json,
};
use chrono::Utc;
use serde_json::json;

use crate::{AppState, Client, Transaction};

pub async fn extract(Path(id): Path<u32>, State(state): State<AppState>) -> Result<Response> {
    let client = sqlx::query_as::<_, Client>("select * from client where id = $1")
        .bind(id as i32)
        .fetch_one(&state.pool)
        .await
        .map_err(|err| {
            eprint!("{}", err);

            (StatusCode::NOT_FOUND, "").into_response()
        })?;

    let transactions = sqlx::query_as::<_, Transaction>(
        "select * from transaction where client_id = $1 order by created_at desc limit 10",
    )
    .bind(client.id)
    .fetch_all(&state.pool)
    .await
    .map_err(|err| {
        eprint!("{}", err);

        (StatusCode::NOT_FOUND, "").into_response()
    })?;

    Ok((
        StatusCode::OK,
        Json(json!(
            {
                "saldo": {
                    "total": client.balance,
                    "data_extrato": Utc::now(),
                    "limite": client.limit
                },
                "ultimas_transacoes": transactions
            }
        )),
    )
        .into_response())
}
