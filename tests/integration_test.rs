use polymarket_mcp::config::Config;
use polymarket_mcp::polymarket_client::PolymarketClient;
use mockito::{Matcher, Server};
use std::sync::Arc;

#[tokio::test]
async fn test_get_active_markets_success() {
    let mut server = Server::new_async().await;
    
    let mock_response = r#"[
        {
            "id": "test-market-1",
            "slug": "test-market",
            "question": "Will this test pass?",
            "description": "A test market",
            "active": true,
            "closed": false,
            "liquidity": "1000.0",
            "volume": "2000.0",
            "endDate": "2024-12-31T23:59:59Z",
            "image": null,
            "category": "Testing",
            "outcomes": "[\"Yes\", \"No\"]",
            "outcomePrices": "[\"0.6\", \"0.4\"]",
            "conditionId": "test-condition",
            "marketType": "binary",
            "twitterCardImage": null,
            "icon": null,
            "startDate": "2024-01-01T00:00:00Z",
            "volume24hr": 100.0,
            "events": null,
            "archived": false,
            "enableOrderBook": true,
            "groupItemTitle": null,
            "groupItemSlug": null
        }
    ]"#;

    let _m = server
        .mock("GET", Matcher::Regex(r"^/markets.*".to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let mut config = Config::default();
    config.api.base_url = server.url();
    config.cache.enabled = false; // Disable cache for testing

    let client = PolymarketClient::new_with_config(&Arc::new(config)).unwrap();
    let result = client.get_active_markets(Some(10)).await;

    assert!(result.is_ok());
    let markets = result.unwrap();
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].id, "test-market-1");
    assert_eq!(markets[0].question, "Will this test pass?");
}

#[tokio::test]
async fn test_get_market_by_id_success() {
    let mut server = Server::new_async().await;
    
    let mock_response = r#"{
        "id": "test-market-1",
        "slug": "test-market",
        "question": "Will this test pass?",
        "description": "A test market",
        "active": true,
        "closed": false,
        "liquidity": "1000.0",
        "volume": "2000.0",
        "endDate": "2024-12-31T23:59:59Z",
        "image": null,
        "category": "Testing",
        "outcomes": "[\"Yes\", \"No\"]",
        "outcomePrices": "[\"0.6\", \"0.4\"]",
        "conditionId": "test-condition",
        "marketType": "binary",
        "twitterCardImage": null,
        "icon": null,
        "startDate": "2024-01-01T00:00:00Z",
        "volume24hr": 100.0,
        "events": null,
        "archived": false,
        "enableOrderBook": true,
        "groupItemTitle": null,
        "groupItemSlug": null
    }"#;

    let _m = server
        .mock("GET", "/markets/test-market-1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let mut config = Config::default();
    config.api.base_url = server.url();
    config.cache.enabled = false;

    let client = PolymarketClient::new_with_config(&Arc::new(config)).unwrap();
    let result = client.get_market_by_id("test-market-1").await;

    assert!(result.is_ok());
    let market = result.unwrap();
    assert_eq!(market.id, "test-market-1");
    assert_eq!(market.question, "Will this test pass?");
    assert_eq!(market.liquidity, 1000.0);
}

#[tokio::test]
async fn test_api_error_handling() {
    let mut server = Server::new_async().await;
    
    let _m = server
        .mock("GET", "/markets/nonexistent")
        .with_status(404)
        .with_body("Market not found")
        .create_async()
        .await;

    let mut config = Config::default();
    config.api.base_url = server.url();
    config.cache.enabled = false;

    let client = PolymarketClient::new_with_config(&Arc::new(config)).unwrap();
    let result = client.get_market_by_id("nonexistent").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_search_markets() {
    let mut server = Server::new_async().await;
    
    let mock_response = r#"[
        {
            "id": "election-market-1",
            "slug": "election-test",
            "question": "Will candidate A win the election?",
            "description": "Election prediction market",
            "active": true,
            "closed": false,
            "liquidity": "5000.0",
            "volume": "10000.0",
            "endDate": "2024-11-05T23:59:59Z",
            "image": null,
            "category": "Politics",
            "outcomes": "[\"Yes\", \"No\"]",
            "outcomePrices": "[\"0.55\", \"0.45\"]",
            "conditionId": "election-condition",
            "marketType": "binary",
            "twitterCardImage": null,
            "icon": null,
            "startDate": "2024-01-01T00:00:00Z",
            "volume24hr": 500.0,
            "events": null,
            "archived": false,
            "enableOrderBook": true,
            "groupItemTitle": null,
            "groupItemSlug": null
        }
    ]"#;

    let _m = server
        .mock("GET", Matcher::Regex(r"^/markets.*".to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let mut config = Config::default();
    config.api.base_url = server.url();
    config.cache.enabled = false;

    let client = PolymarketClient::new_with_config(&Arc::new(config)).unwrap();
    let result = client.search_markets("election", Some(10)).await;

    assert!(result.is_ok());
    let markets = result.unwrap();
    assert_eq!(markets.len(), 1);
    assert!(markets[0].question.to_lowercase().contains("election"));
}

#[tokio::test]
async fn test_caching_functionality() {
    let mut server = Server::new_async().await;
    
    let mock_response = r#"[
        {
            "id": "cached-market-1",
            "slug": "cached-test",
            "question": "Will caching work?",
            "description": "A cache test market",
            "active": true,
            "closed": false,
            "liquidity": "1000.0",
            "volume": "2000.0",
            "endDate": "2024-12-31T23:59:59Z",
            "image": null,
            "category": "Testing",
            "outcomes": "[\"Yes\", \"No\"]",
            "outcomePrices": "[\"0.7\", \"0.3\"]",
            "conditionId": "cache-condition",
            "marketType": "binary",
            "twitterCardImage": null,
            "icon": null,
            "startDate": "2024-01-01T00:00:00Z",
            "volume24hr": 150.0,
            "events": null,
            "archived": false,
            "enableOrderBook": true,
            "groupItemTitle": null,
            "groupItemSlug": null
        }
    ]"#;

    // Mock should only be called once due to caching
    let _m = server
        .mock("GET", Matcher::Regex(r"^/markets.*".to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .expect(1) // Expect exactly one call
        .create_async()
        .await;

    let mut config = Config::default();
    config.api.base_url = server.url();
    config.cache.enabled = true;
    config.cache.ttl_seconds = 10; // Short TTL for testing

    let client = PolymarketClient::new_with_config(&Arc::new(config)).unwrap();
    
    // First call - should hit the API
    let result1 = client.get_active_markets(Some(50)).await;
    assert!(result1.is_ok());
    
    // Second call - should use cache
    let result2 = client.get_active_markets(Some(50)).await;
    assert!(result2.is_ok());
    
    // Results should be identical
    let markets1 = result1.unwrap();
    let markets2 = result2.unwrap();
    assert_eq!(markets1.len(), markets2.len());
    assert_eq!(markets1[0].id, markets2[0].id);
}

#[tokio::test]
async fn test_batch_requests() {
    let mut server = Server::new_async().await;
    
    let mock_response1 = r#"{
        "id": "batch-market-1",
        "slug": "batch-1",
        "question": "Batch test 1?",
        "description": "First batch market",
        "active": true,
        "closed": false,
        "liquidity": "1000.0",
        "volume": "2000.0",
        "endDate": "2024-12-31T23:59:59Z",
        "image": null,
        "category": "Testing",
        "outcomes": "[\"Yes\", \"No\"]",
        "outcomePrices": "[\"0.6\", \"0.4\"]",
        "conditionId": "batch-condition-1",
        "marketType": "binary",
        "twitterCardImage": null,
        "icon": null,
        "startDate": "2024-01-01T00:00:00Z",
        "volume24hr": 100.0,
        "events": null,
        "archived": false,
        "enableOrderBook": true,
        "groupItemTitle": null,
        "groupItemSlug": null
    }"#;

    let mock_response2 = r#"{
        "id": "batch-market-2",
        "slug": "batch-2",
        "question": "Batch test 2?",
        "description": "Second batch market",
        "active": true,
        "closed": false,
        "liquidity": "1500.0",
        "volume": "3000.0",
        "endDate": "2024-12-31T23:59:59Z",
        "image": null,
        "category": "Testing",
        "outcomes": "[\"Yes\", \"No\"]",
        "outcomePrices": "[\"0.7\", \"0.3\"]",
        "conditionId": "batch-condition-2",
        "marketType": "binary",
        "twitterCardImage": null,
        "icon": null,
        "startDate": "2024-01-01T00:00:00Z",
        "volume24hr": 200.0,
        "events": null,
        "archived": false,
        "enableOrderBook": true,
        "groupItemTitle": null,
        "groupItemSlug": null
    }"#;

    let _m1 = server
        .mock("GET", "/markets/batch-market-1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response1)
        .create_async()
        .await;

    let _m2 = server
        .mock("GET", "/markets/batch-market-2")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response2)
        .create_async()
        .await;

    let mut config = Config::default();
    config.api.base_url = server.url();
    config.cache.enabled = false;

    let client = PolymarketClient::new_with_config(&Arc::new(config)).unwrap();
    let market_ids = vec!["batch-market-1".to_string(), "batch-market-2".to_string()];
    let result = client.get_markets_batch(market_ids).await;

    assert!(result.is_ok());
    let markets = result.unwrap();
    assert_eq!(markets.len(), 2);
    assert_eq!(markets[0].id, "batch-market-1");
    assert_eq!(markets[1].id, "batch-market-2");
}

#[tokio::test]
async fn test_metrics_collection() {
    let mut server = Server::new_async().await;
    
    let mock_response = r#"[]"#;

    let _m = server
        .mock("GET", "/markets")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create_async()
        .await;

    let mut config = Config::default();
    config.api.base_url = server.url();
    config.cache.enabled = false;

    let client = PolymarketClient::new_with_config(&Arc::new(config)).unwrap();
    
    // Make a request to generate metrics
    let _result = client.get_active_markets(Some(10)).await;
    
    // Check metrics
    let metrics = client.get_metrics().await;
    assert!(metrics.api_requests_total > 0);
}