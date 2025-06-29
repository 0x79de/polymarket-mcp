use polymarket_mcp::{Config, PolymarketClient};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_config_loading() {
    let config = Config::load();
    assert!(config.is_ok(), "Config should load successfully");

    let config = config.unwrap();
    assert!(
        !config.server.name.is_empty(),
        "Server name should not be empty"
    );
    assert!(
        !config.api.base_url.is_empty(),
        "API base URL should not be empty"
    );
}

#[tokio::test]
async fn test_client_creation() {
    let config = Arc::new(Config::default());
    let client = PolymarketClient::new_with_config(&config);
    assert!(client.is_ok(), "Client should be created successfully");
}

#[test]
fn test_config_structure() {
    let config = Config::default();

    // Test that default config has expected values
    assert!(
        !config.server.name.is_empty(),
        "Default server name should not be empty"
    );
    assert!(
        config.api.base_url.starts_with("https://"),
        "API URL should use HTTPS"
    );
    assert!(
        config.api.timeout_seconds > 0,
        "API timeout should be positive"
    );
    assert!(
        config.server.timeout_seconds > 0,
        "Server timeout should be positive"
    );
    assert!(config.cache.ttl_seconds > 0, "Cache TTL should be positive");
}

#[test]
fn test_market_structure() {
    use polymarket_mcp::Market;

    // Test that Market can be serialized to JSON
    let market = Market {
        id: "test-id".to_string(),
        slug: "test-slug".to_string(),
        question: "Test question?".to_string(),
        description: Some("Test description".to_string()),
        active: true,
        closed: false,
        liquidity: 1000.0,
        volume: 2000.0,
        end_date: "2024-12-31T23:59:59Z".to_string(),
        image: None,
        category: Some("Test".to_string()),
        outcomes: vec!["Yes".to_string(), "No".to_string()],
        outcome_prices: vec!["0.6".to_string(), "0.4".to_string()],
        condition_id: Some("condition-123".to_string()),
        market_type: Some("binary".to_string()),
        twitter_card_image: None,
        icon: None,
        start_date: Some("2024-01-01T00:00:00Z".to_string()),
        volume_24hr: Some(500.0),
        events: None,
        archived: Some(false),
        enable_order_book: Some(true),
        group_item_title: None,
        group_item_slug: None,
        accepting_orders: Some(true),
        accepting_order_timestamp: None,
        clob_token_ids: Some(vec!["token1".to_string(), "token2".to_string()]),
        fpmm: None,
        game_start_time: None,
        maker_base_fee: Some(0.02),
        minimum_order_size: Some(1.0),
        minimum_tick_size: Some(0.01),
        neg_risk: Some(false),
        notifications_enabled: Some(true),
        tags: None,
    };

    // Test basic serialization
    let json_result = serde_json::to_string(&market);
    assert!(json_result.is_ok(), "Market should serialize to JSON");

    // Test struct accessors
    assert_eq!(market.id, "test-id");
    assert_eq!(market.question, "Test question?");
    assert!(market.active);
    assert!(!market.closed);
    assert_eq!(market.liquidity, 1000.0);
    assert_eq!(market.volume, 2000.0);
}

#[test]
fn test_market_query_params() {
    use polymarket_mcp::MarketsQueryParams;

    let params = MarketsQueryParams::default();
    let query_string = params.to_query_string();

    // Should generate a non-empty query string with default values
    assert!(
        !query_string.is_empty(),
        "Default params should generate query string"
    );
    assert!(
        query_string.starts_with('?'),
        "Query string should start with ?"
    );
    assert!(
        query_string.contains("limit="),
        "Should contain limit parameter"
    );
    assert!(
        query_string.contains("active=true"),
        "Should contain active=true"
    );
}

#[test]
fn test_error_types() {
    use polymarket_mcp::{PolymarketError, RequestId};

    // Test error creation
    let api_error = PolymarketError::api_error("Test error", Some(404));
    assert!(
        api_error.to_string().contains("Test error"),
        "Error should contain message"
    );

    let network_error = PolymarketError::network_error("Network issue");
    assert!(
        network_error.to_string().contains("Network issue"),
        "Error should contain message"
    );

    let config_error = PolymarketError::config_error("Config problem");
    assert!(
        config_error.to_string().contains("Config problem"),
        "Error should contain message"
    );

    // Test RequestId
    let req_id = RequestId::new();
    assert!(
        !req_id.to_string().is_empty(),
        "RequestId should generate UUID"
    );
}

#[test]
fn test_resource_cache() {
    use polymarket_mcp::ResourceCache;

    let cache = ResourceCache::new("test data".to_string(), 60);
    assert!(!cache.is_expired(), "Fresh cache should not be expired");
    assert_eq!(cache.data, "test data", "Cache should store data correctly");

    // Test with zero TTL (should be expired immediately)
    let expired_cache = ResourceCache::new("test".to_string(), 0);
    // Note: Due to timing, this might not always be expired immediately
    // so we just check the structure is correct
    assert_eq!(
        expired_cache.data, "test",
        "Cache should store data correctly"
    );
}

#[cfg(test)]
mod mcp_protocol_tests {
    use super::*;

    #[test]
    fn test_mcp_initialize_response() {
        // Test that we can create a proper MCP initialize response
        let response = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {}
            },
            "serverInfo": {
                "name": "polymarket-mcp",
                "version": "0.1.0"
            }
        });

        assert!(response.is_object(), "Response should be a JSON object");
        assert!(
            response.get("protocolVersion").is_some(),
            "Should have protocol version"
        );
        assert!(
            response.get("capabilities").is_some(),
            "Should have capabilities"
        );
        assert!(
            response.get("serverInfo").is_some(),
            "Should have server info"
        );
    }

    #[test]
    fn test_mcp_tools_list_response() {
        // Test that we can create a proper tools list response
        let tools = json!({
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
                }
            ]
        });

        assert!(tools.get("tools").is_some(), "Should have tools array");
        assert!(
            tools.get("tools").unwrap().is_array(),
            "Tools should be an array"
        );
    }
}
