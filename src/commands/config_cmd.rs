use crate::cli::ConfigCommands;
use crate::config::{get_all_env_values, get_env_value, set_env_value, unset_env_value};
use std::process::exit;

/// 敏感键名，在 list 时需要脱敏
const SENSITIVE_KEYS: &[&str] = &["KEY", "API_KEY", "SECRET", "TOKEN", "PASSWORD", "AUTH"];

/// 已知配置项及其说明
const KNOWN_KEYS: &[(&str, &str)] = &[
    ("KEY", "智谱 API Key（必需）"),
    ("HOST", "监听地址（默认 127.0.0.1）"),
    ("PORT", "监听端口（默认 8890）"),
    ("MAX_RETRIES", "最大重试次数（默认 3）"),
    ("RETRY_DELAY", "重试延迟 ms（默认 1000）"),
    ("REQUEST_TIMEOUT", "请求超时 ms（默认 60000）"),
];

fn is_sensitive(key: &str) -> bool {
    SENSITIVE_KEYS.iter().any(|k| key.to_uppercase().contains(k))
}

fn mask_value(value: &str) -> String {
    if value.len() <= 4 {
        return "*".repeat(value.len());
    }
    let visible = 4;
    format!("{}{}", &value[..visible], "*".repeat(value.len() - visible))
}

pub fn run(cmd: &ConfigCommands) {
    match cmd {
        ConfigCommands::List => {
            let values = get_all_env_values();
            let max_key_len = KNOWN_KEYS.iter().map(|(k, _)| k.len()).max().unwrap_or(20);

            // 显示已知配置项（带说明）
            for (key, desc) in KNOWN_KEYS {
                let value = values.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str());
                let display_value = match value {
                    Some(v) if is_sensitive(key) => mask_value(v),
                    Some(v) => v.to_string(),
                    None => "(未设置)".to_string(),
                };
                println!("{:width$} = {}  # {}", key, display_value, desc, width = max_key_len);
            }

            // 显示不在已知列表中的自定义配置项
            for (key, value) in &values {
                if !KNOWN_KEYS.iter().any(|(k, _)| k == key) {
                    if is_sensitive(key) {
                        println!("{:width$} = {}", key, mask_value(value), width = max_key_len);
                    } else {
                        println!("{:width$} = {}", key, value, width = max_key_len);
                    }
                }
            }
        }
        ConfigCommands::Get { key } => {
            match get_env_value(key) {
                Some(value) => {
                    if is_sensitive(key) {
                        println!("{}", mask_value(&value));
                    } else {
                        println!("{}", value);
                    }
                }
                None => {
                    eprintln!("配置项 '{}' 不存在", key);
                    exit(1);
                }
            }
        }
        ConfigCommands::Set { key, value } => {
            if let Err(e) = set_env_value(key, value) {
                eprintln!("设置失败: {}", e);
                exit(1);
            }
            println!("已设置 {}={}", key, if is_sensitive(key) { mask_value(value) } else { value.to_string() });
        }
        ConfigCommands::Unset { key } => {
            if let Err(e) = unset_env_value(key) {
                eprintln!("删除失败: {}", e);
                exit(1);
            }
            println!("已删除 {}", key);
        }
    }
}
