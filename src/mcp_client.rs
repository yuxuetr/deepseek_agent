use serde_json::{Value, json};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tracing::info;

pub struct McpClient {
  child: Child,
  request_id: u64,
}

impl McpClient {
  pub async fn new(server_command: &str) -> Result<Self, Box<dyn std::error::Error>> {
    info!("MCP Client: Starting server process: {}", server_command);

    let child = Command::new("cargo")
      .args(&["run", "--bin", "mcp_server"])
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .spawn()?;

    Ok(Self {
      child,
      request_id: 0,
    })
  }

  fn next_request_id(&mut self) -> u64 {
    self.request_id += 1;
    self.request_id
  }

  pub async fn send_request(
    &mut self,
    method: &str,
    params: Value,
  ) -> Result<Value, Box<dyn std::error::Error>> {
    let request_id = self.next_request_id();
    let request = json!({
      "jsonrpc": "2.0",
      "id": request_id,
      "method": method,
      "params": params
    });

    info!("MCP Client: Sending request: {}", method);

    // Send request to server
    let stdin = self.child.stdin.as_mut().ok_or("Failed to get stdin")?;
    let request_str = serde_json::to_string(&request)?;
    stdin.write_all(request_str.as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    // Read response from server
    let stdout = self.child.stdout.as_mut().ok_or("Failed to get stdout")?;
    let mut reader = BufReader::new(stdout);
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    let response: Value = serde_json::from_str(&response_line)?;

    if let Some(error) = response.get("error") {
      return Err(format!("MCP Server Error: {}", error).into());
    }

    Ok(response["result"].clone())
  }

  pub async fn initialize(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
    self
      .send_request(
        "initialize",
        json!({
          "protocolVersion": "2024-11-05",
          "capabilities": {}
        }),
      )
      .await
  }

  pub async fn list_tools(&mut self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let result = self.send_request("tools/list", json!({})).await?;
    Ok(result["tools"].as_array().unwrap_or(&vec![]).clone())
  }

  pub async fn call_tool(
    &mut self,
    name: &str,
    arguments: Value,
  ) -> Result<Value, Box<dyn std::error::Error>> {
    self
      .send_request(
        "tools/call",
        json!({
          "name": name,
          "arguments": arguments
        }),
      )
      .await
  }

  #[allow(dead_code)]
  pub async fn list_resources(&mut self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let result = self.send_request("resources/list", json!({})).await?;
    Ok(result["resources"].as_array().unwrap_or(&vec![]).clone())
  }

  #[allow(dead_code)]
  pub async fn read_resource(&mut self, uri: &str) -> Result<Value, Box<dyn std::error::Error>> {
    self
      .send_request(
        "resources/read",
        json!({
          "uri": uri
        }),
      )
      .await
  }

  #[allow(dead_code)]
  pub async fn list_prompts(&mut self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let result = self.send_request("prompts/list", json!({})).await?;
    Ok(result["prompts"].as_array().unwrap_or(&vec![]).clone())
  }

  #[allow(dead_code)]
  pub async fn get_prompt(
    &mut self,
    name: &str,
    arguments: Value,
  ) -> Result<Value, Box<dyn std::error::Error>> {
    self
      .send_request(
        "prompts/get",
        json!({
          "name": name,
          "arguments": arguments
        }),
      )
      .await
  }
}

impl Drop for McpClient {
  fn drop(&mut self) {
    // Use start() to spawn the future without awaiting it
    tokio::spawn(async move {
      // This will terminate the process when the client is dropped
    });

    // Force kill the child process
    let _ = self.child.start_kill();
  }
}
