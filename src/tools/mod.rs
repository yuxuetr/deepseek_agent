pub mod amap;
pub mod serper;

use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
use std::future::Future;
use std::pin::Pin;

use std::sync::OnceLock;

// 工具处理结果
pub struct ToolResult {
  pub content: String,
  pub system_prompt: String,
}

// 工具处理函数类型
type ToolHandler = Box<
  dyn Fn(
      &Value,
    ) -> Pin<
      Box<dyn Future<Output = Result<ToolResult, Box<dyn std::error::Error>>> + Send + Sync>,
    > + Send
    + Sync,
>;

// 工具注册表
struct ToolRegistry {
  tools: HashMap<String, ToolHandler>,
}

impl ToolRegistry {
  fn new() -> Self {
    let mut tools = HashMap::new();
    let weather_handler: ToolHandler = Box::new(|call: &Value| {
      let call = call.clone();
      Box::pin(async move {
        let args: Value = serde_json::from_str(call["function"]["arguments"].as_str().unwrap())?;
        let location = args["location"].as_str().unwrap();

        let amap_key = env::var("AMAP_API_KEY").unwrap();
        let weather_info = amap::get_weather(location, &amap_key).await?;

        Ok(ToolResult {
          content: serde_json::to_string(&weather_info)?,
          system_prompt: "你是一个专业的天气顾问，请根据获取到的天气数据给出详细的穿衣建议。注意：\n1. 分析温度范围和温差\n2. 考虑天气现象（晴、阴、雨等）\n3. 考虑风力大小\n4. 给出具体的穿衣层次建议\n5. 如有必要，提醒是否需要携带雨具或防晒用品".to_string(),
        })
      })
    });
    tools.insert("get_weather".to_string(), weather_handler);

    let search_handler: ToolHandler = Box::new(|call: &Value| {
      let call = call.clone();
      Box::pin(async move {
        let args: Value = serde_json::from_str(call["function"]["arguments"].as_str().unwrap())?;
        let query = args["query"].as_str().unwrap();

        let search_results = serper::search(query).await?;
        let formatted_results = serper::format_results(&search_results, 3);

        Ok(ToolResult {
          content: formatted_results,
          system_prompt: "你是一个专业的信息分析师，请根据搜索结果给出准确、简洁的回答。"
            .to_string(),
        })
      })
    });
    tools.insert("search".to_string(), search_handler);
    Self { tools }
  }

  fn get_handler(&self, name: &str) -> Option<&ToolHandler> {
    self.tools.get(name)
  }
}

// 全局工具注册表实例
fn get_registry() -> &'static ToolRegistry {
  static REGISTRY: OnceLock<ToolRegistry> = OnceLock::new();
  REGISTRY.get_or_init(ToolRegistry::new)
}

// 工具定义
pub fn get_tools_definition() -> Value {
  json!([
      {
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
      }
  ])
}

// 处理工具调用
pub async fn handle_tool_call(call: &Value) -> Result<ToolResult, Box<dyn std::error::Error>> {
  let tool_name = call["function"]["name"].as_str().ok_or("无效的工具名称")?;
  let registry = get_registry();

  match registry.get_handler(tool_name) {
    Some(handler) => handler(call).await,
    None => Err(format!("未知的工具调用: {}", tool_name).into()),
  }
}
