use crate::cli::ServiceCommands;
use crate::config;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{exit, Command};

const SERVICE_NAME: &str = "com.glm-coding-xcode-proxy";

fn plist_path() -> PathBuf {
    let home = env::var("HOME").expect("无法获取 HOME 目录");
    PathBuf::from(home).join("Library/LaunchAgents").join(format!("{}.plist", SERVICE_NAME))
}

fn log_dir() -> PathBuf {
    let home = env::var("HOME").expect("无法获取 HOME 目录");
    PathBuf::from(home).join("Library/Logs/glm-coding-xcode-proxy")
}

fn generate_plist(exe_path: &str, port: u16) -> String {
    let log_path = log_dir();
    format!(
r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{service_name}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe_path}</string>
        <string>--port</string>
        <string>{port}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{log_path}/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>{log_path}/stderr.log</string>
</dict>
</plist>
"#,
        service_name = SERVICE_NAME,
        exe_path = exe_path,
        port = port,
        log_path = log_path.display(),
    )
}

fn launchctl(subcmd: &str, plist: &PathBuf) -> Result<(), String> {
    let status = match subcmd {
        "load" => Command::new("launchctl")
            .args(["load", "-w", plist.to_str().unwrap()])
            .status(),
        "unload" => Command::new("launchctl")
            .args(["unload", "-w", plist.to_str().unwrap()])
            .status(),
        _ => return Err(format!("未知 launchctl 子命令: {}", subcmd)),
    };
    status
        .map_err(|e| format!("执行 launchctl {} 失败: {}", subcmd, e))
        .and_then(|s| {
            if s.success() {
                Ok(())
            } else {
                Err(format!("launchctl {} 失败", subcmd))
            }
        })
}

fn is_loaded() -> bool {
    let output = Command::new("launchctl")
        .args(["list", SERVICE_NAME])
        .output();
    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

fn get_pid() -> Option<String> {
    let output = Command::new("launchctl")
        .args(["list", SERVICE_NAME])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("PID") {
            if let Some(pid) = line.split_whitespace().last() {
                return Some(pid.to_string());
            }
        }
    }
    None
}

pub fn run(cmd: &ServiceCommands, port: u16) {
    match cmd {
        ServiceCommands::Install => {
            let exe_path = env::current_exe()
                .expect("无法获取可执行文件路径");
            let exe_str = exe_path.to_str().expect("路径编码错误");

            let plist = plist_path();
            let content = generate_plist(exe_str, port);

            // 确保目录存在
            if let Some(parent) = plist.parent() {
                fs::create_dir_all(parent)
                    .expect("无法创建 LaunchAgents 目录");
            }

            // 确保日志目录存在
            let log = log_dir();
            fs::create_dir_all(&log).expect("无法创建日志目录");

            fs::write(&plist, content).expect("无法写入 plist 文件");
            println!("已生成 plist: {}", plist.display());

            if let Err(e) = launchctl("load", &plist) {
                eprintln!("{}", e);
                exit(1);
            }
            println!("服务已安装并启动");
            println!("日志目录: {}", log.display());
        }
        ServiceCommands::Uninstall => {
            let plist = plist_path();
            if plist.exists() {
                if is_loaded() {
                    if let Err(e) = launchctl("unload", &plist) {
                        eprintln!("{}", e);
                        exit(1);
                    }
                }
                fs::remove_file(&plist).expect("无法删除 plist 文件");
                println!("服务已卸载");
            } else {
                eprintln!("plist 文件不存在: {}", plist.display());
                exit(1);
            }
        }
        ServiceCommands::Start => {
            let plist = plist_path();
            if !plist.exists() {
                eprintln!("plist 文件不存在，请先运行 service install");
                exit(1);
            }
            if let Err(e) = launchctl("load", &plist) {
                eprintln!("{}", e);
                exit(1);
            }
            println!("服务已启动");
        }
        ServiceCommands::Stop => {
            let plist = plist_path();
            if !plist.exists() {
                eprintln!("plist 文件不存在，请先运行 service install");
                exit(1);
            }
            if let Err(e) = launchctl("unload", &plist) {
                eprintln!("{}", e);
                exit(1);
            }
            println!("服务已停止");
        }
        ServiceCommands::Status => {
            println!("服务名称: {}", SERVICE_NAME);
            println!("配置目录: {}", config::config_dir().display());
            println!("plist 路径: {}", plist_path().display());
            println!("日志路径: {}", log_dir().display());

            if !plist_path().exists() {
                println!("状态: 未安装");
                return;
            }

            if is_loaded() {
                println!("状态: 运行中");
                if let Some(pid) = get_pid() {
                    println!("PID: {}", pid);
                }
            } else {
                println!("状态: 已停止");
            }
        }
    }
}
