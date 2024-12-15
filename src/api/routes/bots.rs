use crate::api::errors::ApiError;
use crate::api::AppState;
use crate::db::{Bot, Build, DBError};
use crate::domain::BotId;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    extract::State,
    routing::{delete, get, patch, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/bots", get(fetch_bots))
        .route("/bots", post(create_bot))
        .route("/bots/:id/builds", get(fetch_bot_builds))
        .route("/bots/:id", patch(patch_bot))
        .route("/bots/:id", delete(delete_bot))
}

#[derive(Serialize, Deserialize, Validate)]
struct CreateBotRequest {
    #[validate(length(min = 1, max = 32))]
    pub name: String,
    #[validate(length(max = 100_000))]
    pub source_code: String,
    #[validate(length(min = 1, max = 32))]
    pub language: String,
}

#[derive(Serialize, Deserialize, Validate)]
struct PatchBotRequest {
    #[validate(length(min = 1, max = 32))]
    pub name: String,
}

#[derive(Deserialize)]
struct FetchBotsParams {
    name: Option<String>,
}

#[derive(Serialize)]
struct BotResponse {
    id: i32,
    name: String,
    source_code: String,
    language: String,
    created_at: DateTime<Utc>,
}

impl From<Bot> for BotResponse {
    fn from(bot: Bot) -> Self {
        Self {
            id: bot.id,
            name: bot.name,
            source_code: bot.source_code,
            language: bot.language,
            created_at: bot.created_at,
        }
    }
}

#[derive(Serialize)]
struct BuildResponse {
    worker_name: String,
    status_code: Option<i32>,
    stdout: Option<String>,
    stderr: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<Build> for BuildResponse {
    fn from(build: Build) -> Self {
        Self {
            worker_name: build.worker_name,
            status_code: build.status_code,
            stdout: build.stdout,
            stderr: build.stderr,
            created_at: build.created_at,
        }
    }
}

async fn fetch_bots(
    State(app_state): State<AppState>,
    Query(params): Query<FetchBotsParams>,
) -> Result<impl IntoResponse, ApiError> {
    let mut bots = app_state.db.fetch_bots().await?;

    if let Some(name) = params.name {
        bots.retain(|b| b.name == name);
    }

    let bots: Vec<BotResponse> = bots.into_iter().map(BotResponse::from).collect();

    Ok(Json(bots))
}

async fn create_bot(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateBotRequest>,
) -> Result<impl IntoResponse, ApiError> {
    payload.validate()?;

    app_state
        .db
        .create_bot(&payload.name, &payload.source_code, &payload.language)
        .await?;

    Ok(StatusCode::CREATED)
}

async fn patch_bot(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<PatchBotRequest>,
) -> Result<impl IntoResponse, ApiError> {
    payload.validate()?;

    app_state.db.rename_bot(id, payload.name).await?;

    Ok(StatusCode::OK)
}

async fn delete_bot(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    app_state.db.remove_bot(id).await?;
    Ok(StatusCode::OK)
}

async fn fetch_bot_builds(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    let builds = app_state.db.fetch_builds(BotId(id)).await;
    let builds: Vec<BuildResponse> = builds.into_iter().map(BuildResponse::from).collect();
    Ok(Json(builds))
}

impl From<DBError> for ApiError {
    fn from(value: DBError) -> Self {
        match value {
            DBError::AlreadyExists => ApiError::AlreadyExists,
            DBError::NotFound => ApiError::NotFound,
            DBError::Unexpected(e) => ApiError::Internal(e),
        }
    }
}