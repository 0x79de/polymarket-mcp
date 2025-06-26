# Polymarket MCP Server

A Model Context Protocol (MCP) server for Polymarket prediction market data, built with Rust and the official MCP Rust SDK. This server provides real-time market data, prices, and betting information from Polymarket through MCP tools, resources, and prompts.

## Features

- **Real-time Market Data**: Fetch active markets, trending markets, and market details
- **Market Search**: Search markets by keywords across questions, 
descriptions, and categories
- **Price Information**: Get current yes/no prices for any market
- **MCP Resources**: Auto-refreshing market data resources
- **MCP Prompts**: AI-powered market analysis and arbitrage detection
- **Caching**: Built-in caching with configurable TTL for performance
- **Configuration**: Flexible configuration via environment variables or TOML files

## Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Polymarket API credentials (optional for basic read-only access)

## Installation

### For Claude Desktop

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-username/polymarket-mcp
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
cp .env.example .env
# Edit .env with your API credentials (optional for basic usage)
```

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and configure as needed:

```bash
# API Configuration
POLYMARKET_API_KEY=your_api_key_here
POLYMARKET_API_BASE_URL=https://gamma-api.polymarket.com

# Trading API Credentials (for advanced operations)
POLYMARKET_SECRET=your_secret_here
POLYMARKET_PASSPHRASE=your_passphrase_here

# Wallet Credentials (for blockchain interactions)
WALLET_PUBLIC_KEY=0x_your_public_key_here
WALLET_PRIVATE_KEY=your_private_key_here

# Server Settings
POLYMARKET_CACHE_ENABLED=true
POLYMARKET_CACHE_TTL=60
POLYMARKET_LOG_LEVEL=info
```

### Configuration File

Alternatively, create a `config.toml` file:

```toml
[server]
name = "Polymarket MCP Server"
max_connections = 100
timeout_seconds = 30

[api]
base_url = "https://gamma-api.polymarket.com"
api_key = "your_api_key_here"
timeout_seconds = 30
max_retries = 3

[cache]
enabled = true
ttl_seconds = 60
max_entries = 1000

[logging]
level = "info"
format = "pretty"
enable_colors = true
```

## Usage

### Claude Desktop Integration

To use with Claude Desktop, configure your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "polymarket": {
      "command": "/path/to/polymarket-mcp/target/release/polymarket-mcp",
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

**Configuration file location:**
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

### Building for Production

```bash
# Build release version for Claude Desktop
cargo build --release

# The binary will be at: target/release/polymarket-mcp
```

### Standalone Server (Development)

```bash
# Run with default configuration (development mode)
cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Test the server manually (not recommended for production)
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}},"id":0}' | cargo run
```

### MCP Tools

The server provides the following MCP tools:

#### `get_active_markets`
Fetch currently active prediction markets.

**Parameters:**
- `limit` (optional): Maximum number of markets to return (default: 50)

**Example:**
```json
{
  "tool": "get_active_markets",
  "arguments": {
    "limit": 10
  }
}
```

#### `get_market_details`
Get detailed information about a specific market.

**Parameters:**
- `market_id` (required): The ID of the market

**Example:**
```json
{
  "tool": "get_market_details",
  "arguments": {
    "market_id": "0x123..."
  }
}
```

#### `search_markets`
Search markets by keyword.

**Parameters:**
- `keyword` (required): Search term
- `limit` (optional): Maximum results (default: 20)

**Example:**
```json
{
  "tool": "search_markets",
  "arguments": {
    "keyword": "election",
    "limit": 5
  }
}
```

#### `get_market_prices`
Get current yes/no prices for a market.

**Parameters:**
- `market_id` (required): The ID of the market

#### `get_trending_markets`
Get markets with highest trading volume.

**Parameters:**
- `limit` (optional): Maximum number of markets (default: 10)

### MCP Resources

Auto-refreshing data resources:

- `markets:active` - List of active markets
- `markets:trending` - Trending markets by volume
- `market:{id}` - Specific market details

### MCP Prompts

AI-powered analysis prompts:

#### `analyze_market`
Comprehensive market analysis including sentiment, liquidity, and trading opportunities.

**Arguments:**
- `market_id` (required): Market to analyze

#### `find_arbitrage`
Detect arbitrage opportunities across related markets.

**Arguments:**
- `keyword` (required): Search term for related markets
- `limit` (optional): Number of markets to analyze

#### `market_summary`
Overview of top prediction markets with trading recommendations.

**Arguments:**
- `category` (optional): Filter by category
- `limit` (optional): Number of markets to include

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

The server implements comprehensive error handling:

- **Network Errors**: Automatic retry with exponential backoff
- **Rate Limiting**: Built-in rate limiting with configurable limits
- **Data Validation**: All API responses are validated
- **Caching**: Prevents redundant API calls

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_market_client
```

### Code Structure

```
src/
├── main.rs           # MCP server implementation
├── config.rs         # Configuration management
├── models.rs         # Data structures and types
└── polymarket_client.rs # Polymarket API client
```

### Adding New Features

1. **New Tools**: Add methods to `PolymarketMcpServer` in `main.rs`
2. **New Models**: Define structures in `models.rs`
3. **API Endpoints**: Extend `PolymarketClient` in `polymarket_client.rs`
4. **Configuration**: Add options to `Config` in `config.rs`

## Deployment

### Docker

```bash
# Build Docker image
docker build -t polymarket-mcp .

# Run container
docker run -d \
  --name polymarket-mcp \
  -e POLYMARKET_API_KEY=your_key \
  -p 8080:8080 \
  polymarket-mcp
```

### Systemd Service

Create `/etc/systemd/system/polymarket-mcp.service`:

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

#### JSON Parsing Errors in Claude Desktop Logs
**Error**: `Unexpected token '', "  [2m2025-0"... is not valid JSON`

**Solution**: This indicates logs are being written to stdout instead of stderr. The server has been configured to write all logs to stderr to avoid interfering with MCP JSON communication.

#### MCP Validation Errors
**Error**: `Invalid input`, `Expected string, received null` in `id` field

**Solution**: The server now properly handles JSON-RPC notifications (like `notifications/initialized`) by not responding to them, which is the correct behavior.

#### Server Connection Issues
**Error**: `Server disconnected` or `Server transport closed unexpectedly`

**Troubleshooting steps:**
1. Verify the binary path in your `claude_desktop_config.json`
2. Check that the binary exists: `ls -la /path/to/target/release/polymarket-mcp`
3. Test the binary manually: `echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}},"id":0}' | /path/to/target/release/polymarket-mcp`
4. Check logs in Claude Desktop: `~/Library/Logs/Claude/mcp.log` (macOS)

#### Build Issues
**Error**: `cannot find function 'stdin' in module 'tokio::io'`

**Solution**: Ensure `tokio` has the `io-std` feature enabled in `Cargo.toml`:
```toml
tokio = { version = "1.0", features = ["rt-multi-thread", "macros", "sync", "time", "signal", "io-std"] }
```

### Debug Mode

Enable debug logging to see detailed MCP communication:

```bash
# In your claude_desktop_config.json
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
- **Server stderr output**: Appears in Claude Desktop MCP logs

## Monitoring

The server provides structured logging and can be monitored using:

- **Logs**: JSON or pretty-printed logs with configurable levels (written to stderr)
- **Metrics**: Built-in caching metrics and API call tracking
- **Health**: Connection pooling and retry statistics
- **MCP Protocol**: Full request/response logging in debug mode

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and add tests
4. Run tests: `cargo test`
5. Run lints: `cargo clippy`
6. Format code: `cargo fmt`
7. Submit a pull request

## Security

- API keys are handled securely and never logged
- Private keys are optional and only used for trading operations
- All HTTP connections use TLS
- Input validation prevents injection attacks

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/your-username/polymarket-mcp/issues)
- **Documentation**: [MCP Specification](https://modelcontextprotocol.io/docs)
- **Polymarket API**: [Official Documentation](https://docs.polymarket.com)

## Changelog

### v0.1.1 (Current)

- **Fixed**: JSON parsing errors in Claude Desktop by redirecting logs to stderr
- **Fixed**: MCP validation errors with notification handling
- **Fixed**: Added `io-std` feature to tokio for stdin/stdout support
- **Improved**: Proper JSON-RPC protocol compliance
- **Added**: Comprehensive troubleshooting documentation
- **Added**: Claude Desktop integration guide

### v0.1.0

- Initial release with core MCP functionality
- Support for market data fetching and searching
- MCP resources and prompts implementation
- Comprehensive configuration system
- Built-in caching and error handling