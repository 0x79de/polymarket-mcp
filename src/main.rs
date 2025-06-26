mod config;
mod error;
mod models;
mod polymarket_client;

use anyhow::Result;
use config::Config;
use error::{Metrics, RequestId};
use models::*;
use polymarket_client::PolymarketClient;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use tracing_subscriber::{self, EnvFilter, FmtSubscriber};

#[derive(Debug)]
pub struct PolymarketMcpServer {
    client: Arc<PolymarketClient>,
    resource_cache: Arc<RwLock<HashMap<String, ResourceCache>>>,
    config: Arc<Config>,
    metrics: Arc<RwLock<Metrics>>,
}

impl PolymarketMcpServer {
    pub fn new() -> Result<Self> {
        let config = Arc::new(Config::load()?);
        let client = Arc::new(PolymarketClient::new_with_config(&config)?);
        Ok(Self { 
            client,
            resource_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            metrics: Arc::new(RwLock::new(Metrics::new())),
        })
    }
    
    pub fn with_config(config: Config) -> Result<Self> {
        let config = Arc::new(config);
        let client = Arc::new(PolymarketClient::new_with_config(&config)?);
        Ok(Self { 
            client,
            resource_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
            metrics: Arc::new(RwLock::new(Metrics::new())),
        })
    }

    pub async fn get_active_markets(&self, limit: Option<u32>) -> Result<Value> {
        let request_id = RequestId::new();
        info!(request_id = %request_id, "Fetching active markets with limit: {:?}", limit);
        
        {
            let mut metrics = self.metrics.write().await;
            metrics.increment_api_requests();
        }
        
        let start_time = std::time::Instant::now();
        
        match self.client.get_active_markets(limit).await {
            Ok(markets) => {
                let response_time = start_time.elapsed().as_millis() as f64;
                {
                    let mut metrics = self.metrics.write().await;
                    metrics.update_avg_response_time(response_time);
                }
                
                let result = json!({
                    "markets": markets,
                    "count": markets.len(),
                    "request_id": request_id.as_str()
                });
                
                Ok(result)
            }
            Err(e) => {
                {
                    let mut metrics = self.metrics.write().await;
                    metrics.increment_api_failures();
                }
                warn!(request_id = %request_id, "Failed to fetch active markets: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn get_market_details(&self, market_id: String) -> Result<Value> {
        info!("Fetching market details for: {}", market_id);
        
        let market = self.client.get_market_by_id(&market_id).await?;
        let result = json!(market);
        
        Ok(result)
    }

    pub async fn search_markets(&self, keyword: String, limit: Option<u32>) -> Result<Value> {
        info!("Searching markets for keyword: '{}' with limit: {:?}", keyword, limit);
        
        let markets = self.client.search_markets(&keyword, limit).await?;
        let result = json!({
            "markets": markets,
            "count": markets.len(),
            "keyword": keyword
        });
        
        Ok(result)
    }

    pub async fn get_market_prices(&self, market_id: String) -> Result<Value> {
        info!("Fetching market prices for: {}", market_id);
        
        let prices = self.client.get_market_prices(&market_id).await?;
        let result = json!({
            "market_id": market_id,
            "prices": prices
        });
        
        Ok(result)
    }

    pub async fn get_trending_markets(&self, limit: Option<u32>) -> Result<Value> {
        info!("Fetching trending markets with limit: {:?}", limit);
        
        let markets = self.client.get_trending_markets(limit).await?;
        let result = json!({
            "markets": markets,
            "count": markets.len()
        });
        
        Ok(result)
    }

    // MCP Resources Support
    pub async fn list_resources(&self) -> Result<Value> {
        info!("Listing available MCP resources");
        
        let resources = vec![
            McpResource {
                uri: "markets:active".to_string(),
                name: "Active Markets".to_string(),
                description: "List of currently active prediction markets".to_string(),
                mime_type: "application/json".to_string(),
            },
            McpResource {
                uri: "markets:trending".to_string(),
                name: "Trending Markets".to_string(),
                description: "Markets with highest trading volume".to_string(),
                mime_type: "application/json".to_string(),
            },
        ];

        Ok(json!({ "resources": resources }))
    }

    pub async fn read_resource(&self, uri: &str) -> Result<Value> {
        info!("Reading resource: {}", uri);

        // Check cache first
        {
            let cache = self.resource_cache.read().await;
            if let Some(cached) = cache.get(uri) {
                if !cached.is_expired() {
                    info!("Returning cached resource for: {}", uri);
                    return Ok(json!({
                        "contents": [{
                            "uri": uri,
                            "mimeType": "application/json",
                            "text": cached.data
                        }]
                    }));
                }
            }
        }

        let content = match uri {
            "markets:active" => {
                let markets = self.client.get_active_markets(Some(20)).await?;
                serde_json::to_string_pretty(&json!({
                    "markets": markets,
                    "count": markets.len(),
                    "last_updated": chrono::Utc::now().to_rfc3339()
                }))?
            }
            "markets:trending" => {
                let markets = self.client.get_trending_markets(Some(10)).await?;
                serde_json::to_string_pretty(&json!({
                    "markets": markets,
                    "count": markets.len(),
                    "last_updated": chrono::Utc::now().to_rfc3339()
                }))?
            }
            _ if uri.starts_with("market:") => {
                let market_id = uri.strip_prefix("market:").unwrap();
                let market = self.client.get_market_by_id(market_id).await?;
                serde_json::to_string_pretty(&market)?
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown resource URI: {}", uri));
            }
        };

        // Cache the result
        {
            let mut cache = self.resource_cache.write().await;
            let ttl = self.config.resource_cache_ttl().as_secs();
            cache.insert(uri.to_string(), ResourceCache::new(content.clone(), ttl));
        }

        Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": content
            }]
        }))
    }

    // MCP Prompts Support
    pub async fn list_prompts(&self) -> Result<Value> {
        info!("Listing available MCP prompts");
        
        let prompts = vec![
            McpPrompt {
                name: "analyze_market".to_string(),
                description: "Analyze a prediction market and provide insights on trends, liquidity, and potential opportunities".to_string(),
                arguments: vec![
                    McpPromptArgument {
                        name: "market_id".to_string(),
                        description: "The ID of the market to analyze".to_string(),
                        required: true,
                    }
                ],
            },
            McpPrompt {
                name: "find_arbitrage".to_string(),
                description: "Look for arbitrage opportunities across multiple markets with similar outcomes".to_string(),
                arguments: vec![
                    McpPromptArgument {
                        name: "keyword".to_string(),
                        description: "Keyword to search for related markets".to_string(),
                        required: true,
                    },
                    McpPromptArgument {
                        name: "limit".to_string(),
                        description: "Maximum number of markets to analyze (default: 10)".to_string(),
                        required: false,
                    }
                ],
            },
            McpPrompt {
                name: "market_summary".to_string(),
                description: "Provide a comprehensive summary of the top prediction markets".to_string(),
                arguments: vec![
                    McpPromptArgument {
                        name: "category".to_string(),
                        description: "Filter by category (optional)".to_string(),
                        required: false,
                    },
                    McpPromptArgument {
                        name: "limit".to_string(),
                        description: "Number of markets to include (default: 5)".to_string(),
                        required: false,
                    }
                ],
            },
        ];

        Ok(json!({ "prompts": prompts }))
    }

    pub async fn get_prompt(&self, name: &str, arguments: Option<Value>) -> Result<Value> {
        info!("Getting prompt: {} with arguments: {:?}", name, arguments);
        
        let args = arguments.unwrap_or_default();
        
        let messages = match name {
            "analyze_market" => {
                let market_id = args.get("market_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("market_id argument is required"))?;
                
                let market = self.client.get_market_by_id(market_id).await?;
                let prices = self.client.get_market_prices(market_id).await?;
                
                vec![
                    McpPromptMessage {
                        role: "user".to_string(),
                        content: McpPromptContent::Text(format!(
                            "Analyze this prediction market:\n\nMarket: {}\nQuestion: {}\nLiquidity: ${:.0}\nVolume: ${:.0}\nActive: {}\n\nCurrent Prices:\n{}\n\nProvide analysis on:\n1. Market sentiment and trends\n2. Liquidity assessment\n3. Price efficiency\n4. Potential trading opportunities\n5. Risk factors",
                            market.id,
                            market.question,
                            market.liquidity,
                            market.volume,
                            market.active,
                            serde_json::to_string_pretty(&prices)?
                        ))
                    }
                ]
            }
            "find_arbitrage" => {
                let keyword = args.get("keyword")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("keyword argument is required"))?;
                
                let limit = args.get("limit")
                    .and_then(|v| v.as_u64())
                    .map(|l| l as u32)
                    .unwrap_or(10);
                
                let markets = self.client.search_markets(keyword, Some(limit)).await?;
                
                vec![
                    McpPromptMessage {
                        role: "user".to_string(),
                        content: McpPromptContent::Text(format!(
                            "Find arbitrage opportunities among these related markets:\n\nKeyword: {}\nMarkets found: {}\n\n{}\n\nAnalyze:\n1. Similar questions with different prices\n2. Cross-market arbitrage opportunities\n3. Risk-adjusted returns\n4. Execution feasibility\n5. Recommended actions",
                            keyword,
                            markets.len(),
                            serde_json::to_string_pretty(&markets)?
                        ))
                    }
                ]
            }
            "market_summary" => {
                let limit = args.get("limit")
                    .and_then(|v| v.as_u64())
                    .map(|l| l as u32)
                    .unwrap_or(5);
                
                let trending = self.client.get_trending_markets(Some(limit)).await?;
                let active = self.client.get_active_markets(Some(limit)).await?;
                
                vec![
                    McpPromptMessage {
                        role: "user".to_string(),
                        content: McpPromptContent::Text(format!(
                            "Provide a comprehensive market summary:\n\nTop Trending Markets (by volume):\n{}\n\nTop Active Markets:\n{}\n\nSummarize:\n1. Overall market sentiment\n2. Popular categories and themes\n3. Liquidity distribution\n4. Notable price movements\n5. Trading recommendations",
                            serde_json::to_string_pretty(&trending)?,
                            serde_json::to_string_pretty(&active)?
                        ))
                    }
                ]
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown prompt: {}", name));
            }
        };

        Ok(json!({ "messages": messages }))
    }

    pub async fn get_metrics(&self) -> Result<Value> {
        let metrics = self.metrics.read().await;
        Ok(json!({
            "api_requests_total": metrics.api_requests_total,
            "api_requests_failed": metrics.api_requests_failed,
            "cache_hits": metrics.cache_hits,
            "cache_misses": metrics.cache_misses,
            "cache_hit_ratio": metrics.cache_hit_ratio(),
            "error_rate": metrics.error_rate(),
            "active_connections": metrics.active_connections,
            "avg_response_time_ms": metrics.avg_response_time_ms
        }))
    }
}

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file if it exists
    dotenv::dotenv().ok();
    
    // Load configuration
    let config = Config::load()?;
    
    // Initialize tracing subscriber to write to stderr only
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.logging.level));
        
    // Write logs to stderr to avoid interfering with MCP JSON protocol on stdout
    FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .compact()
        .init();

    eprintln!("Starting Polymarket MCP Server");
    
    // Create the MCP server handler with configuration
    let server = Arc::new(PolymarketMcpServer::with_config(config)?);
    
    // Set up MCP server using stdin/stdout
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let mut reader = AsyncBufReader::new(stdin);
    let mut writer = stdout;
    
    let mut line = String::new();
    
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                if let Ok(request) = serde_json::from_str::<serde_json::Value>(&line) {
                    if let Some(response) = handle_mcp_request(&server, request).await {
                        let response_json = serde_json::to_string(&response).unwrap();
                        writer.write_all(response_json.as_bytes()).await.unwrap();
                        writer.write_all(b"\n").await.unwrap();
                        writer.flush().await.unwrap();
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                break;
            }
        }
    }
    
    eprintln!("Shutting down Polymarket MCP Server");
    Ok(())
}

async fn handle_mcp_request(server: &Arc<PolymarketMcpServer>, request: serde_json::Value) -> Option<serde_json::Value> {
    let method = request.get("method")?.as_str()?;
    let id = request.get("id").cloned();
    let params = request.get("params").cloned().unwrap_or(serde_json::Value::Null);
    
    // Handle notifications (no response expected)
    if method.starts_with("notifications/") {
        return None;
    }
    
    let result = match method {
        "initialize" => {
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "prompts": {}
                },
                "serverInfo": {
                    "name": "polymarket-mcp",
                    "version": "1.0.0"
                }
            })
        }
        "tools/list" => {
            json!({
                "tools": [
                    {
                        "name": "get_active_markets",
                        "description": "Get list of active prediction markets",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "limit": {
                                    "type": "number",
                                    "description": "Maximum number of markets to return"
                                }
                            }
                        }
                    },
                    {
                        "name": "get_market_details",
                        "description": "Get detailed information about a specific market",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "market_id": {
                                    "type": "string",
                                    "description": "The ID of the market"
                                }
                            },
                            "required": ["market_id"]
                        }
                    },
                    {
                        "name": "search_markets",
                        "description": "Search markets by keyword",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "keyword": {
                                    "type": "string",
                                    "description": "Keyword to search for"
                                },
                                "limit": {
                                    "type": "number",
                                    "description": "Maximum number of results"
                                }
                            },
                            "required": ["keyword"]
                        }
                    },
                    {
                        "name": "get_market_prices",
                        "description": "Get current prices for a market",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "market_id": {
                                    "type": "string",
                                    "description": "The ID of the market"
                                }
                            },
                            "required": ["market_id"]
                        }
                    },
                    {
                        "name": "get_trending_markets",
                        "description": "Get trending markets with high volume",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "limit": {
                                    "type": "number",
                                    "description": "Maximum number of markets to return"
                                }
                            }
                        }
                    }
                ]
            })
        }
        "tools/call" => {
            let name = params.get("name")?.as_str()?;
            let arguments = params.get("arguments").cloned().unwrap_or(serde_json::Value::Object(Default::default()));
            
            match name {
                "get_active_markets" => {
                    let limit = arguments.get("limit").and_then(|v| v.as_u64()).map(|l| l as u32);
                    match server.get_active_markets(limit).await {
                        Ok(result) => json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&result).unwrap()
                            }]
                        }),
                        Err(e) => json!({
                            "content": [{
                                "type": "text", 
                                "text": format!("Error: {}", e)
                            }],
                            "isError": true
                        })
                    }
                }
                "get_market_details" => {
                    let market_id = arguments.get("market_id")?.as_str()?.to_string();
                    match server.get_market_details(market_id).await {
                        Ok(result) => json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&result).unwrap()
                            }]
                        }),
                        Err(e) => json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Error: {}", e)
                            }],
                            "isError": true
                        })
                    }
                }
                "search_markets" => {
                    let keyword = arguments.get("keyword")?.as_str()?.to_string();
                    let limit = arguments.get("limit").and_then(|v| v.as_u64()).map(|l| l as u32);
                    match server.search_markets(keyword, limit).await {
                        Ok(result) => json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&result).unwrap()
                            }]
                        }),
                        Err(e) => json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Error: {}", e)
                            }],
                            "isError": true
                        })
                    }
                }
                "get_market_prices" => {
                    let market_id = arguments.get("market_id")?.as_str()?.to_string();
                    match server.get_market_prices(market_id).await {
                        Ok(result) => json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&result).unwrap()
                            }]
                        }),
                        Err(e) => json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Error: {}", e)
                            }],
                            "isError": true
                        })
                    }
                }
                "get_trending_markets" => {
                    let limit = arguments.get("limit").and_then(|v| v.as_u64()).map(|l| l as u32);
                    match server.get_trending_markets(limit).await {
                        Ok(result) => json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&result).unwrap()
                            }]
                        }),
                        Err(e) => json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Error: {}", e)
                            }],
                            "isError": true
                        })
                    }
                }
                _ => json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Unknown tool: {}", name)
                    }],
                    "isError": true
                })
            }
        }
        "resources/list" => {
            match server.list_resources().await {
                Ok(result) => result,
                Err(e) => json!({
                    "resources": [],
                    "error": format!("Error listing resources: {}", e)
                })
            }
        }
        "resources/read" => {
            let uri = params.get("uri")?.as_str()?;
            match server.read_resource(uri).await {
                Ok(result) => result,
                Err(e) => json!({
                    "contents": [],
                    "error": format!("Error reading resource: {}", e)
                })
            }
        }
        "prompts/list" => {
            match server.list_prompts().await {
                Ok(result) => result,
                Err(e) => json!({
                    "prompts": [],
                    "error": format!("Error listing prompts: {}", e)
                })
            }
        }
        "prompts/get" => {
            let name = params.get("name")?.as_str()?;
            let arguments = params.get("arguments").cloned();
            match server.get_prompt(name, arguments).await {
                Ok(result) => result,
                Err(e) => json!({
                    "messages": [],
                    "error": format!("Error getting prompt: {}", e)
                })
            }
        }
        _ => {
            json!({
                "error": {
                    "code": -32601,
                    "message": "Method not found"
                }
            })
        }
    };
    
    Some(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    }))
}
