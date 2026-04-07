use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

/// API 错误类型
#[derive(Debug)]
pub enum ApiError {
    /// 上游 API 错误
    UpstreamError(String),
    /// 网络错误
    NetworkError(String),
    /// 内部错误
    InternalError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::UpstreamError(msg) => write!(f, "API 请求失败: {}", msg),
            ApiError::NetworkError(msg) => write!(f, "网络请求失败: {}", msg),
            ApiError::InternalError(msg) => write!(f, "内部错误: {}", msg),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            ApiError::UpstreamError(msg) => (
                StatusCode::BAD_GATEWAY,
                "api_error",
                format!("API 请求失败: {}", msg),
            ),
            ApiError::NetworkError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "network_error",
                format!("网络请求失败: {}", msg),
            ),
            ApiError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                msg,
            ),
        };

        let body = json!({
            "error": {
                "message": message,
                "type": error_type,
            }
        });

        (status, Json(body)).into_response()
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ApiError::NetworkError(format!("请求超时: {}", err))
        } else if err.is_connect() {
            ApiError::NetworkError(format!("连接失败: {}", err))
        } else if err.is_status() {
            let status_str = err
                .status()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            ApiError::UpstreamError(format!("HTTP {}: {}", status_str, err))
        } else {
            ApiError::NetworkError(err.to_string())
        }
    }
}

impl From<axum::http::Error> for ApiError {
    fn from(err: axum::http::Error) -> Self {
        ApiError::InternalError(err.to_string())
    }
}
