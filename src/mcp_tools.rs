use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use tracing::info;

use crate::tools::amap::get_weather;
use crate::tools::serper::{search_with_key, format_results};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
  pub name: String,
  pub description: String,
  pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
  pub uri: String,
  pub name: String,
  pub description: String,
  pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
  pub name: String,
  pub description: String,
  pub arguments: Vec<McpPromptArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
  pub name: String,
  pub description: String,
  pub required: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct McpToolResult {
  pub content: String,
  pub system_prompt: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DeepSeekMcpTools {
  amap_key: String,
  serper_key: String,
  tools: Vec<McpTool>,
  resources: Vec<McpResource>,
  prompts: Vec<McpPrompt>,
}

impl DeepSeekMcpTools {
  #[allow(dead_code)]
  pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
    let amap_key = env::var("AMAP_API_KEY")?;
    let serper_key = env::var("SERPER_API_KEY")?;
    
    let tools = vec![
      McpTool {
        name: "get_weather".to_string(),
        description: "获取指定城市的天气预报信息".to_string(),
        parameters: json!({
          "type": "object",
          "properties": {
            "location": {
              "type": "string",
              "description": "城市名称，例如：上海"
            }
          },
          "required": ["location"]
        }),
      },
      McpTool {
        name: "search".to_string(),
        description: "使用Google搜索获取实时信息".to_string(),
        parameters: json!({
          "type": "object",
          "properties": {
            "query": {
              "type": "string",
              "description": "搜索查询词"
            }
          },
          "required": ["query"]
        }),
      },
    ];

    let resources = vec![
      McpResource {
        uri: "weather://recent-queries".to_string(),
        name: "Recent Weather Queries".to_string(),
        description: "Recently queried weather locations".to_string(),
        mime_type: "application/json".to_string(),
      },
      McpResource {
        uri: "search://recent-queries".to_string(),
        name: "Recent Search Queries".to_string(),
        description: "Recently performed search queries".to_string(),
        mime_type: "application/json".to_string(),
      },
    ];

    let prompts = vec![
      McpPrompt {
        name: "weather_advisor".to_string(),
        description: "专业的天气顾问提示模板".to_string(),
        arguments: vec![
          McpPromptArgument {
            name: "weather_data".to_string(),
            description: "天气数据JSON".to_string(),
            required: true,
          },
        ],
      },
      McpPrompt {
        name: "search_analyzer".to_string(),
        description: "搜索结果分析师提示模板".to_string(),
        arguments: vec![
          McpPromptArgument {
            name: "search_results".to_string(),
            description: "搜索结果数据".to_string(),
            required: true,
          },
        ],
      },
    ];
    
    Ok(Self {
      amap_key,
      serper_key,
      tools,
      resources,
      prompts,
    })
  }

  // MCP-style tool listing
  #[allow(dead_code)]
  pub fn list_tools(&self) -> &[McpTool] {
    &self.tools
  }

  // MCP-style resource listing  
  #[allow(dead_code)]
  pub fn list_resources(&self) -> &[McpResource] {
    &self.resources
  }

  // MCP-style prompt listing
  #[allow(dead_code)]
  pub fn list_prompts(&self) -> &[McpPrompt] {
    &self.prompts
  }

  // MCP-style tool execution
  #[allow(dead_code)]
  pub async fn call_tool(&self, name: &str, arguments: &Value) -> Result<McpToolResult, Box<dyn std::error::Error>> {
    info!("MCP Tool Call: {} with args: {}", name, arguments);
    
    match name {
      "get_weather" => {
        let location = arguments["location"]
          .as_str()
          .ok_or("Missing location parameter")?;

        match get_weather(location, &self.amap_key).await {
          Ok(weather_info) => {
            let content = serde_json::to_string_pretty(&weather_info)?;
            Ok(McpToolResult {
              content: format!("天气信息获取成功：\n{}", content),
              system_prompt: Some(
                "你是一个专业的天气顾问，请根据获取到的天气数据给出详细的穿衣建议。注意：\n1. 分析温度范围和温差\n2. 考虑天气现象（晴、阴、雨等）\n3. 考虑风力大小\n4. 给出具体的穿衣层次建议\n5. 如有必要，提醒是否需要携带雨具或防晒用品".to_string()
              ),
            })
          }
          Err(e) => Ok(McpToolResult {
            content: format!("Weather API error: {}", e),
            system_prompt: None,
          }),
        }
      }
      "search" => {
        let query = arguments["query"]
          .as_str()
          .ok_or("Missing query parameter")?;

        match search_with_key(query, &self.serper_key).await {
          Ok(search_results) => {
            let formatted_results = format_results(&search_results, 3);
            Ok(McpToolResult {
              content: format!("搜索结果：\n{}", formatted_results),
              system_prompt: Some(
                "你是一个专业的信息分析师，请根据搜索结果给出准确、简洁的回答。".to_string()
              ),
            })
          }
          Err(e) => Ok(McpToolResult {
            content: format!("Search API error: {}", e),
            system_prompt: None,
          }),
        }
      }
      _ => Err(format!("Unknown tool: {}", name).into()),
    }
  }

  // MCP-style resource reading
  #[allow(dead_code)]
  pub async fn read_resource(&self, uri: &str) -> Result<String, Box<dyn std::error::Error>> {
    info!("MCP Resource Read: {}", uri);
    
    match uri {
      "weather://recent-queries" => {
        Ok(r#"{"recent_queries": ["上海", "北京", "深圳"]}"#.to_string())
      }
      "search://recent-queries" => {
        Ok(r#"{"recent_queries": ["2025年编程语言排行榜", "天气查询", "MCP协议"]}"#.to_string())
      }
      _ => Err(format!("Unknown resource URI: {}", uri).into()),
    }
  }

  // MCP-style prompt generation
  #[allow(dead_code)]
  pub fn get_prompt(&self, name: &str, arguments: &Value) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    info!("MCP Prompt Get: {} with args: {}", name, arguments);
    
    match name {
      "weather_advisor" => {
        let weather_data = arguments["weather_data"]
          .as_str()
          .ok_or("Missing weather_data parameter")?;

        Ok(vec![
          "你是一个专业的天气顾问，请根据获取到的天气数据给出详细的穿衣建议。注意：\n1. 分析温度范围和温差\n2. 考虑天气现象（晴、阴、雨等）\n3. 考虑风力大小\n4. 给出具体的穿衣层次建议\n5. 如有必要，提醒是否需要携带雨具或防晒用品".to_string(),
          format!("请分析以下天气数据并给出建议：\n{}", weather_data),
        ])
      }
      "search_analyzer" => {
        let search_results = arguments["search_results"]
          .as_str()
          .ok_or("Missing search_results parameter")?;

        Ok(vec![
          "你是一个专业的信息分析师，请根据搜索结果给出准确、简洁的回答。".to_string(),
          format!("请分析以下搜索结果：\n{}", search_results),
        ])
      }
      _ => Err(format!("Unknown prompt: {}", name).into()),
    }
  }

  // Convert to MCP-style tool definitions for LLM API
  #[allow(dead_code)]
  pub fn get_mcp_tools_definition(&self) -> Value {
    json!(self.tools.iter().map(|tool| {
      json!({
        "type": "function",
        "function": {
          "name": tool.name,
          "description": tool.description,
          "parameters": tool.parameters
        }
      })
    }).collect::<Vec<_>>())
  }
}