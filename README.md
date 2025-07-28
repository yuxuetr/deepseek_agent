# DeepSeek Agent - MCP Server

这是一个基于 Rust 语言开发的智能助手项目，使用 **Model Context Protocol (MCP)** 架构，展示了如何构建符合 MCP 标准的服务器和客户端，并结合 DeepSeek 大语言模型实现智能工具调用。

## 🌟 特性

- 🤖 基于 DeepSeek 大语言模型
- 🔌 **MCP (Model Context Protocol) 架构**
  - JSON-RPC 2.0 通信协议
  - 客户端-服务器分离架构
  - 标准化的工具、资源和提示接口
- 🛠 支持多种外部工具集成
  - 🌤 高德天气 API（天气查询和穿衣建议）
  - 🔍 Google Serper API（实时搜索信息）
- 📚 检索增强生成（RAG）架构
- 🦀 Rust 异步编程实践
- 🎯 MCP 标准工具函数调用

## 🔧 技术栈

- **编程语言**: Rust
- **异步运行时**: Tokio
- **协议**: Model Context Protocol (MCP) with JSON-RPC 2.0
- **MCP SDK**: rmcp, rmcp-macros
- **HTTP 客户端**: Reqwest
- **序列化**: Serde
- **日志系统**: Tracing
- **环境变量**: Dotenv

## 🏗 MCP 架构说明

本项目实现了完整的 MCP 服务器-客户端架构：

```
┌─────────────────┐    JSON-RPC 2.0    ┌─────────────────┐
│   MCP Client    │ ◄──────────────── │   MCP Server    │
│   (main.rs)     │                   │ (mcp_server.rs) │
├─────────────────┤                   ├─────────────────┤
│ - 启动服务器     │                   │ - 工具执行       │
│ - JSON-RPC 通信 │                   │ - 资源管理       │
│ - DeepSeek 集成 │                   │ - 提示模板       │
│ - 结果处理       │                   │ - stdio 通信     │
└─────────────────┘                   └─────────────────┘
```

### MCP 组件

1. **Tools (工具)**: 可执行的功能单元
   - `get_weather`: 天气查询工具
   - `search`: 搜索工具

2. **Resources (资源)**: 可读取的数据资源
   - `weather://recent-queries`: 最近的天气查询记录
   - `search://recent-queries`: 最近的搜索查询记录

3. **Prompts (提示)**: 预定义的提示模板
   - `weather_advisor`: 天气顾问提示模板
   - `search_analyzer`: 搜索结果分析师提示模板

## 🚀 快速开始

### 前置要求

- Rust 工具链 (推荐使用 rustup)
- 必要的 API 密钥:
  - DeepSeek API Key
  - 高德地图 API Key
  - Google Serper API Key

### 安装步骤

1. 克隆项目

   ```bash
   git clone https://github.com/yuxuetr/deepseek_agent.git
   cd deepseek_agent
   ```

2. 配置环境变量

   ```bash
   cp .env.example .env
   ```

   编辑 `.env` 文件，填入你的 API 密钥：

   ```shell
   # DeepSeek API 配置
   DEEPSEEK_API_KEY=your_deepseek_api_key
   DEEPSEEK_API_URL=https://api.deepseek.com/v1/chat/completions
   MODEL_NAME=deepseek-chat

   # 工具 API 配置
   AMAP_API_KEY=your_amap_api_key
   SERPER_API_KEY=your_serper_api_key
   ```

3. 构建项目

   ```bash
   cargo build --release
   ```

4. 运行应用

   ```bash
   # 运行主客户端应用
   cargo run --bin main

   # 或直接运行
   cargo run
   ```

## 📖 使用示例

### 基本用法

主程序会自动启动 MCP 服务器并建立连接：

```rust
// 自动启动 MCP 服务器
let mut mcp_client = McpClient::new("mcp_server").await?;

// 初始化 MCP 连接
let _server_info = mcp_client.initialize().await?;

// 获取可用工具
let tools = mcp_client.list_tools().await?;
```

### 天气查询示例

```bash
# 修改 main.rs 中的查询内容
let search_query = "今天上海天气怎么样？请给出穿衣建议。";
```

### 搜索信息示例

```bash
# 修改 main.rs 中的查询内容
let search_query = "什么是MCP协议？它有什么优势？";
```

### 手动测试 MCP 服务器

你也可以单独运行 MCP 服务器进行测试：

```bash
# 启动 MCP 服务器
cargo run --bin mcp_server

# 然后发送 JSON-RPC 请求到 stdin
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}
```

## 🔧 MCP API 参考

### 工具 (Tools)

#### 列出工具

```json
{ "jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {} }
```

#### 调用工具

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "get_weather",
    "arguments": { "location": "上海" }
  }
}
```

### 资源 (Resources)

#### 列出资源

```json
{ "jsonrpc": "2.0", "id": 3, "method": "resources/list", "params": {} }
```

#### 读取资源

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "resources/read",
  "params": { "uri": "weather://recent-queries" }
}
```

### 提示 (Prompts)

#### 列出提示

```json
{ "jsonrpc": "2.0", "id": 5, "method": "prompts/list", "params": {} }
```

#### 获取提示

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "prompts/get",
  "params": {
    "name": "weather_advisor",
    "arguments": { "weather_data": "天气数据JSON" }
  }
}
```

## 🏗 项目结构

```shell
src/
├── main.rs                    # 主程序入口 (MCP Client)
├── lib.rs                     # 库模块声明
├── mcp_client.rs              # MCP 客户端实现
├── mcp_server_simple.rs       # MCP 服务器实现
├── mcp_tools.rs               # MCP 工具包装器 (未来扩展)
├── bin/
│   └── mcp_server_simple.rs   # MCP 服务器二进制入口
└── tools/                     # 工具模块
    ├── mod.rs                 # 模块声明
    ├── amap.rs                # 高德天气 API 工具
    └── serper.rs              # Google 搜索 API 工具
```

## 🔍 核心功能

### MCP 服务器功能

- **协议初始化**: 支持 MCP 2024-11-05 协议版本
- **工具管理**: 动态工具注册和调用
- **资源管理**: 结构化数据资源访问
- **提示管理**: 预定义提示模板系统
- **错误处理**: 完整的 JSON-RPC 错误响应

### Agent 系统

- **智能工具选择**: 基于 DeepSeek 模型的工具调用决策
- **上下文感知**: MCP 协议标准化的上下文传递
- **动态响应生成**: 结合工具结果的智能回答生成

### RAG 实现

- **外部知识获取**: 通过 MCP 工具接口获取实时信息
- **信息整合与分析**: DeepSeek 模型处理和分析工具返回结果
- **结构化输出**: MCP 标准化的结果格式

## 🧪 开发和测试

### 运行测试

```bash
cargo test
```

### 开发模式

```bash
# 启用详细日志
RUST_LOG=debug cargo run

# 运行特定二进制文件
cargo run --bin mcp_server
cargo run --bin main
```

### 调试 MCP 通信

启用 tracing 日志来观察 MCP 通信过程：

```bash
RUST_LOG=info cargo run
```

你会看到类似的日志输出：

```
INFO main::mcp_client: MCP Client: Connected to MCP server
INFO main: MCP Client: Server initialized successfully
INFO main: MCP Client: Available tools: 2
INFO deepseek_agent::mcp_server_simple: MCP Server: Handling method: initialize
INFO deepseek_agent::mcp_server_simple: MCP Server: Handling method: tools/list
```

## 📝 配置文件

### 环境变量说明

| 变量名             | 描述                 | 示例值                                         |
| ------------------ | -------------------- | ---------------------------------------------- |
| `DEEPSEEK_API_KEY` | DeepSeek API 密钥    | `sk-xxx`                                       |
| `DEEPSEEK_API_URL` | DeepSeek API 端点    | `https://api.deepseek.com/v1/chat/completions` |
| `MODEL_NAME`       | 使用的模型名称       | `deepseek-chat`                                |
| `AMAP_API_KEY`     | 高德地图 API 密钥    | `your_amap_key`                                |
| `SERPER_API_KEY`   | Serper 搜索 API 密钥 | `your_serper_key`                              |

### Cargo.toml 配置

项目配置了双二进制结构：

```toml
[[bin]]
name = "main"
path = "src/main.rs"

[[bin]]
name = "mcp_server"
path = "src/bin/mcp_server_simple.rs"
```

## 🆕 更新日志

### v0.3.0 - MCP 架构重构

- ✅ 实施完整的 MCP (Model Context Protocol) 架构
- ✅ JSON-RPC 2.0 协议支持
- ✅ 客户端-服务器分离设计
- ✅ 标准化工具、资源、提示接口
- ✅ rmcp SDK 集成
- ✅ stdio 传输支持

### v0.2.0 - 基础功能

- ✅ DeepSeek API 集成
- ✅ 高德天气 API 工具
- ✅ Google Serper 搜索工具
- ✅ 基础工具调用系统

## 🔮 待办事项

- [ ] 添加更多 MCP 工具支持
- [ ] 实现工具调用缓存机制
- [ ] 添加 MCP 服务器配置管理
- [ ] 支持 WebSocket 传输
- [ ] 实现对话历史记忆
- [ ] 优化错误处理机制
- [ ] 添加单元测试和集成测试
- [ ] 支持更多搜索引擎
- [ ] 添加 OpenAPI 文档
- [ ] MCP 工具热插拔支持

## 已支持的 MCP 功能

### 工具 (Tools)

- [x] 高德天气查询 (`get_weather`)
- [x] Google Serper 搜索 (`search`)

### 资源 (Resources)

- [x] 天气查询历史 (`weather://recent-queries`)
- [x] 搜索查询历史 (`search://recent-queries`)

### 提示 (Prompts)

- [x] 天气顾问模板 (`weather_advisor`)
- [x] 搜索分析师模板 (`search_analyzer`)

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

### 开发指南

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启 Pull Request

## 📚 相关链接

- [Model Context Protocol 规范](https://modelcontextprotocol.io/)
- [MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [大模型工具调用指南](https://yuxuetr.com/blog/2025/05/25/llm-func-tools)
- [DeepSeek API 文档](https://platform.deepseek.com/docs)

## 📄 许可证

MIT License

---

**注意**: 这是一个 MCP 协议的演示项目，展示了如何在 Rust 中实现符合标准的 MCP 服务器和客户端。适合用于学习 MCP 协议和构建自己的 AI 工具集成系统。
