mod cli;
mod client;
mod commands;
mod config;
mod error;
mod handlers;
mod models;
mod retry;

use crate::config::Config;
use crate::handlers::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    let cli = cli::Cli::parse();

    match &cli.command {
        Some(cli::Commands::Config { action }) => {
            commands::config_cmd::run(action);
        }
        Some(cli::Commands::Service { action }) => {
            commands::service_cmd::run(action, cli.port);
        }
        None => {
            run_server(cli.host.clone(), cli.port);
        }
    }
}

fn run_server(host: String, port: u16) {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 检查 KEY 是否已配置，未配置则交互式引导输入
    if config::get_env_value("KEY").is_none() {
        println!("未检测到 API Key 配置。");
        println!("请输入智谱 API Key（可在 https://open.bigmodel.cn 获取）:");
        let mut key = String::new();
        std::io::stdin().read_line(&mut key).expect("读取输入失败");
        let key = key.trim().to_string();
        if key.is_empty() {
            eprintln!("错误: API Key 不能为空，请重新运行程序并输入有效的 Key");
            std::process::exit(1);
        }
        // 保存到配置文件
        config::set_env_value("KEY", &key).unwrap_or_else(|e| {
            eprintln!("保存配置失败: {}", e);
            std::process::exit(1);
        });
        info!("✅ API Key 已保存到配置文件");
    }

    // 加载配置
    let config = Config::from_config();

    // 打印启动信息
    info!("🚀 Xcode AI 代理服务已启动");
    info!("📡 监听地址: http://{}:{}", host, port);

    info!("⚙️ 重试配置:");
    info!("   最大重试次数: {}", config.max_retries);
    info!("   重试延迟: {}ms (递增)", config.retry_delay_ms);
    info!("   请求超时: {}ms", config.request_timeout_ms);

    info!("📋 配置 Claude Code:");
    info!("   ANTHROPIC_BASE_URL: http://localhost:{}", port);
    info!("   ANTHROPIC_AUTH_TOKEN: any-string-works");
    info!("🔧 功能: 智谱 GLM Coding Plan 代理，流式响应，智能重试");

    // 创建 tokio runtime
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async move {
        // 创建客户端
        let client = crate::client::ZhipuClient::new(config);

        // 创建应用状态
        let state = AppState { client };

        // 配置 CORS
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        // 构建路由
        let app = Router::new()
            .route("/v1/models", get(handlers::list_models))
            .route("/v1/chat/completions", post(handlers::chat_completions))
            .layer(cors)
            .with_state(state);

        // 启动服务器
        let addr: SocketAddr = format!("{}:{}", host, port)
            .parse()
            .expect("Invalid address");

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Failed to bind address");

        info!("🌐 服务器正在监听 {}", addr);

        axum::serve(listener, app)
            .await
            .expect("Failed to start server");
    });
}
