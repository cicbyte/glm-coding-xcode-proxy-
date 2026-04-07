use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

/// 通用异步重试函数
///
/// # 参数
/// - `operation`: 要执行的异步操作
/// - `max_retries`: 最大重试次数
/// - `base_delay_ms`: 基础延迟（毫秒），实际延迟为 base_delay * attempt
///
/// # 返回
/// 成功时返回操作结果，失败时返回最后一次错误
pub async fn with_retry<T, E, F, Fut>(
    operation: F,
    max_retries: u32,
    base_delay_ms: u64,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut last_error: Option<E> = None;

    for attempt in 1..=max_retries {
        info!("🔄 第{}次尝试", attempt);

        match operation().await {
            Ok(result) => {
                return Ok(result);
            }
            Err(e) => {
                warn!("❌ 第{}次尝试失败: {}", attempt, e);
                last_error = Some(e);

                if attempt < max_retries {
                    let delay = base_delay_ms * attempt as u64;
                    info!("⏳ {}ms后重试...", delay);
                    sleep(Duration::from_millis(delay)).await;
                }
            }
        }
    }

    warn!("❌ 所有{}次重试都失败了", max_retries);
    Err(last_error.expect("At least one error should exist after failed retries"))
}
