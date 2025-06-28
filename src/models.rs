use serde::{Deserialize, Deserializer, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub id: String,
    pub slug: String,
    pub question: String,
    pub description: Option<String>,
    pub active: bool,
    pub closed: bool,

    // Polymarket returns these as strings, we'll parse them
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub liquidity: f64,
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub volume: f64,

    #[serde(rename = "endDate")]
    pub end_date: String,

    pub image: Option<String>,
    pub category: Option<String>,

    // These are JSON strings in the API
    #[serde(deserialize_with = "deserialize_json_string_to_vec")]
    pub outcomes: Vec<String>,
    #[serde(
        rename = "outcomePrices",
        deserialize_with = "deserialize_json_string_to_vec"
    )]
    pub outcome_prices: Vec<String>,

    #[serde(rename = "conditionId")]
    pub condition_id: Option<String>,
    #[serde(rename = "marketType")]
    pub market_type: Option<String>,
    #[serde(rename = "twitterCardImage")]
    pub twitter_card_image: Option<String>,
    pub icon: Option<String>,

    // Optional fields that might not always be present
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(
        rename = "volume24hr",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub volume_24hr: Option<f64>,
    pub events: Option<Vec<Event>>,

    // Additional optional fields that might be present
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(rename = "enableOrderBook", default)]
    pub enable_order_book: Option<bool>,
    #[serde(rename = "groupItemTitle", default)]
    pub group_item_title: Option<String>,
    #[serde(rename = "groupItemSlug", default)]
    pub group_item_slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPrice {
    pub market_id: String,
    pub outcome_id: String,
    pub price: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub ticker: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
    pub image: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(
        deserialize_with = "deserialize_optional_string_or_number_to_f64",
        default
    )]
    pub volume: Option<f64>,

    // Additional fields that might be present
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResponse {
    pub data: Vec<Event>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub market_id: String,
    pub user_address: String,
    pub outcome_id: String,
    pub shares: f64,
    pub value: f64,
    pub cost_basis: f64,
    pub unrealized_pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionsResponse {
    pub data: Vec<Position>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub market_id: String,
    pub outcome_id: String,
    pub side: String, // "buy" or "sell"
    pub size: f64,
    pub price: f64,
    pub timestamp: String,
    pub trader_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesResponse {
    pub data: Vec<Trade>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub market_id: String,
    pub outcome_id: String,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStats {
    pub market_id: String,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub liquidity: f64,
    pub num_traders: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub status_code: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketsQueryParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub order: Option<String>,
    pub ascending: Option<bool>,
    pub active: Option<bool>,
    pub closed: Option<bool>,
    pub archived: Option<bool>,
    pub liquidity_num_min: Option<f64>,
    pub liquidity_num_max: Option<f64>,
    pub volume_num_min: Option<f64>,
    pub volume_num_max: Option<f64>,
    pub start_date_min: Option<String>,
    pub start_date_max: Option<String>,
    pub end_date_min: Option<String>,
    pub end_date_max: Option<String>,
    pub tag_id: Option<String>,
    pub related_tags: Option<bool>,
}

impl Default for MarketsQueryParams {
    fn default() -> Self {
        Self {
            limit: Some(20),
            offset: Some(0),
            order: Some("liquidity".to_string()),
            ascending: Some(false),
            active: Some(true),
            closed: None,
            archived: Some(false),
            liquidity_num_min: None,
            liquidity_num_max: None,
            volume_num_min: None,
            volume_num_max: None,
            start_date_min: None,
            start_date_max: None,
            end_date_min: None,
            end_date_max: None,
            tag_id: None,
            related_tags: None,
        }
    }
}

impl MarketsQueryParams {
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();

        if let Some(limit) = self.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = self.offset {
            params.push(format!("offset={}", offset));
        }
        if let Some(ref order) = self.order {
            params.push(format!("order={}", order));
        }
        if let Some(ascending) = self.ascending {
            params.push(format!("ascending={}", ascending));
        }
        if let Some(active) = self.active {
            params.push(format!("active={}", active));
        }
        if let Some(closed) = self.closed {
            params.push(format!("closed={}", closed));
        }
        if let Some(archived) = self.archived {
            params.push(format!("archived={}", archived));
        }
        if let Some(liquidity_min) = self.liquidity_num_min {
            params.push(format!("liquidity_num_min={}", liquidity_min));
        }
        if let Some(liquidity_max) = self.liquidity_num_max {
            params.push(format!("liquidity_num_max={}", liquidity_max));
        }
        if let Some(volume_min) = self.volume_num_min {
            params.push(format!("volume_num_min={}", volume_min));
        }
        if let Some(volume_max) = self.volume_num_max {
            params.push(format!("volume_num_max={}", volume_max));
        }
        if let Some(ref start_min) = self.start_date_min {
            params.push(format!("start_date_min={}", start_min));
        }
        if let Some(ref start_max) = self.start_date_max {
            params.push(format!("start_date_max={}", start_max));
        }
        if let Some(ref end_min) = self.end_date_min {
            params.push(format!("end_date_min={}", end_min));
        }
        if let Some(ref end_max) = self.end_date_max {
            params.push(format!("end_date_max={}", end_max));
        }
        if let Some(ref tag_id) = self.tag_id {
            params.push(format!("tag_id={}", tag_id));
        }
        if let Some(related_tags) = self.related_tags {
            params.push(format!("related_tags={}", related_tags));
        }

        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceContent {
    pub uri: String,
    pub mime_type: String,
    pub text: Option<String>,
    pub blob: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptMessage {
    pub role: String,
    pub content: McpPromptContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpPromptContent {
    Text(String),
    Image { r#type: String, data: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCache {
    pub data: String,
    pub timestamp: u64,
    pub expires_at: u64,
}

impl ResourceCache {
    pub fn new(data: String, ttl_seconds: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            data,
            timestamp: now,
            expires_at: now + ttl_seconds,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now > self.expires_at
    }
}

// Custom deserializers for Polymarket API format
fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

fn deserialize_json_string_to_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

fn deserialize_optional_string_or_number_to_f64<'de, D>(
    deserializer: D,
) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    // First try to deserialize as an optional value
    match Option::<Value>::deserialize(deserializer) {
        Ok(Some(Value::String(s))) => s.parse::<f64>().map(Some).map_err(serde::de::Error::custom),
        Ok(Some(Value::Number(n))) => Ok(n.as_f64()),
        Ok(Some(_)) => Err(serde::de::Error::custom("Expected string or number")),
        Ok(None) => Ok(None),
        Err(_) => Ok(None), // If field is missing, return None
    }
}
