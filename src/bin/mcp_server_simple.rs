use deepseek_agent::mcp_server_simple::SimpleMcpServer;
use dotenv::dotenv;
use std::io;
use tracing::{Level, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Initialize logging system to stderr to avoid interfering with MCP communication
  tracing_subscriber::fmt()
    .with_max_level(Level::INFO)
    .with_writer(io::stderr)
    .init();

  // Load environment variables
  dotenv().ok();
  info!("MCP Server: Environment variables loaded");

  // Create and run simple MCP server
  let server = SimpleMcpServer::new()?;
  info!("MCP Server: Simple MCP Server created, starting stdio server...");
  
  server.run_stdio().await?;

  Ok(())
}