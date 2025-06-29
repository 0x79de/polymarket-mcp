# Polymarket MCP Server

A high-performance Model Context Protocol (MCP) server for Polymarket prediction market data, built with Rust. This server provides real-time market data, prices, and betting information from Polymarket through MCP tools, resources, and prompts.

[![CI](https://github.com/0x79de/polymarket-mcp/workflows/CI/badge.svg)](https://github.com/0x79de/polymarket-mcp/actions)
[![Release](https://img.shields.io/github/v/release/0x79de/polymarket-mcp)](https://github.com/0x79de/polymarket-mcp/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **🔄 Real-time Market Data**: Fetch active markets, trending markets, and detailed market information
- **🔍 Advanced Search**: Search markets by keywords across questions, descriptions, and categories  
- **💰 Price Information**: Get current yes/no prices and market statistics
- **📊 MCP Resources**: Auto-refreshing market data resources with intelligent caching
- **🤖 AI-Powered Prompts**: Market analysis, arbitrage detection, and trading insights
- **⚡ High Performance**: Built-in caching, connection pooling, and optimized for speed
- **🔧 Flexible Configuration**: Environment variables, TOML files, or defaults
- **🏗️ Production Ready**: Zero compilation warnings, comprehensive error handling, full test coverage

## Quick Start

### Prerequisites
- **Rust 1.70+** - Install via [rustup](https://rustup.rs/)
- **Claude Desktop** - Download from [Claude.ai](https://claude.ai/download)
- **Internet Connection** - For Polymarket API access (no API key required)

### Installation

#### Option 1: Download Pre-built Binary (Recommended)

1. **Download the latest release** for your platform:
   - Visit [Releases](https://github.com/0x79de/polymarket-mcp/releases)
   - Download the appropriate binary for your OS

2. **Configure Claude Desktop:**
   
   Edit your Claude Desktop configuration file:
   - **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
   - **Linux**: `~/.config/Claude/claude_desktop_config.json`

   Add the Polymarket MCP server:
   ```json
   {
     "mcpServers": {
       "polymarket": {
         "command": "/Users/0x79de/dev/polymarket-mcp/target/release/polymarket-mcp",
         "env": {
           "RUST_LOG": "info"
         }
       }
     }
   }
   ```

3. **Restart Claude Desktop** to load the new MCP server.

#### Option 2: Build from Source

1. **Clone and build:**
   ```bash
   git clone https://github.com/0x79de/polymarket-mcp
   cd polymarket-mcp
   cargo build --release
   ```

2. **Use the binary at** `target/release/polymarket-mcp` in your Claude Desktop configuration.

#### Option 3: Install via Cargo

```bash
cargo install --git https://github.com/0x79de/polymarket-mcp
```

The binary will be installed to `~/.cargo/bin/polymarket-mcp`.

## Configuration (Optional)

The server works out-of-the-box with sensible defaults. No configuration is required for basic usage.

### Environment Variables

Create a `.env` file for custom configuration:

```bash
# API Configuration
POLYMARKET_API_BASE_URL=https://gamma-api.polymarket.com
# POLYMARKET_API_KEY=your_key_here  # Optional - not needed for public data

# Performance Settings
POLYMARKET_CACHE_ENABLED=true
POLYMARKET_CACHE_TTL=60              # Cache TTL in seconds
POLYMARKET_RESOURCE_CACHE_TTL=300    # Resource cache TTL

# Logging
POLYMARKET_LOG_LEVEL=info            # trace, debug, info, warn, error
RUST_LOG=info                        # Alternative log level setting

# Advanced Settings (rarely needed)
POLYMARKET_API_TIMEOUT=30            # API timeout in seconds
POLYMARKET_API_MAX_RETRIES=3         # Retry attempts
POLYMARKET_API_RETRY_DELAY=100       # Retry delay in ms
```

### Configuration File

Alternatively, copy `config.toml.example` to `config.toml` and customize:

```toml
[server]
name = "Polymarket MCP Server"
timeout_seconds = 30

[api]
base_url = "https://gamma-api.polymarket.com"
# api_key = "your_key_here"  # Optional
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

### Configuration Priority

Configuration is loaded in this order (highest to lowest priority):
1. **Environment variables** (e.g., `POLYMARKET_LOG_LEVEL=debug`)
2. **Configuration file** (`config.toml`, `polymarket-mcp.toml`, etc.)
3. **Built-in defaults** (production-ready settings)

## Usage

### Claude Desktop Integration

After installing and configuring the MCP server, you can interact with Polymarket data directly through Claude Desktop. Try these example prompts:

```
🔍 Market Discovery:
- "Show me the top 10 active prediction markets"
- "Search for markets about 'AI' or 'artificial intelligence'"
- "What are the trending markets with highest volume?"

📊 Market Analysis:
- "Get details for market ID 12345"
- "What are the current prices for the Trump 2024 market?"
- "Analyze the sentiment of markets about cryptocurrency"

🤖 AI-Powered Insights:
- "Find arbitrage opportunities in election markets"
- "Give me a summary of the top 5 political prediction markets"
- "Analyze market 67890 for trading opportunities"
```

### Example Usage Flow

1. **Ask Claude**: "Show me active prediction markets about AI"
2. **Claude uses**: `search_markets` tool with keyword "AI"
3. **You get**: List of AI-related markets with prices and details
4. **Follow up**: "Analyze the most liquid AI market"
5. **Claude uses**: `analyze_market` prompt for deep insights

### Development & Testing

```bash
# Quick test (requires Rust)
cargo run

# With debug logging
RUST_LOG=debug cargo run

# Run all tests
cargo test

# Check code quality
cargo clippy

# Test MCP protocol manually
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}},"id":0}' | cargo run
```

## MCP Protocol Implementation

This server implements the full MCP specification with **5 tools**, **3 resources**, and **3 prompts**.

### 🔧 MCP Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `get_active_markets` | Fetch currently active prediction markets | `limit` (optional, default: 50) |
| `get_market_details` | Get detailed information about a specific market | `market_id` (required) |
| `search_markets` | Search markets by keyword in questions/descriptions | `keyword` (required), `limit` (optional, default: 20) |
| `get_market_prices` | Get current yes/no prices for a market | `market_id` (required) |
| `get_trending_markets` | Get markets with highest trading volume | `limit` (optional, default: 10) |

### 📊 MCP Resources

Auto-refreshing data resources that Claude can access:

| Resource | Description | Refresh Rate |
|----------|-------------|--------------|
| `markets:active` | List of currently active markets | Every 5 minutes |
| `markets:trending` | Markets sorted by trading volume | Every 5 minutes |
| `market:{id}` | Specific market details by ID | Every 5 minutes |

### 🤖 MCP Prompts

AI-powered analysis prompts for intelligent market insights:

| Prompt | Description | Arguments |
|--------|-------------|-----------|
| `analyze_market` | Comprehensive market analysis with trading insights | `market_id` (required) |
| `find_arbitrage` | Detect arbitrage opportunities across related markets | `keyword` (required), `limit` (optional, default: 10) |
| `market_summary` | Overview of top markets with recommendations | `category` (optional), `limit` (optional, default: 5) |

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

### Quality Assurance

This project maintains high code quality standards:

```bash
# Run all tests (11 tests total)
cargo test

# Check for compilation warnings (should be zero)
cargo check

# Run clippy lints (should pass clean)
cargo clippy --all-targets --all-features -- -D warnings -A clippy::pedantic

# Format code
cargo fmt

# Build optimized release
cargo build --release
```

**Current Status:**
- ✅ **Zero compilation warnings**
- ✅ **All tests passing** (11/11)
- ✅ **Clean clippy lints**
- ✅ **100% API coverage**

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

We welcome contributions! Please follow these steps:

1. **Fork the repository** and create a feature branch
2. **Make your changes** with proper tests
3. **Ensure quality standards**:
   ```bash
   cargo test                    # All tests must pass
   cargo clippy --all-targets --all-features -- -D warnings -A clippy::pedantic
   cargo fmt --check            # Code must be formatted
   cargo check                  # No compilation warnings
   ```
4. **Submit a pull request** with a clear description

### Development Guidelines
- **Zero warnings policy**: All code must compile without warnings
- **Test coverage**: New features must include tests
- **Documentation**: Update README for user-facing changes
- **Performance**: Consider caching and efficiency
- **Security**: No API keys in code, secure error handling

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/0x79de/polymarket-mcp/issues)
- **Documentation**: [MCP Specification](https://modelcontextprotocol.io/docs)
- **Polymarket API**: [Official Documentation](https://docs.polymarket.com)

## Changelog

### v0.3.0 (Current)

- **🎯 Zero Warnings**: Completely clean compilation with zero warnings
- **📊 Full MCP Implementation**: 5 tools, 3 resources, 3 prompts
- **⚡ Performance Optimized**: Connection pooling, intelligent caching, retry logic
- **🧪 Comprehensive Testing**: 11 tests covering all functionality
- **📚 Enhanced Documentation**: Updated README with examples and installation options
- **🔧 Production Ready**: Robust error handling and configuration management

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