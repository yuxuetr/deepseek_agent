use dotenv::dotenv;
use reqwest;
use serde_json::{Value, json};
use std::env;
use tracing::{Level, info};
use tracing_subscriber;

mod tools;
mod mcp_tools;
mod mcp_client;

use mcp_client::McpClient;

async fn chat_with_mcp_server(user_query: &str) -> Result<(), Box<dyn std::error::Error>> {
  let api_key = env::var("DEEPSEEK_API_KEY").unwrap();
  let endpoint = env::var("DEEPSEEK_API_URL").unwrap().to_string();

  // Initialize MCP Client and connect to server
  let mut mcp_client = McpClient::new("mcp_server").await?;
  info!("MCP Client: Connected to MCP server");

  // Initialize MCP connection
  let _server_info = mcp_client.initialize().await?;
  info!("MCP Client: Server initialized successfully");

  // Get available tools from MCP server
  let tools = mcp_client.list_tools().await?;
  info!("MCP Client: Available tools: {}", tools.len());

  // 构建请求体
  let body = json!({
      "model": env::var("MODEL_NAME").unwrap(),
      "messages": [
          {
              "role": "system",
              "content": "你是一个专业的助手，可以：\n1. 提供天气信息和穿衣建议\n2. 搜索互联网获取实时信息\n请根据用户的问题，选择合适的工具来提供帮助。\n\n这是一个基于Model Context Protocol (MCP)的工具系统。"
          },
          {"role": "user", "content": user_query}
      ],
      "tools": tools.iter().map(|tool| {
        json!({
          "type": "function",
          "function": {
            "name": tool["name"],
            "description": tool["description"],
            "parameters": tool["inputSchema"]
          }
        })
      }).collect::<Vec<_>>(),
      "tool_choice": "auto",
  });

  // 发送初次请求
  let response = reqwest::Client::new()
    .post(&endpoint)
    .header("Authorization", format!("Bearer {}", api_key))
    .json(&body)
    .send()
    .await?
    .json::<Value>()
    .await?;
  info!(
    "MCP LLM响应结果: {}",
    serde_json::to_string_pretty(&response).unwrap()
  );

  // 处理MCP工具调用
  if let Some(tool_calls) = response["choices"][0]["message"]["tool_calls"].as_array() {
    for call in tool_calls {
      let tool_name = call["function"]["name"].as_str().unwrap();
      let arguments: Value = serde_json::from_str(call["function"]["arguments"].as_str().unwrap())?;
      
      // Use MCP server tool execution
      let tool_result = mcp_client.call_tool(tool_name, arguments).await?;

      // Extract result from MCP response
      let content = tool_result["content"][0]["text"].as_str().unwrap_or("No content");
      let system_prompt = "请根据工具返回的结果给出准确、有帮助的回答。".to_string();

      let final_response = reqwest::Client::new()
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": env::var("MODEL_NAME").unwrap(),
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "assistant", "content": null, "tool_calls": [call]},
                {
                    "role": "tool",
                    "content": content,
                    "tool_call_id": call["id"]
                }
            ]
        }))
        .send()
        .await?
        .json::<Value>()
        .await?;

      info!(
        "MCP最终回答: {}",
        final_response["choices"][0]["message"]["content"]
      );
    }
  }

  Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // 初始化日志系统
  tracing_subscriber::fmt().with_max_level(Level::INFO).init();

  dotenv().ok();
  info!("环境变量加载完成");

  // 测试MCP服务器搜索工具
  let search_query = "什么是MCP协议";
  info!("MCP Server搜索查询: {}", search_query);
  chat_with_mcp_server(search_query).await?;

  Ok(())
}
