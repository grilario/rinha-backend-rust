use axum::{
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use extract::extract;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::net::TcpListener;
use transaction::transaction;

mod extract;
mod transaction;

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_type")]
#[sqlx(rename_all = "lowercase")]
pub enum TrasactionType {
    #[serde(rename = "c")]
    Credit,
    #[serde(rename = "d")]
    Debit,
}

#[derive(Debug, Deserialize)]
pub struct IncomingTransaction {
    #[serde(rename = "valor")]
    pub value: i32,
    #[serde(rename = "tipo")]
    pub r#type: TrasactionType,
    #[serde(rename = "descricao")]
    pub description: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Transaction {
    #[serde(skip)]
    pub id: i32,
    #[serde(skip)]
    pub client_id: i32,
    #[serde(rename = "valor")]
    pub value: i32,
    #[serde(rename = "tipo")]
    pub r#type: TrasactionType,
    #[serde(rename = "descricao")]
    pub description: String,
    #[serde(rename = "realizada_em")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct Client {
    pub id: i32,
    pub name: String,
    #[sqlx(rename = "max_limit")]
    pub limit: i32,
    pub balance: i32,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: PgPool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgres://api:123@127.0.0.1:5432/app").await?;
    let state = AppState { pool };

    let app = Router::new()
        .route("/clientes/:id/transacoes", post(transaction))
        .route("/clientes/:id/extrato", get(extract))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    println!("Running...");
    axum::serve(listener, app).await?;

    Ok(())
}
