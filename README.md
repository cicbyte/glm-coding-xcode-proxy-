# GLM Coding Xcode Proxy

Xcode AI 代理服务，将 Claude API 请求转发到智谱 GLM Coding Plan，解决 Xcode 直接连接智谱官方地址时因响应格式不兼容导致的对话失败问题。

## 背景

Xcode 支持配置外部模型提供者使用 AI 辅助编程。在 Xcode Settings 中可以添加智谱 Bigmodel 作为 Internet Hosted 模型提供者：

!\[Xcode Settings 入口]\(images/001-setting.png null)

配置智谱官方地址 `https://open.bigmodel.cn/api/anthropic`：

!\[配置智谱官方地址]\(images/002-bigmodel.png null)

但直接使用官方地址发送对话请求时，会因响应中缺少必要字段导致解码失败：

!\[直接使用官方地址报错]\(images/003-error.png null)

> `The data couldn't be read because it is missing.`
> `Unable to decode as APIErrorResponse.`

**原因**：智谱官方 API 的响应格式与 Xcode 期望的 Claude API 格式存在差异，部分必要字段缺失。

## 解决方案

本项目作为本地代理，在 Xcode 和智谱 API 之间进行协议转换，补齐缺失字段，使 Xcode 能正常对话。

1. 启动代理服务监听本地端口（默认 `127.0.0.1:8890`）
2. 在 Xcode 中添加 **Locally Hosted** 模型提供者，端口填写 `8890`

!\[配置本地代理]\(images/004-proxy.png null)

配置完成后即可正常使用 AI 对话：

!\[通过代理正常对话]\(images/005-proxy-success.png null)

## 快速开始

### 1. 编译

```bash
cargo build --release
```

### 2. 配置 API Key

```bash
glm_coding_xcode_proxy config set KEY your-api-key-here
```

也可以直接编辑配置文件 `~/.glm-coding-xcode-proxy/config`：

```bash
mkdir -p ~/.glm-coding-xcode-proxy
echo 'KEY=your-api-key-here' > ~/.glm-coding-xcode-proxy/config
```

或者直接运行程序，未检测到 API Key 时会交互式引导输入。

### 3. 运行

```bash
# 直接运行（默认监听 127.0.0.1:8890）
glm_coding_xcode_proxy

# 指定端口
glm_coding_xcode_proxy --port 9090

# 指定监听地址和端口
glm_coding_xcode_proxy --host 0.0.0.0 --port 9090
```

## 命令用法

### 全局选项

```bash
Usage: glm_coding_xcode_proxy [OPTIONS] [COMMAND]

Commands:
  config   管理配置项
  service  管理 launchd 服务
  help     Print this message or the help of the given subcommand(s)

Options:
  -p, --port <PORT>  服务监听端口 [default: 8890]
      --host <HOST>  服务监听地址 [default: 127.0.0.1]
  -h, --help         Print help
```

不带子命令直接运行时启动代理服务。

### config — 管理配置项

```bash
Usage: glm_coding_xcode_proxy config <COMMAND>

Commands:
  list   查看所有配置（API Key 脱敏）
  get    获取指定配置值
  set    设置配置值
  unset  删除配置项
```

**示例：**

```bash
# 设置 API Key（必需）
glm_coding_xcode_proxy config set KEY your-api-key-here

# 设置监听端口
glm_coding_xcode_proxy config set PORT 9090

# 设置最大重试次数
glm_coding_xcode_proxy config set MAX_RETRIES 5

# 查看所有配置
glm_coding_xcode_proxy config list
# 输出：
# KEY             = abcd****1234  # 智谱 API Key（必需）
# HOST            = (未设置)      # 监听地址（默认 127.0.0.1）
# PORT            = (未设置)      # 监听端口（默认 8890）
# MAX_RETRIES     = (未设置)      # 最大重试次数（默认 3）
# RETRY_DELAY     = (未设置)      # 重试延迟 ms（默认 1000）
# REQUEST_TIMEOUT = (未设置)      # 请求超时 ms（默认 60000）

# 获取指定配置值
glm_coding_xcode_proxy config get KEY

# 删除配置项
glm_coding_xcode_proxy config unset MAX_RETRIES
```

**可配置项：**

| 键                 | 说明             | 默认值         |
| ----------------- | -------------- | ----------- |
| `KEY`             | 智谱 API Key（必需） | 无           |
| `HOST`            | 监听地址           | `127.0.0.1` |
| `PORT`            | 监听端口           | `8890`      |
| `MAX_RETRIES`     | 最大重试次数         | `3`         |
| `RETRY_DELAY`     | 重试延迟（毫秒）       | `1000`      |
| `REQUEST_TIMEOUT` | 请求超时（毫秒）       | `60000`     |

### service — 管理 launchd 服务

```bash
Usage: glm_coding_xcode_proxy service <COMMAND>

Commands:
  install    生成 plist 并 launchctl load
  uninstall  launchctl unload 并删除 plist
  start      launchctl load
  stop       launchctl unload
  status     显示运行状态、PID、日志路径
```

**示例：**

```bash
# 安装并启动服务
glm_coding_xcode_proxy service install

# 查看状态
glm_coding_xcode_proxy service status

# 停止服务
glm_coding_xcode_proxy service stop

# 重新启动
glm_coding_xcode_proxy service stop
glm_coding_xcode_proxy service start

# 卸载服务（停止并删除 plist）
glm_coding_xcode_proxy service uninstall
```

`service install` 会自动生成 plist 文件到 `~/Library/LaunchAgents/`，日志输出到 `~/Library/Logs/glm-coding-xcode-proxy/`，并配置 `KeepAlive` 保活。

## API 接口

### 获取模型列表

```bash
curl http://localhost:8890/v1/models
```

### Chat Completions

```bash
curl http://localhost:8890/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "glm-5",
    "messages": [{"role": "user", "content": "你好"}],
    "stream": false
  }'
```

## 注意事项

1. **API Key 安全**：配置文件存储在 `~/.glm-coding-xcode-proxy/config`，注意保护权限。
2. **修改配置后需重启服务**：`glm_coding_xcode_proxy service stop && glm_coding_xcode_proxy service start`
3. **端口冲突**：通过 `--port` 参数或 `config set PORT <端口>` 修改。
4. **服务保活**：plist 中配置了 `KeepAlive`，服务异常退出后会自动重启。

## 项目结构

```
src/
├── main.rs             # 入口文件
├── cli.rs              # CLI 子命令定义
├── config.rs           # 配置加载与读写
├── client.rs           # 智谱 API 客户端
├── handlers.rs         # HTTP 请求处理
├── models.rs           # 数据模型
├── error.rs            # 错误处理
├── retry.rs            # 重试逻辑
└── commands/
    ├── mod.rs           # 子命令模块入口
    ├── config_cmd.rs    # config 子命令实现
    └── service_cmd.rs   # service 子命令实现
```

