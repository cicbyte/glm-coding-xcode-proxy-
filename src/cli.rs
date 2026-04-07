use clap::{Parser, Subcommand};

/// Xcode AI 代理服务 CLI
#[derive(Parser, Debug)]
#[command(name = "glm-coding-xcode-proxy")]
#[command(about = "Xcode AI 代理服务 - 智谱 GLM Coding Plan")]
pub struct Cli {
    /// 服务监听端口
    #[arg(short, long, default_value = "8890", global = true)]
    pub port: u16,

    /// 服务监听地址
    #[arg(long, default_value = "127.0.0.1", global = true)]
    pub host: String,

    /// 子命令
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 管理配置项
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
    /// 管理 launchd 服务
    Service {
        #[command(subcommand)]
        action: ServiceCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// 查看所有配置（API Key 脱敏）
    List,
    /// 获取指定配置值
    Get {
        /// 配置键名
        key: String,
    },
    /// 设置配置值
    Set {
        /// 配置键名
        key: String,
        /// 配置值
        value: String,
    },
    /// 删除配置项
    Unset {
        /// 配置键名
        key: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ServiceCommands {
    /// 生成 plist 并 launchctl load
    Install,
    /// launchctl unload 并删除 plist
    Uninstall,
    /// launchctl load
    Start,
    /// launchctl unload
    Stop,
    /// 显示运行状态、PID、日志路径
    Status,
}
