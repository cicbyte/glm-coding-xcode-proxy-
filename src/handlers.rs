use crate::client::ZhipuClient;
use crate::error::ApiError;
use crate::models::ChatCompletionRequest;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use tracing::info;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub client: ZhipuClient,
}

/// 模型列表处理器（转发智谱 API）
pub async fn list_models(
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    info!("📋 转发模型列表请求");
    let result = state.client.list_models().await?;
    Ok((StatusCode::OK, Json(result)).into_response())
}

/// 聊天完成处理器
pub async fn chat_completions(
    State(state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Response, ApiError> {
    info!("🎯 请求模型: {}", request.model);
    info!("🔍 是否流式: {}", request.stream);

    if request.stream {
        state.client.chat_completion_stream(&request).await
    } else {
        let result = state.client.chat_completion(&request).await?;
        Ok((StatusCode::OK, Json(result)).into_response())
    }
}
