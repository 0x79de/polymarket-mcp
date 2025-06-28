# Polymarket MCP Server

A Model Context Protocol (MCP) server for Polymarket prediction market data, built with Rust and the official MCP Rust SDK. This server provides real-time market data, prices, and betting information from Polymarket through MCP tools, resources, and prompts.

## Features

- **Real-time Market Data**: Fetch active markets, trending markets, and market details
- **Market Search**: Search markets by keywords across questions, descriptions, and categories
- **Price Information**: Get current yes/no prices for any market
- **MCP Resources**: Auto-refreshing market data resources
- **MCP Prompts**: AI-powered market analysis and arbitrage detection
- **Caching**: Built-in caching with configurable TTL for performance
- **Configuration**: Flexible configuration via environment variables or TOML files
- **Clean Architecture**: Optimized codebase with minimal dependencies

## Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Polymarket API credentials (optional for basic read-only access)

## Installation

### For Claude Desktop

1. **Clone the repository:**
   ```bash
   git clone https://github.com/0x79de/polymarket-mcp
   cd polymarket-mcp
   ```

2. **Build the release binary:**
   ```bash
   cargo build --release
   ```

3. **Configure Claude Desktop:**
   
   Edit your Claude Desktop configuration file at:
   - **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
   - **Linux**: `~/.config/Claude/claude_desktop_config.json`

   Add the Polymarket MCP server:
   ```json
   {
     "mcpServers": {
       "polymarket": {
         "command": "/full/path/to/polymarket-mcp/target/release/polymarket-mcp",
         "env": {
           "RUST_LOG": "info"
         }
       }
     }
   }
   ```

4. **Restart Claude Desktop** to load the new MCP server.

### Optional Configuration

Set up environment variables for enhanced functionality:
```bash
# Create a .env file for optional configuration
echo "POLYMARKET_API_BASE_URL=https://gamma-api.polymarket.com" > .env
echo "POLYMARKET_CACHE_ENABLED=true" >> .env
echo "POLYMARKET_CACHE_TTL=60" >> .env
echo "POLYMARKET_LOG_LEVEL=info" >> .env
```

## Configuration

### Environment Variables

Create a `.env` file and configure as needed:

```bash
# API Configuration
POLYMARKET_API_BASE_URL=https://gamma-api.polymarket.com
POLYMARKET_API_KEY=your_api_key_here  # Optional for basic usage

# Server Settings
POLYMARKET_CACHE_ENABLED=true
POLYMARKET_CACHE_TTL=60
POLYMARKET_RESOURCE_CACHE_TTL=300
POLYMARKET_LOG_LEVEL=info

# Advanced Settings
POLYMARKET_API_TIMEOUT=30
POLYMARKET_API_MAX_RETRIES=3
POLYMARKET_API_RETRY_DELAY=100
```

### Configuration File

Alternatively, create a `config.toml` file:

```toml
[server]
name = "Polymarket MCP Server"
timeout_seconds = 30

[api]
base_url = "https://gamma-api.polymarket.com"
api_key = "your_api_key_here"  # Optional
timeout_seconds = 30
max_retries = 3
retry_delay_ms = 100

[cache]
enabled = true
ttl_seconds = 60
max_entries = 1000
resource_cache_ttl_seconds = 300

[logging]
level = "info"
format = "pretty"
enable_colors = true
```

## Usage

### Claude Desktop Integration

After configuring your `claude_desktop_config.json` and restarting Claude Desktop, you can interact with Polymarket data directly through Claude. Example prompts:

- "Show me the top 5 active prediction markets"
- "Search for markets about 'election'"
- "Get details for market ID 0x123..."
- "What are the current prices for this market?"
- "Analyze the sentiment of the AI prediction market"

### Building for Production

```bash
# Build release version for Claude Desktop
cargo build --release

# The binary will be at: target/release/polymarket-mcp
```

### Testing the Server (Development)

```bash
# Run with default configuration
cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Test basic functionality
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}},"id":0}' | cargo run
```

## MCP Tools

The server provides the following MCP tools:

### `get_active_markets`
Fetch currently active prediction markets.

**Parameters:**
- `limit` (optional): Maximum number of markets to return (default: 50)

### `get_market_details`
Get detailed information about a specific market.

**Parameters:**
- `market_id` (required): The ID of the market

### `search_markets`
Search markets by keyword.

**Parameters:**
- `keyword` (required): Search term
- `limit` (optional): Maximum results (default: 20)

### `get_market_prices`
Get current yes/no prices for a market.

**Parameters:**
- `market_id` (required): The ID of the market

### `get_trending_markets`
Get markets with highest trading volume.

**Parameters:**
- `limit` (optional): Maximum number of markets (default: 10)

## MCP Resources

Auto-refreshing data resources:

- `markets:active` - List of active markets (refreshes every 5 minutes)
- `markets:trending` - Trending markets by volume (refreshes every 5 minutes)
- `market:{id}` - Specific market details (refreshes every 5 minutes)

## MCP Prompts

AI-powered analysis prompts:

### `analyze_market`
Comprehensive market analysis including sentiment, liquidity, and trading opportunities.

**Arguments:**
- `market_id` (required): Market to analyze

### `find_arbitrage`
Detect arbitrage opportunities across related markets.

**Arguments:**
- `keyword` (required): Search term for related markets
- `limit` (optional): Number of markets to analyze (default: 10)

### `market_summary`
Overview of top prediction markets with trading recommendations.

**Arguments:**
- `category` (optional): Filter by category
- `limit` (optional): Number of markets to include (default: 5)

## API Documentation

### Market Object Structure

```json
{
  "id": "0x123...",
  "slug": "market-slug",
  "question": "Will X happen by Y date?",
  "description": "Detailed market description",
  "active": true,
  "closed": false,
  "liquidity": 50000.0,
  "volume": 125000.0,
  "end_date": "2024-12-31T23:59:59Z",
  "outcomes": ["Yes", "No"],
  "outcome_prices": ["0.65", "0.35"],
  "category": "Politics"
}
```

### Error Handling

The server implements robust error handling:

- **Network Errors**: Automatic retry with exponential backoff and jitter
- **Rate Limiting**: Automatic delays for rate-limited requests
- **Data Validation**: All API responses are validated and parsed safely
- **Caching**: Prevents redundant API calls and improves performance

## Development

### Code Structure

```
src/
├── main.rs              # MCP server implementation and request handling
├── lib.rs               # Library exports
├── config.rs            # Configuration management with env/file support
├── models.rs            # Data structures and Polymarket API types
├── polymarket_client.rs # HTTP client with caching and retry logic
└── error.rs             # Error types and handling
```

### Key Features of the Implementation

- **Clean Architecture**: Minimal dependencies, focused functionality
- **Type Safety**: Comprehensive Rust type system usage
- **Performance**: Efficient caching and HTTP connection pooling
- **Reliability**: Robust error handling and retry logic
- **MCP Compliance**: Full implementation of MCP protocol specification

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Check for warnings and errors
cargo check
```

### Dependencies

The project uses minimal, focused dependencies:

```toml
# Core MCP and async runtime
rmcp = { git = "https://github.com/modelcontextprotocol/rust-sdk" }
tokio = { version = "1.0", features = ["rt-multi-thread", "macros", "sync", "time", "signal", "io-std"] }

# HTTP client and serialization
reqwest = { version = "0.12", features = ["json", "gzip", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Configuration and utilities
config = "0.14"
dotenv = "0.15"
clap = { version = "4.0", features = ["derive"] }
fastrand = "2.0"  # For request jitter

# Date/time and errors
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
thiserror = "1.0"
uuid = { version = "1.0", features = ["v4"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

## Deployment

### Systemd Service (Linux)

For Linux servers, create `/etc/systemd/system/polymarket-mcp.service`:

```ini
[Unit]
Description=Polymarket MCP Server
After=network.target

[Service]
Type=simple
User=polymarket
ExecStart=/usr/local/bin/polymarket-mcp
Restart=always
RestartSec=10
Environment=RUST_LOG=info
EnvironmentFile=/etc/polymarket-mcp/.env

[Install]
WantedBy=multi-user.target
```

## Troubleshooting

### Common Issues

#### Server Won't Start
**Check these steps:**
1. Verify binary path: `ls -la /path/to/target/release/polymarket-mcp`
2. Test binary: `./target/release/polymarket-mcp --help`
3. Check configuration: `RUST_LOG=debug cargo run`

#### MCP Connection Issues
**Troubleshooting:**
1. Verify `claude_desktop_config.json` syntax is valid JSON
2. Check absolute path to binary in configuration
3. Restart Claude Desktop after configuration changes
4. Check Claude Desktop logs: `~/Library/Logs/Claude/mcp.log` (macOS)

#### API Errors
**Common solutions:**
- Verify internet connectivity
- Check API endpoint availability
- Ensure no rate limiting (automatic handling built-in)
- Review error logs with `RUST_LOG=debug`

### Debug Mode

Enable debug logging for detailed output:

```json
{
  "mcpServers": {
    "polymarket": {
      "command": "/path/to/polymarket-mcp/target/release/polymarket-mcp",
      "env": {
        "RUST_LOG": "debug"
      }
    }
  }
}
```

### Log Locations

- **Claude Desktop MCP Logs**: `~/Library/Logs/Claude/mcp.log` (macOS)
- **Claude Desktop Main Logs**: `~/Library/Logs/Claude/main.log` (macOS)
- **Server Output**: All logs go to stderr, visible in Claude Desktop MCP logs

## Performance

The server is optimized for performance:

- **Caching**: Intelligent caching with configurable TTL
- **Connection Pooling**: Reuses HTTP connections efficiently
- **Minimal Allocations**: Efficient memory usage patterns
- **Async Processing**: Non-blocking I/O throughout
- **Fast JSON**: Optimized serialization/deserialization

## Security

- API keys are handled securely and never logged
- All HTTP connections use TLS encryption
- Input validation prevents injection attacks
- No sensitive data is cached or logged
- Minimal attack surface with focused dependencies

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and add tests
4. Run tests: `cargo test`
5. Check code: `cargo check`
6. Format code: `cargo fmt`
7. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/0x79de/polymarket-mcp/issues)
- **Documentation**: [MCP Specification](https://modelcontextprotocol.io/docs)
- **Polymarket API**: [Official Documentation](https://docs.polymarket.com)

## Changelog

### v0.2.0 (Current)

- **Optimized**: Removed all unused code and dependencies
- **Performance**: Eliminated metrics system for better performance
- **Clean**: Zero warnings during compilation
- **Simplified**: Streamlined error handling and client architecture
- **Dependencies**: Reduced to essential dependencies only
- **Reliability**: Improved stability with focused codebase

### v0.1.1

- **Fixed**: JSON parsing errors in Claude Desktop by redirecting logs to stderr
- **Fixed**: MCP validation errors with notification handling
- **Fixed**: Added `io-std` feature to tokio for stdin/stdout support
- **Improved**: Proper JSON-RPC protocol compliance
- **Added**: Comprehensive troubleshooting documentation

### v0.1.0

- Initial release with core MCP functionality
- Support for market data fetching and searching
- MCP resources and prompts implementation
- Comprehensive configuration system
- Built-in caching and error handling