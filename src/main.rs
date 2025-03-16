use dotenv::dotenv;
use reqwest;
use serde_json::{Value, json};
use std::env;
use tracing::{Level, info};
use tracing_subscriber;

mod tools;
use tools::{amap, serper};

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
      "tools": [{
          "type": "function",
          "function": {
              "name": "get_weather",
              "description": "获取指定城市的天气预报信息",
              "parameters": {
                  "type": "object",
                  "properties": {
                      "location": {
                          "type": "string",
                          "description": "城市名称，例如：上海"
                      }
                  },
                  "required": ["location"]
              }
          }
      },
      {
          "type": "function",
          "function": {
              "name": "search",
              "description": "使用Google搜索获取实时信息",
              "parameters": {
                  "type": "object",
                  "properties": {
                      "query": {
                          "type": "string",
                          "description": "搜索查询词"
                      }
                  },
                  "required": ["query"]
              }
          }
      }],
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
      match call["function"]["name"].as_str() {
        Some("get_weather") => {
          // 解析参数
          let args: Value = serde_json::from_str(call["function"]["arguments"].as_str().unwrap())?;
          info!(
            "工具调用参数: {}",
            serde_json::to_string_pretty(&args).unwrap()
          );
          let location = args["location"].as_str().unwrap();
          info!("被查询的城市: {}", location);

          // 实际调用天气API
          let amap_key = env::var("AMAP_API_KEY").unwrap();
          let weather_info = amap::get_weather(location, &amap_key).await?;
          info!(
            "天气信息: {}",
            serde_json::to_string_pretty(&weather_info).unwrap()
          );

          // 将结果反馈给大模型
          let final_response = reqwest::Client::new()
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&json!({
                "model": env::var("MODEL_NAME").unwrap(),
                "messages": [
                    {"role": "system", "content": "你是一个专业的天气顾问，请根据获取到的天气数据给出详细的穿衣建议。注意：\n1. 分析温度范围和温差\n2. 考虑天气现象（晴、阴、雨等）\n3. 考虑风力大小\n4. 给出具体的穿衣层次建议\n5. 如有必要，提醒是否需要携带雨具或防晒用品"},
                    {"role": "assistant", "content": null, "tool_calls": [call]},
                    {
                        "role": "tool",
                        "content": serde_json::to_string(&weather_info)?,
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
        Some("search") => {
          // 解析参数
          let args: Value = serde_json::from_str(call["function"]["arguments"].as_str().unwrap())?;
          let query = args["query"].as_str().unwrap();
          info!("搜索查询: {}", query);

          // 调用搜索API
          let search_results = serper::search(query).await?;
          let formatted_results = serper::format_results(&search_results, 3); // 只取前3条结果
          info!("搜索结果: {}", formatted_results);

          // 将结果反馈给大模型
          let final_response = reqwest::Client::new()
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&json!({
                "model": env::var("MODEL_NAME").unwrap(),
                "messages": [
                    {"role": "system", "content": "你是一个专业的信息分析师，请根据搜索结果给出准确、简洁的回答。"},
                    {"role": "assistant", "content": null, "tool_calls": [call]},
                    {
                        "role": "tool",
                        "content": formatted_results,
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
        _ => {}
      }
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
