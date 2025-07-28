use serde_json::{json, Value};
use std::env;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tracing::{info, error};

use crate::tools::amap::get_weather;
use crate::tools::serper::{search_with_key, format_results};

#[derive(Debug, Clone)]
pub struct SimpleMcpServer {
  amap_key: String,
  serper_key: String,
}

impl SimpleMcpServer {
  pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
    let amap_key = env::var("AMAP_API_KEY")?;
    let serper_key = env::var("SERPER_API_KEY")?;
    
    Ok(Self {
      amap_key,
      serper_key,
    })
  }

  // MCP JSON-RPC message handling
  pub async fn handle_message(&self, message: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let method = message["method"].as_str().unwrap_or("");
    let id = message["id"].clone();
    let params = message.get("params").cloned().unwrap_or(json!({}));

    info!("MCP Server: Handling method: {}", method);

    let result = match method {
      "initialize" => self.handle_initialize(params).await?,
      "tools/list" => self.handle_list_tools().await?,
      "tools/call" => self.handle_call_tool(params).await?,
      "resources/list" => self.handle_list_resources().await?,
      "resources/read" => self.handle_read_resource(params).await?,
      "prompts/list" => self.handle_list_prompts().await?,
      "prompts/get" => self.handle_get_prompt(params).await?,
      _ => {
        return Ok(json!({
          "jsonrpc": "2.0",
          "id": id,
          "error": {
            "code": -32601,
            "message": "Method not found"
          }
        }));
      }
    };

    Ok(json!({
      "jsonrpc": "2.0",
      "id": id,
      "result": result
    }))
  }

  async fn handle_initialize(&self, _params: Value) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(json!({
      "protocolVersion": "2024-11-05",
      "capabilities": {
        "tools": {},
        "resources": {},
        "prompts": {}
      },
      "serverInfo": {
        "name": "deepseek-agent",
        "version": "0.4.0"
      }
    }))
  }

  async fn handle_list_tools(&self) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(json!({
      "tools": [
        {
          "name": "get_weather",
          "description": "获取指定城市的天气预报信息",
          "inputSchema": {
            "type": "object",
            "properties": {
              "location": {
                "type": "string",
                "description": "城市名称，例如：上海"
              }
            },
            "required": ["location"]
          }
        },
        {
          "name": "search",
          "description": "使用Google搜索获取实时信息",
          "inputSchema": {
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
      ]
    }))
  }

  async fn handle_call_tool(&self, params: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let name = params["name"].as_str().ok_or("Missing tool name")?;
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    match name {
      "get_weather" => {
        let location = arguments["location"].as_str().ok_or("Missing location parameter")?;
        
        match get_weather(location, &self.amap_key).await {
          Ok(weather_info) => {
            let content = serde_json::to_string_pretty(&weather_info)?;
            Ok(json!({
              "content": [{
                "type": "text",
                "text": format!("天气信息获取成功：\n{}", content)
              }],
              "isError": false
            }))
          }
          Err(e) => {
            Ok(json!({
              "content": [{
                "type": "text", 
                "text": format!("Weather API error: {}", e)
              }],
              "isError": true
            }))
          }
        }
      }
      "search" => {
        let query = arguments["query"].as_str().ok_or("Missing query parameter")?;
        
        match search_with_key(query, &self.serper_key).await {
          Ok(search_results) => {
            let formatted_results = format_results(&search_results, 3);
            Ok(json!({
              "content": [{
                "type": "text",
                "text": format!("搜索结果：\n{}", formatted_results)
              }],
              "isError": false
            }))
          }
          Err(e) => {
            Ok(json!({
              "content": [{
                "type": "text",
                "text": format!("Search API error: {}", e)
              }],
              "isError": true
            }))
          }
        }
      }
      _ => {
        Err(format!("Unknown tool: {}", name).into())
      }
    }
  }

  async fn handle_list_resources(&self) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(json!({
      "resources": [
        {
          "uri": "weather://recent-queries",
          "name": "Recent Weather Queries",
          "description": "Recently queried weather locations",
          "mimeType": "application/json"
        },
        {
          "uri": "search://recent-queries",
          "name": "Recent Search Queries", 
          "description": "Recently performed search queries",
          "mimeType": "application/json"
        }
      ]
    }))
  }

  async fn handle_read_resource(&self, params: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let uri = params["uri"].as_str().ok_or("Missing resource URI")?;
    
    match uri {
      "weather://recent-queries" => {
        Ok(json!({
          "contents": [{
            "type": "text",
            "text": r#"{"recent_queries": ["上海", "北京", "深圳"]}"#
          }]
        }))
      }
      "search://recent-queries" => {
        Ok(json!({
          "contents": [{
            "type": "text", 
            "text": r#"{"recent_queries": ["2025年编程语言排行榜", "MCP协议", "Rust开发"]}"#
          }]
        }))
      }
      _ => Err(format!("Unknown resource URI: {}", uri).into())
    }
  }

  async fn handle_list_prompts(&self) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(json!({
      "prompts": [
        {
          "name": "weather_advisor",
          "description": "专业的天气顾问提示模板",
          "arguments": [{
            "name": "weather_data",
            "description": "天气数据JSON",
            "required": true
          }]
        },
        {
          "name": "search_analyzer",
          "description": "搜索结果分析师提示模板", 
          "arguments": [{
            "name": "search_results",
            "description": "搜索结果数据",
            "required": true
          }]
        }
      ]
    }))
  }

  async fn handle_get_prompt(&self, params: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let name = params["name"].as_str().ok_or("Missing prompt name")?;
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    match name {
      "weather_advisor" => {
        let weather_data = arguments["weather_data"].as_str().ok_or("Missing weather_data parameter")?;
        
        Ok(json!({
          "description": "天气顾问专业分析",
          "messages": [
            {
              "role": "system",
              "content": {
                "type": "text",
                "text": "你是一个专业的天气顾问，请根据获取到的天气数据给出详细的穿衣建议。注意：\n1. 分析温度范围和温差\n2. 考虑天气现象（晴、阴、雨等）\n3. 考虑风力大小\n4. 给出具体的穿衣层次建议\n5. 如有必要，提醒是否需要携带雨具或防晒用品"
              }
            },
            {
              "role": "user", 
              "content": {
                "type": "text",
                "text": format!("请分析以下天气数据并给出建议：\n{}", weather_data)
              }
            }
          ]
        }))
      }
      "search_analyzer" => {
        let search_results = arguments["search_results"].as_str().ok_or("Missing search_results parameter")?;
        
        Ok(json!({
          "description": "搜索结果专业分析",
          "messages": [
            {
              "role": "system",
              "content": {
                "type": "text", 
                "text": "你是一个专业的信息分析师，请根据搜索结果给出准确、简洁的回答。"
              }
            },
            {
              "role": "user",
              "content": {
                "type": "text",
                "text": format!("请分析以下搜索结果：\n{}", search_results)
              }
            }
          ]
        }))
      }
      _ => Err(format!("Unknown prompt: {}", name).into())
    }
  }

  // Run the MCP server on stdio
  pub async fn run_stdio(&self) -> Result<(), Box<dyn std::error::Error>> {
    info!("MCP Server: Starting stdio server...");
    
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = AsyncBufReader::new(stdin);
    let mut line = String::new();

    loop {
      line.clear();
      let bytes_read = reader.read_line(&mut line).await?;
      
      if bytes_read == 0 {
        info!("MCP Server: EOF received, shutting down");
        break;
      }

      let line = line.trim();
      if line.is_empty() {
        continue;
      }

      match serde_json::from_str::<Value>(line) {
        Ok(message) => {
          match self.handle_message(message).await {
            Ok(response) => {
              let response_str = serde_json::to_string(&response)?;
              stdout.write_all(response_str.as_bytes()).await?;
              stdout.write_all(b"\n").await?;
              stdout.flush().await?;
            }
            Err(e) => {
              error!("MCP Server: Error handling message: {}", e);
            }
          }
        }
        Err(e) => {
          error!("MCP Server: Invalid JSON received: {}", e);
        }
      }
    }

    Ok(())
  }
}