use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};

#[cfg(feature = "official-api")]
use tomato_novel_official_api::SearchClient;

use crate::ui::web::state::AppState;

#[derive(Debug, Deserialize)]
pub(crate) struct SearchQuery {
    pub(crate) q: String,
}

pub(crate) async fn api_search(
    State(_state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    #[cfg(not(feature = "official-api"))]
    {
        let keyword = q.q.trim();
        if keyword.is_empty() {
            return Ok(Json(json!({"items": []})));
        }
        Ok(Json(json!({
            "items": [],
            "error": "search requires `official-api` feature",
        })))
    }

    #[cfg(feature = "official-api")]
    {
        let keyword = q.q.trim().to_string();
        if keyword.is_empty() {
            return Ok(Json(json!({"items": []})));
        }

        // 并发限制：最多 2 个同时进行的上游 API 请求。
        let _permit =
            _state.api_semaphore.acquire().await.map_err(|_| {
                api_error(StatusCode::SERVICE_UNAVAILABLE, "Лимит параллельных запросов к API закрыт")
            })?;

        let resp = tokio::task::spawn_blocking(move || {
            let client = SearchClient::new()?;
            client.search_books(&keyword)
        })
        .await
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "Ошибка выполнения поиска"))?
        .map_err(|err| api_error(StatusCode::BAD_GATEWAY, format!("Ошибка поиска: {err}")))?;

        let items: Vec<Value> = resp
            .books
            .into_iter()
            .map(|b| {
                json!({
                    "book_id": b.book_id,
                    "title": b.title,
                    "author": b.author,
                    "raw": b.raw,
                })
            })
            .collect();

        Ok(Json(json!({"items": items})))
    }
}

#[cfg(feature = "official-api")]
fn api_error(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "error": message.into() })))
}
