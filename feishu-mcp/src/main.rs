/// Feishu MCP Server — Binary entry point
///
/// MCP (Model Context Protocol) server exposing Feishu APIs
/// (documents, tasks, messages) as MCP tools.

use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;

use feishu_mcp::auth::token_store::TokenStore;
use feishu_mcp::config::Config;
use feishu_mcp::feishu::client::FeishuClient;
use feishu_mcp::middleware::rate_limit::RateLimiter;
use feishu_mcp::FeishuMcpServer;

/// Feishu MCP Server — Expose Feishu APIs as MCP tools
#[derive(Parser, Debug)]
#[command(name = "feishu-mcp-server")]
#[command(about = "Feishu MCP Server: Expose Feishu APIs as MCP tools for Claude/Cursor/etc.")]
struct Args {
    /// Port for Streamable HTTP transport (optional, default: stdio transport)
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Path to config file (default: ~/.feishu-mcp/config.yaml)
    #[arg(short, long)]
    config: Option<String>,

    /// Use HTTP transport instead of stdio
    #[arg(long)]
    http: bool,

    /// Verbose logging (debug level)
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(args.verbose)?;

    // Load configuration
    let config = if let Some(ref config_path) = args.config {
        load_config_from_file(config_path)?
    } else {
        Config::load().context("Failed to load configuration")?
    };

    tracing::info!("Feishu MCP Server starting up");

    // Initialize FeishuClient
    let feishu_client = init_feishu_client(&config)?;

    // Initialize rate limiter
    let rate_limiter = Arc::new(RateLimiter::new(
        config.rate_limit.max_requests_per_minute,
        60,
    ));

    // Create MCP server
    let server = FeishuMcpServer::new(
        feishu_client,
        rate_limiter,
    );

    tracing::info!(
        "Starting MCP server (port={}, http={})",
        args.port,
        args.http
    );

    // Run the server with appropriate transport
    if args.http {
        server.run_http(args.port).await?;
    } else {
        server.run_stdio().await?;
    }

    Ok(())
}

/// Initialize tracing/logging
fn init_logging(verbose: bool) -> Result<()> {
    let level = if verbose { "debug" } else { "info" };

    tracing_subscriber::fmt()
        .with_env_filter(format!("feishu_mcp={}", level))
        .with_target(true)
        .init();

    Ok(())
}

/// Load config from a specific file path
fn load_config_from_file(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path))?;
    let config: Config = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path))?;
    Ok(config)
}

/// Initialize FeishuClient from configuration
fn init_feishu_client(config: &Config) -> Result<Arc<FeishuClient>> {
    let encryption_key = get_encryption_key()?;
    let token_store = init_token_store(&encryption_key)?;

    let client = FeishuClient::new(
        config.feishu.base_url.clone(),
        token_store,
        config.feishu.app_id.clone(),
    );

    Ok(Arc::new(client))
}

/// Get encryption key from environment or use a default
fn get_encryption_key() -> Result<[u8; 32]> {
    if let Ok(key_str) = std::env::var("FEISHU_ENCRYPTION_KEY") {
        let key_bytes = key_str.as_bytes();
        if key_bytes.len() < 32 {
            anyhow::bail!("FEISHU_ENCRYPTION_KEY must be at least 32 bytes");
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes[..32]);
        Ok(key)
    } else {
        // Use a default key — in production, this should be set via environment
        tracing::warn!("FEISHU_ENCRYPTION_KEY not set, using default key");
        let default_key = b"feishu-mcp-default-encryption-key-0000";
        let mut key = [0u8; 32];
        key.copy_from_slice(&default_key[..32]);
        Ok(key)
    }
}

/// Initialize token store with SQLite + AES-GCM encryption
fn init_token_store(encryption_key: &[u8; 32]) -> Result<Arc<TokenStore>> {
    let db_path = get_db_path()?;
    let store = TokenStore::new(db_path, encryption_key)
        .context("Failed to initialize token store")?;
    Ok(Arc::new(store))
}

/// Get database path for token storage
fn get_db_path() -> Result<String> {
    if let Ok(path) = std::env::var("FEISHU_TOKEN_DB") {
        return Ok(path);
    }

    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".feishu-mcp");
    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("tokens.db").to_string_lossy().to_string())
}
