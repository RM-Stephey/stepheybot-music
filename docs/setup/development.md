# 🛠️ StepheyBot Music - Development Setup Guide

> **Complete guide for setting up the StepheyBot Music development environment**

## 📋 Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Detailed Setup](#detailed-setup)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Debugging](#debugging)
- [IDE Configuration](#ide-configuration)
- [Docker Development](#docker-development)
- [Contributing](#contributing)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements
- **OS**: Linux (CachyOS recommended), macOS, or Windows with WSL2
- **RAM**: Minimum 4GB, Recommended 8GB+
- **Storage**: 2GB free space for development dependencies
- **Network**: Internet connection for dependency downloads

### Required Software

#### 1. **Rust Toolchain** (Latest Stable)
```bash
# Install rustup (Rust installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version  # Should be 1.80+
cargo --version
```

#### 2. **SQLite** (Database)
```bash
# CachyOS/Arch Linux
sudo pacman -S sqlite

# Ubuntu/Debian
sudo apt-get install sqlite3 libsqlite3-dev

# macOS
brew install sqlite

# Verify installation
sqlite3 --version
```

#### 3. **Git** (Version Control)
```bash
# Most systems have git pre-installed
git --version

# If not installed:
# CachyOS/Arch: sudo pacman -S git
# Ubuntu/Debian: sudo apt-get install git
# macOS: brew install git
```

#### 4. **Docker** (Optional, for containerized development)
```bash
# Install Docker following official instructions for your OS
# https://docs.docker.com/get-docker/

# Verify installation
docker --version
docker-compose --version
```

### Optional Tools

#### Development Tools
```bash
# Install additional Rust components
rustup component add rustfmt clippy rust-analyzer

# Install cargo tools
cargo install cargo-watch    # File watching for auto-rebuild
cargo install cargo-expand   # Macro expansion debugging
cargo install sqlx-cli       # Database migration tool
```

#### System Tools
```bash
# HTTP testing
curl --version || sudo pacman -S curl  # or apt-get install curl

# JSON processing
jq --version || sudo pacman -S jq      # or apt-get install jq

# Process monitoring
htop --version || sudo pacman -S htop  # or apt-get install htop
```

## Quick Start

### 1. Clone the Repository
```bash
# Clone from GitHub
git clone https://github.com/RM-Stephey/stepheybot-music.git
cd stepheybot-music

# Or if you're working on a fork
git clone https://github.com/YOUR_USERNAME/stepheybot-music.git
cd stepheybot-music
```

### 2. Environment Setup
```bash
# Copy environment template
cp .env.template .env

# Edit configuration (optional for development)
# nano .env  # or your preferred editor
```

### 3. Build and Run
```bash
# Install dependencies and build
cargo build

# Run with development logging
RUST_LOG=debug cargo run

# Or run with file watching (if cargo-watch installed)
cargo watch -x run
```

### 4. Verify Installation
```bash
# Test API endpoints
curl http://localhost:8083/health | jq
curl http://localhost:8083/api/v1/status | jq
```

## Detailed Setup

### Development Environment Configuration

#### 1. **Rust Configuration**

Create `~/.cargo/config.toml` for optimal development:
```toml
[build]
# Use all CPU cores for compilation
jobs = 0

[target.x86_64-unknown-linux-gnu]
# Use mold linker for faster linking (if available)
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[alias]
# Useful cargo aliases
b = "build"
c = "check"
t = "test"
r = "run"
br = "build --release"
```

#### 2. **Database Setup**

The application automatically creates and populates the database on first run:

```bash
# Default database location
mkdir -p data
# Database will be created at: data/stepheybot-music.db

# For custom database location, set environment variable:
export DATABASE_URL="sqlite:custom/path/music.db"
```

#### 3. **Logging Configuration**

Set up development logging:
```bash
# Environment variables for development
export RUST_LOG="stepheybot_music=debug,tower_http=debug,sqlx=warn"
export RUST_BACKTRACE=1
export RUST_ENV=development
```

### IDE Configuration

#### VS Code Setup

**Extensions** (`.vscode/extensions.json`):
```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "serayuzgur.crates",
    "vadimcn.vscode-lldb",
    "ms-vscode.vscode-json"
  ]
}
```

**Settings** (`.vscode/settings.json`):
```json
{
  "rust-analyzer.cargo.buildScripts.enable": true,
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.cargo.autoreload": true,
  "rust-analyzer.inlayHints.enable": true,
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "rust-lang.rust-analyzer"
}
```

**Debug Configuration** (`.vscode/launch.json`):
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug StepheyBot Music",
      "cargo": {
        "args": ["build", "--bin=stepheybot-music"],
        "filter": {
          "name": "stepheybot-music",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug",
        "DATABASE_URL": "sqlite:data/stepheybot-music.db"
      }
    }
  ]
}
```

#### JetBrains RustRover/IntelliJ + Rust Plugin

1. Open project directory
2. RustRover should auto-detect Cargo project
3. Configure run configuration:
   - **Program**: `cargo`
   - **Arguments**: `run`
   - **Working directory**: Project root
   - **Environment variables**: `RUST_LOG=debug`

### Advanced Configuration

#### Environment Variables Reference

```bash
# Server Configuration
export PORT=8083                    # Server port
export HOST=0.0.0.0                # Server host

# Database Configuration
export DATABASE_URL="sqlite:data/stepheybot-music.db"
export DATABASE_MAX_CONNECTIONS=10

# Logging Configuration
export RUST_LOG="stepheybot_music=debug,tower_http=debug,sqlx=warn"
export RUST_BACKTRACE=1             # Enable backtraces
export RUST_ENV=development         # Environment mode

# External Services (Optional)
export NAVIDROME_URL="http://localhost:4533"
export NAVIDROME_USERNAME="admin"
export NAVIDROME_PASSWORD="password"
export LIDARR_URL="http://localhost:8686"
export LIDARR_API_KEY="your_api_key"

# Development Settings
export CARGO_TERM_COLOR=always      # Colored cargo output
export RUST_SRC_PATH="$(rustc --print sysroot)/lib/rustlib/src/rust/library"
```

## Development Workflow

### Daily Development Process

#### 1. **Start Development Session**
```bash
# Update dependencies
cargo update

# Check for issues
cargo check

# Run tests
cargo test

# Start development server with auto-reload
cargo watch -x 'run'
```

#### 2. **Code Development Cycle**
```bash
# Make changes to code
# ...

# Format code
cargo fmt

# Check for issues
cargo clippy -- -D warnings

# Run specific tests
cargo test test_name

# Build and test
cargo build && cargo test
```

#### 3. **Testing Endpoints**
```bash
# Health check
curl http://localhost:8083/health

# API status
curl http://localhost:8083/api/v1/status | jq

# Get recommendations
curl "http://localhost:8083/api/v1/recommendations/user1?limit=5" | jq

# Library stats
curl http://localhost:8083/api/v1/library/stats | jq

# Generate playlist
curl -X POST http://localhost:8083/api/v1/playlists/generate \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Playlist", "duration_minutes": 30}' | jq
```

### Git Workflow

#### Branch Management
```bash
# Create feature branch
git checkout -b feature/new-recommendation-algorithm

# Make commits with conventional commit format
git commit -m "feat: add collaborative filtering algorithm"
git commit -m "fix: resolve database connection timeout"
git commit -m "docs: update API documentation"

# Push branch
git push origin feature/new-recommendation-algorithm

# Create pull request on GitHub
```

#### Commit Message Convention
```
<type>(<scope>): <description>

Types:
- feat: New feature
- fix: Bug fix
- docs: Documentation changes
- style: Code style changes
- refactor: Code refactoring
- test: Test changes
- chore: Build/tooling changes

Examples:
feat(api): add user preference endpoints
fix(db): resolve connection pool exhaustion
docs(readme): update installation instructions
```

## Testing

### Unit Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_recommendation_engine

# Run tests in specific module
cargo test recommendation::tests

# Run tests with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out html
```

### Integration Testing

```bash
# Run integration tests
cargo test --test integration

# Test with specific database
DATABASE_URL="sqlite::memory:" cargo test

# Load testing with Apache Bench
ab -n 1000 -c 10 http://localhost:8083/health
```

### API Testing with Scripts

Create `scripts/test_api.sh`:
```bash
#!/bin/bash
set -e

BASE_URL="http://localhost:8083"

echo "Testing API endpoints..."

# Health check
echo "Health check:"
curl -s "$BASE_URL/health" | jq

# API status
echo -e "\nAPI Status:"
curl -s "$BASE_URL/api/v1/status" | jq '.service, .version'

# Recommendations
echo -e "\nRecommendations:"
curl -s "$BASE_URL/api/v1/recommendations/user1?limit=3" | jq '.total'

echo -e "\nAll tests completed!"
```

```bash
chmod +x scripts/test_api.sh
./scripts/test_api.sh
```

## Debugging

### Common Debugging Techniques

#### 1. **Enable Debug Logging**
```bash
# Maximum verbosity
RUST_LOG=trace cargo run

# Specific modules
RUST_LOG=stepheybot_music::services::recommendation=trace cargo run

# With backtraces
RUST_BACKTRACE=full cargo run
```

#### 2. **Database Debugging**
```bash
# Enable SQL query logging
RUST_LOG=sqlx=debug cargo run

# Connect to database directly
sqlite3 data/stepheybot-music.db
.tables
.schema users
SELECT * FROM users LIMIT 5;
```

#### 3. **Performance Debugging**
```bash
# Build with debug symbols
cargo build

# Profile with perf (Linux)
perf record -g cargo run
perf report

# Memory profiling with valgrind
valgrind --tool=massif cargo run
```

#### 4. **Network Debugging**
```bash
# Monitor HTTP requests
sudo tcpdump -i lo -A 'port 8083'

# Test with verbose curl
curl -v http://localhost:8083/health

# Load testing
wrk -t12 -c400 -d30s http://localhost:8083/health
```

### VS Code Debugging

Set breakpoints in code and use the debug configuration:

1. Set breakpoints by clicking left margin
2. Press F5 or use "Run and Debug" panel
3. Use debug console for expression evaluation
4. Step through code with F10 (step over) and F11 (step into)

### Common Issues and Solutions

#### Issue: Compilation Errors
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Check for conflicting features
cargo tree
```

#### Issue: Database Lock Errors
```bash
# Check for hanging connections
lsof -p $(pgrep stepheybot-music)

# Reset database
rm data/stepheybot-music.db
cargo run  # Will recreate with sample data
```

#### Issue: Port Already in Use
```bash
# Find process using port
lsof -i :8083
sudo netstat -tulpn | grep :8083

# Kill process
kill -9 <pid>

# Or use different port
PORT=8084 cargo run
```

## Docker Development

### Development Container

Create `docker-compose.dev.yml`:
```yaml
version: '3.8'
services:
  stepheybot-dev:
    build:
      context: .
      dockerfile: Dockerfile.dev
    ports:
      - "8083:8083"
    volumes:
      - .:/app
      - cargo-cache:/usr/local/cargo/registry
      - target-cache:/app/target
    environment:
      - RUST_LOG=debug
      - DATABASE_URL=sqlite:/app/data/stepheybot-music.db
    command: cargo watch -x run

volumes:
  cargo-cache:
  target-cache:
```

Development Dockerfile (`Dockerfile.dev`):
```dockerfile
FROM rust:latest

WORKDIR /app

# Install development tools
RUN cargo install cargo-watch

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Pre-build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build
RUN rm -rf src

# Development mode
CMD ["cargo", "watch", "-x", "run"]
```

### Docker Commands

```bash
# Build development image
docker-compose -f docker-compose.dev.yml build

# Run development container
docker-compose -f docker-compose.dev.yml up

# Shell access to container
docker-compose -f docker-compose.dev.yml exec stepheybot-dev bash

# View logs
docker-compose -f docker-compose.dev.yml logs -f
```

## Contributing

### Before Contributing

1. **Read the contributing guidelines**: `docs/development/contributing.md`
2. **Check existing issues**: Look for relevant GitHub issues
3. **Discuss major changes**: Open an issue for discussion first

### Development Checklist

Before submitting a pull request:

- [ ] Code compiles without warnings: `cargo clippy -- -D warnings`
- [ ] Code is formatted: `cargo fmt`
- [ ] Tests pass: `cargo test`
- [ ] Documentation updated if needed
- [ ] Commit messages follow convention
- [ ] Branch is up to date with main

### Code Quality Standards

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy -- -D warnings

# Audit dependencies
cargo audit

# Check for outdated dependencies
cargo outdated

# Generate documentation
cargo doc --no-deps --open
```

## Troubleshooting

### Common Development Issues

#### 1. **Rust Compilation Issues**

**Error**: "error: could not compile `stepheybot-music`"
```bash
# Solution 1: Clean and rebuild
cargo clean
cargo build

# Solution 2: Update toolchain
rustup update

# Solution 3: Check for conflicting dependencies
cargo tree --duplicates
```

#### 2. **Database Connection Issues**

**Error**: "database is locked"
```bash
# Check for multiple instances
ps aux | grep stepheybot-music

# Remove lock files
rm data/stepheybot-music.db-*

# Reset database
rm data/stepheybot-music.db
cargo run
```

#### 3. **Port Binding Issues**

**Error**: "Address already in use (os error 98)"
```bash
# Find and kill process
sudo lsof -i :8083
kill -9 <PID>

# Or use different port
PORT=8084 cargo run
```

#### 4. **Missing Dependencies**

**Error**: Build failures due to missing system libraries
```bash
# CachyOS/Arch Linux
sudo pacman -S pkg-config openssl sqlite

# Ubuntu/Debian
sudo apt-get install pkg-config libssl-dev libsqlite3-dev

# macOS
brew install pkg-config openssl sqlite
```

### Performance Issues

#### Slow Compilation Times
```bash
# Use faster linker (Linux)
sudo pacman -S mold  # or apt-get install mold
# Add to ~/.cargo/config.toml (see above)

# Parallel compilation
export CARGO_BUILD_JOBS=$(nproc)

# Use sccache for caching
cargo install sccache
export RUSTC_WRAPPER=sccache
```

#### High Memory Usage
```bash
# Limit parallel jobs
export CARGO_BUILD_JOBS=2

# Use release mode for better performance
cargo run --release
```

### Getting Help

#### Resources
- **Documentation**: `docs/` directory
- **API Reference**: `docs/api/endpoints.md`
- **GitHub Issues**: [Project Issues](https://github.com/RM-Stephey/stepheybot-music/issues)
- **Discussions**: [GitHub Discussions](https://github.com/RM-Stephey/stepheybot-music/discussions)

#### Contact
- **Email**: stephey@stepheybot.dev
- **GitHub**: [@RM-Stephey](https://github.com/RM-Stephey)

---

**Happy coding! 🎵**

This development guide will help you get up and running with StepheyBot Music development. For additional help or questions, don't hesitate to reach out through the channels listed above.

**Last Updated**: 2025-06-20  
**Guide Version**: 1.0