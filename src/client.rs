use crate::config::{API_BASE_URL, Config};
use crate::error::ApiError;
use crate::models::ChatCompletionRequest;
use crate::retry::with_retry;
use axum::{
    body::Body,
    http::{header, Response},
};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tracing::info;

/// 智谱 API 客户端
#[derive(Clone)]
pub struct ZhipuClient {
    client: Client,
    config: Config,
}

impl ZhipuClient {
    /// 创建新的客户端实例
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.request_timeout_ms))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// 获取模型列表（转发智谱 API）
    pub async fn list_models(&self) -> Result<Value, ApiError> {
        let url = format!("{}/models", API_BASE_URL);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.zhipu_api_key))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApiError::UpstreamError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let json = response.json().await?;
        Ok(json)
    }

    /// 非流式聊天完成请求
    pub async fn chat_completion(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<Value, ApiError> {
        let result = with_retry(
            || self.make_request(request),
            self.config.max_retries,
            self.config.retry_delay_ms,
        )
        .await?;

        info!("📦 返回非流式响应");
        Ok(result)
    }

    /// 流式聊天完成请求
    pub async fn chat_completion_stream(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<Response<Body>, ApiError> {
        let response = with_retry(
            || self.make_stream_request(request),
            self.config.max_retries,
            self.config.retry_delay_ms,
        )
        .await?;

        info!("🔄 返回流式响应");
        Ok(response)
    }

    /// 执行非流式请求
    async fn make_request(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<Value, ApiError> {
        let url = format!("{}/chat/completions", API_BASE_URL);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.zhipu_api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await?;

        let status = response.status();
        info!("✅ GLM API 响应状态: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApiError::UpstreamError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let json = response.json().await?;
        Ok(json)
    }

    /// 执行流式请求
    async fn make_stream_request(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<Response<Body>, ApiError> {
        let url = format!("{}/chat/completions", API_BASE_URL);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.zhipu_api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await?;

        let status = response.status();
        info!("✅ GLM API 响应状态: {}", status);

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApiError::UpstreamError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        // 构建流式响应
        let stream = response.bytes_stream();
        let body = Body::from_stream(stream);

        let response = Response::builder()
            .header(header::CONTENT_TYPE, "text/event-stream")
            .header(header::CACHE_CONTROL, "no-cache")
            .header(header::CONNECTION, "keep-alive")
            .body(body)?;

        Ok(response)
    }
}
