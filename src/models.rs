use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenAI 兼容的聊天完成请求
#[derive(Debug, Deserialize, Serialize)]
pub struct ChatCompletionRequest {
    /// 模型 ID
    pub model: String,
    /// 消息列表
    pub messages: Vec<Message>,
    /// 是否流式响应
    #[serde(default)]
    pub stream: bool,
    /// 其他额外字段（透传）
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// 消息结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    /// 角色: system, user, assistant
    pub role: String,
    /// 消息内容
    #[serde(default)]
    pub content: Option<MessageContent>,
}

/// 消息内容（支持字符串或数组格式）
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// 简单文本
    Text(String),
    /// 复杂内容数组
    Array(Vec<ContentPart>),
}

/// 内容部分
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContentPart {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
    #[serde(default)]
    pub image_url: Option<ImageUrl>,
}

/// 图片 URL
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageUrl {
    pub url: String,
}
