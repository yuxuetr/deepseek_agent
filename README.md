# DeepSeek Agents

这是一个基于 Rust 语言开发的智能助手演示项目，展示了如何使用 DeepSeek 大语言模型构建 AI Agent，并结合外部工具实现检索增强生成（RAG）功能。

## 🌟 特性

- 🤖 基于 DeepSeek 大语言模型
- 🛠 支持多种外部工具集成
  - 🌤 高德天气 API（天气查询和穿衣建议）
  - 🔍 Google Serper API（实时搜索信息）
- 📚 检索增强生成（RAG）架构
- 🦀 Rust 异步编程实践
- 🎯 工具函数动态调用

## 🔧 技术栈

- **编程语言**: Rust
- **异步运行时**: Tokio
- **HTTP 客户端**: Reqwest
- **序列化**: Serde
- **日志系统**: Tracing
- **环境变量**: Dotenv

## 🚀 快速开始

### 前置要求

- Rust 工具链
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
   DEEPSEEK_API_KEY=your_deepseek_api_key
   DEEPSEEK_API_URL=your_deepseek_api_endpoint
   AMAP_API_KEY=your_amap_api_key
   SERPER_API_KEY=your_serper_api_key
   ```

3. 运行项目

   ```bash
   cargo run
   ```

## 📖 使用示例

### 天气查询

```rust
let query = "今天上海天气怎么样？请给出穿衣建议。";
chat(query).await?;
```

### 搜索信息

```rust
let query = "最近的世界新闻有哪些？";
chat(query).await?;
```

## 🏗 项目结构

```shell
src/
├── main.rs          # 主程序入口
├── tools/           # 工具模块
│   ├── mod.rs      # 模块声明
│   ├── amap.rs     # 高德天气 API 工具
│   └── serper.rs   # Google 搜索 API 工具
└── ...
```

## 🔍 核心功能

### Agent 系统

- 智能工具选择
- 上下文感知
- 动态响应生成

### RAG 实现

- 外部知识获取
- 信息整合与分析
- 结构化输出

## 📝 待办事项

- [ ] 添加更多外部工具支持
- [ ] 实现对话历史记忆
- [ ] 优化错误处理机制
- [ ] 添加单元测试
- [ ] 支持更多搜索引擎
- [ ] 添加 API 文档

## 已支持的工具列表

- [X] 高德天气查询
- [X] Google Serper的谷歌搜索

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License