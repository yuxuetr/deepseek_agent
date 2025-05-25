use dotenv::dotenv;
use reqwest;
use serde_json::{Value, json};
use std::env;
use tracing::{Level, info};
use tracing_subscriber;

mod tools;
use tools::{get_tools_definition, handle_tool_call};

async fn chat(user_query: &str) -> Result<(), Box<dyn std::error::Error>> {
  let api_key = env::var("DEEPSEEK_API_KEY").unwrap();
  let endpoint = env::var("DEEPSEEK_API_URL").unwrap().to_string();

  // 构建请求体
  let body = json!({
      "model": env::var("MODEL_NAME").unwrap(),
      "messages": [
          {
              "role": "system",
              "content": "你是一个专业的助手，可以：\n1. 提供天气信息和穿衣建议\n2. 搜索互联网获取实时信息\n请根据用户的问题，选择合适的工具来提供帮助。"
          },
          {"role": "user", "content": user_query}
      ],
      "tools": get_tools_definition(),
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
    "LLM工具的响应结果: {}",
    serde_json::to_string_pretty(&response).unwrap()
  );

  // 处理工具调用
  if let Some(tool_calls) = response["choices"][0]["message"]["tool_calls"].as_array() {
    for call in tool_calls {
      let tool_result = handle_tool_call(call).await?;

      // 将结果反馈给大模型
      let final_response = reqwest::Client::new()
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": env::var("MODEL_NAME").unwrap(),
            "messages": [
                {"role": "system", "content": tool_result.system_prompt},
                {"role": "assistant", "content": null, "tool_calls": [call]},
                {
                    "role": "tool",
                    "content": tool_result.content,
                    "tool_call_id": call["id"]
                }
            ]
        }))
        .send()
        .await?
        .json::<Value>()
        .await?;

      info!(
        "最终回答: {}",
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

  // 测试查询
  let user_query = "2025年2月的编程语言排行榜是什么";
  info!("用户查询: {}", user_query);
  chat(user_query).await?;

  Ok(())
}
