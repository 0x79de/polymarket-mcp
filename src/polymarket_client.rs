use crate::config::Config;
use crate::error::{Metrics, PolymarketError, RequestId, Result};
use crate::models::*;
use futures::future;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub data: T,
    pub timestamp: Instant,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            timestamp: Instant::now(),
        }
    }

    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed() > ttl
    }
}

#[derive(Debug)]
pub struct PolymarketClient {
    client: Client,
    base_url: String,
    config: Arc<Config>,
    market_cache: Arc<RwLock<HashMap<String, CacheEntry<Vec<Market>>>>>,
    single_market_cache: Arc<RwLock<HashMap<String, CacheEntry<Market>>>>,
    metrics: Arc<RwLock<Metrics>>,
}

impl PolymarketClient {
    pub fn new_with_config(config: &Arc<Config>) -> Result<Self> {
        let client_builder = Client::builder()
            .timeout(config.api_timeout())
            .gzip(true)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(30))
            .tcp_keepalive(Duration::from_secs(60));
            
        let client_builder = if let Some(ref api_key) = config.api.api_key {
            let mut headers = reqwest::header::HeaderMap::new();
            let auth_value = reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| PolymarketError::config_error(format!("Invalid API key: {}", e)))?;
            headers.insert(reqwest::header::AUTHORIZATION, auth_value);
            client_builder.default_headers(headers)
        } else {
            client_builder
        };
        
        let client = client_builder.build()
            .map_err(|e| PolymarketError::config_error(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url: config.api.base_url.clone(),
            config: config.clone(),
            market_cache: Arc::new(RwLock::new(HashMap::new())),
            single_market_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(Metrics::new())),
        })
    }

    async fn make_request_with_retry<T: for<'de> serde::Deserialize<'de>>(
        &self,
        url: &str,
    ) -> Result<T> {
        let request_id = RequestId::new();
        debug!(request_id = %request_id, "Making request to: {}", url);

        {
            let mut metrics = self.metrics.write().await;
            metrics.increment_api_requests();
        }
        
        let mut last_error = None;
        let max_retries = self.config.api.max_retries;
        let start_time = Instant::now();

        for attempt in 1..=max_retries {
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(text) => {
                                debug!("Raw response from {}: {}", url, &text[..std::cmp::min(500, text.len())]);
                                match serde_json::from_str::<T>(&text) {
                                    Ok(data) => {
                                        let response_time = start_time.elapsed().as_millis() as f64;
                                        {
                                            let mut metrics = self.metrics.write().await;
                                            metrics.update_avg_response_time(response_time);
                                        }
                                        debug!(request_id = %request_id, "Successfully parsed JSON data from {}", url);
                                        return Ok(data);
                                    }
                                    Err(e) => {
                                        error!(request_id = %request_id, "Failed to parse JSON response from {}: {}", url, e);
                                        error!(request_id = %request_id, "Response text (first 1000 chars): {}", &text[..std::cmp::min(1000, text.len())]);
                                        last_error = Some(PolymarketError::deserialization_error(format!("JSON parsing error: {} - Response: {}", e, &text[..std::cmp::min(200, text.len())])));
                                    }
                                }
                            }
                            Err(e) => {
                                error!(request_id = %request_id, "Failed to read response text from {}: {}", url, e);
                                last_error = Some(PolymarketError::network_error(format!("Response reading error: {}", e)));
                            }
                        }
                    } else {
                        let status = response.status();
                        let text = response.text().await.unwrap_or_default();
                        error!(request_id = %request_id, "HTTP error {} from {}: {}", status, url, text);
                        last_error = Some(PolymarketError::api_error(format!("HTTP error: {}", text), Some(status.as_u16())));
                    }
                }
                Err(e) => {
                    warn!(request_id = %request_id, "Request attempt {} failed for {}: {}", attempt, url, e);
                    last_error = Some(PolymarketError::network_error(format!("Request error: {}", e)));
                }
            }

            if attempt < max_retries {
                let base_delay = self.config.retry_delay();
                let delay = Duration::from_millis(base_delay.as_millis() as u64 * (1 << attempt));
                debug!("Retrying in {:?}...", delay);
                tokio::time::sleep(delay).await;
            }
        }

        {
            let mut metrics = self.metrics.write().await;
            metrics.increment_api_failures();
        }
        
        let error = last_error.unwrap_or_else(|| PolymarketError::network_error("All retry attempts failed"));
        error.log_error();
        Err(error)
    }

    pub async fn get_markets(&self, params: Option<MarketsQueryParams>) -> Result<Vec<Market>> {
        let query_params = params.unwrap_or_default();
        let cache_key = format!("markets_{}", serde_json::to_string(&query_params).map_err(|e| PolymarketError::deserialization_error(format!("Failed to serialize query params: {}", e)))?);

        if self.config.cache.enabled {
            let cache = self.market_cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if !entry.is_expired(self.config.cache_ttl()) {
                    {
                        let mut metrics = self.metrics.write().await;
                        metrics.increment_cache_hits();
                    }
                    debug!("Returning cached markets data");
                    return Ok(entry.data.clone());
                }
            }
        }
        
        {
            let mut metrics = self.metrics.write().await;
            metrics.increment_cache_misses();
        }

        let query_string = query_params.to_query_string();
        let url = format!("{}/markets{}", self.base_url, query_string);
        
        info!("Fetching markets from: {}", url);

        let response: Vec<Market> = self.make_request_with_retry(&url).await?;
        
        if self.config.cache.enabled {
            let mut cache = self.market_cache.write().await;
            cache.insert(cache_key, CacheEntry::new(response.clone()));
        }

        info!("Successfully fetched {} markets", response.len());
        Ok(response)
    }

    pub async fn get_market_by_id(&self, market_id: &str) -> Result<Market> {
        let cache_key = market_id.to_string();

        if self.config.cache.enabled {
            let cache = self.single_market_cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if !entry.is_expired(self.config.cache_ttl()) {
                    {
                        let mut metrics = self.metrics.write().await;
                        metrics.increment_cache_hits();
                    }
                    debug!("Returning cached market data for {}", market_id);
                    return Ok(entry.data.clone());
                }
            }
        }
        
        {
            let mut metrics = self.metrics.write().await;
            metrics.increment_cache_misses();
        }

        let url = format!("{}/markets/{}", self.base_url, market_id);
        info!("Fetching market details from: {}", url);

        let market: Market = self.make_request_with_retry(&url).await?;
        
        if self.config.cache.enabled {
            let mut cache = self.single_market_cache.write().await;
            cache.insert(cache_key, CacheEntry::new(market.clone()));
        }

        info!("Successfully fetched market: {}", market.question);
        Ok(market)
    }

    pub async fn search_markets(&self, keyword: &str, limit: Option<u32>) -> Result<Vec<Market>> {
        let params = MarketsQueryParams {
            limit: limit.or(Some(20)),
            ..Default::default()
        };
        
        let markets = self.get_markets(Some(params)).await?;
        
        let keyword_lower = keyword.to_lowercase();
        let filtered: Vec<Market> = markets
            .into_iter()
            .filter(|market| {
                market.question.to_lowercase().contains(&keyword_lower)
                    || market.description.as_ref().is_some_and(|desc| {
                        desc.to_lowercase().contains(&keyword_lower)
                    })
                    || market.category.as_ref().is_some_and(|cat| {
                        cat.to_lowercase().contains(&keyword_lower)
                    })
            })
            .collect();

        info!("Found {} markets matching '{}'", filtered.len(), keyword);
        Ok(filtered)
    }

    pub async fn get_market_prices(&self, market_id: &str) -> Result<Vec<MarketPrice>> {
        let market = self.get_market_by_id(market_id).await?;
        let mut prices = Vec::new();

        for (i, _outcome) in market.outcomes.iter().enumerate() {
            if let Some(price_str) = market.outcome_prices.get(i) {
                if let Ok(price) = price_str.parse::<f64>() {
                    prices.push(MarketPrice {
                        market_id: market_id.to_string(),
                        outcome_id: format!("outcome_{}", i),
                        price,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    });
                }
            }
        }

        Ok(prices)
    }

    pub async fn get_trending_markets(&self, limit: Option<u32>) -> Result<Vec<Market>> {
        let params = MarketsQueryParams {
            limit: limit.or(Some(10)),
            order: Some("volume".to_string()),
            ascending: Some(false),
            active: Some(true),
            ..Default::default()
        };

        self.get_markets(Some(params)).await
    }

    pub async fn get_active_markets(&self, limit: Option<u32>) -> Result<Vec<Market>> {
        let params = MarketsQueryParams {
            limit: limit.or(Some(50)),
            active: Some(true),
            archived: Some(false),
            ..Default::default()
        };

        self.get_markets(Some(params)).await
    }

    #[allow(dead_code)]
    pub async fn get_markets_batch(&self, market_ids: Vec<String>) -> Result<Vec<Market>> {
        if market_ids.is_empty() {
            return Ok(Vec::new());
        }

        const BATCH_SIZE: usize = 10;
        let mut all_markets = Vec::new();
        
        for chunk in market_ids.chunks(BATCH_SIZE) {
            let futures = chunk.iter().map(|id| self.get_market_by_id(id));
            let results = future::join_all(futures).await;
            
            for result in results {
                match result {
                    Ok(market) => all_markets.push(market),
                    Err(e) => {
                        warn!("Failed to fetch market in batch: {}", e);
                    }
                }
            }
            
            if chunk.len() == BATCH_SIZE {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        
        Ok(all_markets)
    }

    #[allow(dead_code)]
    pub async fn get_metrics(&self) -> Metrics {
        self.metrics.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn create_test_config() -> Arc<Config> {
        let mut config = Config::default();
        config.api.base_url = "http://localhost:3000".to_string();
        config.cache.enabled = false;
        Arc::new(config)
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = create_test_config();
        let client = PolymarketClient::new_with_config(&config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let config = create_test_config();
        let client = PolymarketClient::new_with_config(&config).unwrap();
        
        let initial_metrics = client.get_metrics().await;
        assert_eq!(initial_metrics.api_requests_total, 0);
        assert_eq!(initial_metrics.cache_hits, 0);
    }

    #[test]
    fn test_cache_entry_expiration() {
        let entry = CacheEntry::new("test_data".to_string());
        
        assert!(!entry.is_expired(Duration::from_secs(1)));
        
        std::thread::sleep(Duration::from_millis(10));
        assert!(!entry.is_expired(Duration::from_secs(1)));
        assert!(entry.is_expired(Duration::from_millis(5)));
    }
}