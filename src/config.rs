use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

/// 智谱 API 基础 URL
pub const API_BASE_URL: &str = "https://open.bigmodel.cn/api/coding/paas/v4";

/// 应用配置
#[derive(Clone, Debug)]
pub struct Config {
    pub zhipu_api_key: String,
    #[allow(dead_code)]
    pub host: String,
    #[allow(dead_code)]
    pub port: u16,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub request_timeout_ms: u64,
}

impl Config {
    /// 从配置文件直接加载，配置文件为唯一事实来源
    pub fn from_config() -> Self {
        let cfg = read_config_file();

        let zhipu_api_key = cfg
            .get("KEY")
            .expect("缺少配置项 KEY，请运行 config set KEY <your-key>")
            .clone();

        Self {
            zhipu_api_key,
            host: cfg.get("HOST").cloned().unwrap_or_else(|| "127.0.0.1".to_string()),
            port: cfg
                .get("PORT")
                .and_then(|p| p.parse().ok())
                .unwrap_or(8890),
            max_retries: cfg
                .get("MAX_RETRIES")
                .and_then(|r| r.parse().ok())
                .unwrap_or(3),
            retry_delay_ms: cfg
                .get("RETRY_DELAY")
                .and_then(|d| d.parse().ok())
                .unwrap_or(1000),
            request_timeout_ms: cfg
                .get("REQUEST_TIMEOUT")
                .and_then(|t| t.parse().ok())
                .unwrap_or(60000),
        }
    }
}

/// 解析配置文件为键值对
fn read_config_file() -> HashMap<String, String> {
    let path = env_file_path();
    let mut map = HashMap::new();
    if !path.exists() {
        return map;
    }
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return map,
    };
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    map
}

// ===== 配置文件读写 =====

/// 配置文件中每一行的表示，保留注释和空行
enum EnvLine {
    KeyValue { key: String, value: String },
    Comment(String),
    Blank,
}

/// 配置目录: ~/.glm-coding-xcode-proxy/
pub fn config_dir() -> PathBuf {
    let home = env::var("HOME").expect("无法获取 HOME 目录");
    PathBuf::from(home).join(".glm-coding-xcode-proxy")
}

/// 配置文件路径: ~/.glm-coding-xcode-proxy/config
fn env_file_path() -> PathBuf {
    config_dir().join("config")
}

fn parse_env_lines(content: &str) -> Vec<EnvLine> {
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                EnvLine::Blank
            } else if trimmed.starts_with('#') {
                EnvLine::Comment(line.to_string())
            } else if let Some((key, value)) = trimmed.split_once('=') {
                EnvLine::KeyValue {
                    key: key.trim().to_string(),
                    value: value.trim().to_string(),
                }
            } else {
                EnvLine::Comment(line.to_string())
            }
        })
        .collect()
}

fn serialize_env_lines(lines: &[EnvLine]) -> String {
    lines
        .iter()
        .map(|line| match line {
            EnvLine::KeyValue { key, value } => format!("{}={}", key, value),
            EnvLine::Comment(s) => s.clone(),
            EnvLine::Blank => String::new(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// 获取指定配置值
pub fn get_env_value(key: &str) -> Option<String> {
    let path = env_file_path();
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    let lines = parse_env_lines(&content);
    for line in &lines {
        if let EnvLine::KeyValue { key: k, value } = line {
            if k == key {
                return Some(value.clone());
            }
        }
    }
    None
}

/// 获取所有配置项
pub fn get_all_env_values() -> Vec<(String, String)> {
    let path = env_file_path();
    if !path.exists() {
        return Vec::new();
    }
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let lines = parse_env_lines(&content);
    lines
        .iter()
        .filter_map(|line| {
            if let EnvLine::KeyValue { key, value } = line {
                Some((key.clone(), value.clone()))
            } else {
                None
            }
        })
        .collect()
}

/// 设置配置值，写入配置文件
pub fn set_env_value(key: &str, value: &str) -> Result<(), String> {
    let path = env_file_path();
    // 确保配置目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    let content = if path.exists() {
        fs::read_to_string(&path).unwrap_or_default()
    } else {
        String::new()
    };

    let mut lines = parse_env_lines(&content);
    let mut found = false;

    for line in &mut lines {
        if let EnvLine::KeyValue { key: k, value: v } = line {
            if k == key {
                *v = value.to_string();
                found = true;
                break;
            }
        }
    }

    if !found {
        // 如果文件不为空且最后一行不是空行，添加空行分隔
        if !lines.is_empty() {
            if let Some(EnvLine::KeyValue { .. }) = lines.last() {
                lines.push(EnvLine::Blank);
            }
        }
        lines.push(EnvLine::KeyValue {
            key: key.to_string(),
            value: value.to_string(),
        });
    }

    let new_content = serialize_env_lines(&lines);
    // 确保文件以换行结尾
    let new_content = if new_content.ends_with('\n') {
        new_content
    } else {
        format!("{}\n", new_content)
    };

    fs::write(&path, new_content).map_err(|e| format!("写入配置文件失败: {}", e))
}

/// 删除配置项
pub fn unset_env_value(key: &str) -> Result<(), String> {
    let path = env_file_path();
    if !path.exists() {
        return Err("配置项不存在".to_string());
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("读取配置文件失败: {}", e))?;
    let mut lines = parse_env_lines(&content);
    let original_len = lines.len();

    lines.retain(|line| {
        if let EnvLine::KeyValue { key: k, .. } = line {
            k != key
        } else {
            true
        }
    });

    if lines.len() == original_len {
        return Err(format!("配置项 '{}' 不存在", key));
    }

    let new_content = serialize_env_lines(&lines);
    let new_content = if new_content.ends_with('\n') {
        new_content
    } else {
        format!("{}\n", new_content)
    };

    fs::write(&path, new_content).map_err(|e| format!("写入配置文件失败: {}", e))
}
