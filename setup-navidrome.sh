#!/bin/bash

# StepheyBot Music - Navidrome Integration Setup Script
# This script helps you connect StepheyBot Music to your Navidrome instance

set -e

# Colors for neon-themed output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
PINK='\033[95m'
NC='\033[0m' # No Color

# Configuration variables
NAVIDROME_URL=""
USERNAME=""
PASSWORD=""
CONFIG_FILE=".env"
BACKUP_FILE=".env.backup"

# ASCII Art Banner
show_banner() {
    echo -e "${CYAN}"
    echo "  ____  _             _                ____        _   "
    echo " / ___|| |_ ___ _ __ | |__   ___ _   _| __ )  ___ | |_ "
    echo " \\___ \\| __/ _ \\ '_ \\| '_ \\ / _ \\ | | |  _ \\ / _ \\| __|"
    echo "  ___) | ||  __/ |_) | | | |  __/ |_| | |_) | (_) | |_ "
    echo " |____/ \\__\\___| .__/|_| |_|\\___| \\__, |____/ \\___/ \\__|"
    echo "               |_|              |___/                 "
    echo -e "${PINK}              🎵 Navidrome Integration 🎵              ${NC}"
    echo ""
    echo -e "${PURPLE}=========================================================${NC}"
    echo -e "${CYAN}   Connect your music library to AI recommendations    ${NC}"
    echo -e "${PURPLE}=========================================================${NC}"
    echo ""
}

# Function to check dependencies
check_dependencies() {
    echo -e "${BLUE}🔍 Checking system dependencies...${NC}"

    local missing=()

    command -v curl >/dev/null 2>&1 || missing+=("curl")
    command -v openssl >/dev/null 2>&1 || missing+=("openssl")
    command -v cargo >/dev/null 2>&1 || missing+=("cargo (Rust)")

    if [ ${#missing[@]} -ne 0 ]; then
        echo -e "${RED}❌ Missing required tools: ${missing[*]}${NC}"
        echo -e "${YELLOW}Please install the missing tools and try again${NC}"
        exit 1
    fi

    echo -e "${GREEN}✅ All dependencies found${NC}"
    echo ""
}

# Function to detect existing Navidrome instances
detect_navidrome() {
    echo -e "${BLUE}🔍 Scanning for Navidrome instances...${NC}"

    local common_ports=(4533 4534 4535 8080 8081)
    local found_instances=()

    for port in "${common_ports[@]}"; do
        echo -e "${CYAN}   Checking localhost:$port...${NC}"
        if timeout 3 curl -s "http://localhost:$port" >/dev/null 2>&1; then
            # Check if it's actually Navidrome by looking for specific responses
            local response=$(timeout 3 curl -s "http://localhost:$port/rest/ping" 2>/dev/null || echo "")
            if echo "$response" | grep -q "subsonic" 2>/dev/null; then
                found_instances+=("http://localhost:$port")
                echo -e "${GREEN}   ✅ Found Navidrome at localhost:$port${NC}"
            else
                echo -e "${YELLOW}   ⚠️  Service found but not Navidrome${NC}"
            fi
        else
            echo -e "${CYAN}   ❌ No service at localhost:$port${NC}"
        fi
    done

    if [ ${#found_instances[@]} -eq 0 ]; then
        echo -e "${YELLOW}⚠️  No Navidrome instances detected automatically${NC}"
        echo -e "${CYAN}   You can still configure manually${NC}"
    elif [ ${#found_instances[@]} -eq 1 ]; then
        echo -e "${GREEN}🎯 Found one Navidrome instance: ${found_instances[0]}${NC}"
        NAVIDROME_URL="${found_instances[0]}"
    else
        echo -e "${GREEN}🎯 Found multiple Navidrome instances:${NC}"
        for i in "${!found_instances[@]}"; do
            echo -e "${CYAN}   $((i+1)). ${found_instances[i]}${NC}"
        done
        echo ""
        echo -e "${BLUE}Select which instance to use (1-${#found_instances[@]}):${NC}"
        read -r selection
        if [[ "$selection" =~ ^[0-9]+$ ]] && [ "$selection" -ge 1 ] && [ "$selection" -le ${#found_instances[@]} ]; then
            NAVIDROME_URL="${found_instances[$((selection-1))]}"
            echo -e "${GREEN}Selected: $NAVIDROME_URL${NC}"
        fi
    fi

    echo ""
}

# Function to load existing configuration
load_existing_config() {
    if [ -f "$CONFIG_FILE" ]; then
        echo -e "${BLUE}📁 Found existing configuration in $CONFIG_FILE${NC}"

        # Parse existing .env file
        while IFS='=' read -r key value; do
            # Skip comments and empty lines
            [[ $key =~ ^#.*$ ]] && continue
            [[ -z $key ]] && continue

            # Remove quotes from value
            value=$(echo "$value" | sed 's/^"\(.*\)"$/\1/' | sed "s/^'\(.*\)'$/\1/")

            case $key in
                NAVIDROME_URL) NAVIDROME_URL="$value" ;;
                NAVIDROME_USERNAME) USERNAME="$value" ;;
                NAVIDROME_PASSWORD) PASSWORD="$value" ;;
            esac
        done < "$CONFIG_FILE"

        if [ -n "$NAVIDROME_URL" ]; then
            echo -e "${GREEN}   Current Navidrome URL: $NAVIDROME_URL${NC}"
        fi
        if [ -n "$USERNAME" ]; then
            echo -e "${GREEN}   Current username: $USERNAME${NC}"
        fi
        echo ""
    fi
}

# Function to prompt for configuration
prompt_configuration() {
    echo -e "${PINK}🔧 Navidrome Configuration${NC}"
    echo ""

    # Navidrome URL
    if [ -z "$NAVIDROME_URL" ]; then
        echo -e "${BLUE}Enter your Navidrome server URL:${NC}"
        echo -e "${CYAN}Examples: http://localhost:4533, https://music.yourdomain.com${NC}"
        read -r NAVIDROME_URL
    else
        echo -e "${GREEN}Using detected URL: $NAVIDROME_URL${NC}"
        echo -e "${BLUE}Press Enter to keep this URL, or type a new one:${NC}"
        read -r new_url
        if [ -n "$new_url" ]; then
            NAVIDROME_URL="$new_url"
        fi
    fi

    # Remove trailing slash
    NAVIDROME_URL="${NAVIDROME_URL%/}"

    # Username
    if [ -z "$USERNAME" ]; then
        echo -e "${BLUE}Enter your Navidrome username:${NC}"
        read -r USERNAME
    else
        echo -e "${GREEN}Using existing username: $USERNAME${NC}"
        echo -e "${BLUE}Press Enter to keep this username, or type a new one:${NC}"
        read -r new_username
        if [ -n "$new_username" ]; then
            USERNAME="$new_username"
        fi
    fi

    # Password
    echo -e "${BLUE}Enter your Navidrome password:${NC}"
    read -s PASSWORD
    echo ""

    # Validate inputs
    if [ -z "$NAVIDROME_URL" ] || [ -z "$USERNAME" ] || [ -z "$PASSWORD" ]; then
        echo -e "${RED}❌ All fields are required${NC}"
        exit 1
    fi

    echo ""
}

# Function to test Navidrome connection
test_navidrome_connection() {
    echo -e "${BLUE}🔐 Testing Navidrome connection...${NC}"

    # Test basic connectivity
    echo -e "${CYAN}   Testing basic connectivity...${NC}"
    if ! timeout 10 curl -s "$NAVIDROME_URL" >/dev/null; then
        echo -e "${RED}❌ Cannot connect to $NAVIDROME_URL${NC}"
        echo -e "${YELLOW}   Please check the URL and ensure Navidrome is running${NC}"
        return 1
    fi
    echo -e "${GREEN}   ✅ Server is reachable${NC}"

    # Test authentication
    echo -e "${CYAN}   Testing authentication...${NC}"

    # Generate authentication token
    local salt=$(openssl rand -hex 16 2>/dev/null || echo "randomsalt$(date +%s)")
    local token=$(echo -n "${PASSWORD}${salt}" | openssl md5 | cut -d' ' -f2 2>/dev/null || echo -n "${PASSWORD}${salt}" | md5sum | cut -d' ' -f1)
    local auth_params="u=${USERNAME}&t=${token}&s=${salt}&v=1.16.1&c=StepheyBot-Music"

    local ping_url="${NAVIDROME_URL}/rest/ping?${auth_params}"
    local response=$(curl -s "$ping_url")

    if echo "$response" | grep -q '"status":"ok"'; then
        echo -e "${GREEN}   ✅ Authentication successful${NC}"

        # Test library access
        echo -e "${CYAN}   Testing library access...${NC}"
        local users_url="${NAVIDROME_URL}/rest/getUsers?${auth_params}"
        local users_response=$(curl -s "$users_url")

        if echo "$users_response" | grep -q '"status":"ok"'; then
            echo -e "${GREEN}   ✅ Library access confirmed${NC}"

            # Get library stats
            local artists_url="${NAVIDROME_URL}/rest/getArtists?${auth_params}"
            local artists_response=$(curl -s "$artists_url")

            if echo "$artists_response" | grep -q '"status":"ok"'; then
                local artist_count=$(echo "$artists_response" | grep -o '"name"' | wc -l)
                echo -e "${GREEN}   📊 Found $artist_count artists in your library${NC}"
            fi
        else
            echo -e "${YELLOW}   ⚠️  Authentication works but limited library access${NC}"
        fi

        return 0
    else
        echo -e "${RED}❌ Authentication failed${NC}"
        echo -e "${YELLOW}   Response: $response${NC}"
        echo -e "${YELLOW}   Please check your username and password${NC}"
        return 1
    fi
}

# Function to create configuration file
create_config_file() {
    echo -e "${BLUE}💾 Creating configuration file...${NC}"

    # Backup existing config
    if [ -f "$CONFIG_FILE" ]; then
        cp "$CONFIG_FILE" "$BACKUP_FILE"
        echo -e "${YELLOW}   Backed up existing config to $BACKUP_FILE${NC}"
    fi

    # Create new configuration
    cat > "$CONFIG_FILE" << EOF
# StepheyBot Music Configuration
# Generated by setup-navidrome.sh on $(date)

# =============================================================================
# NAVIDROME INTEGRATION
# =============================================================================

# Navidrome server URL
NAVIDROME_URL=$NAVIDROME_URL

# Navidrome credentials
NAVIDROME_USERNAME=$USERNAME
NAVIDROME_PASSWORD=$PASSWORD

# Connection settings
NAVIDROME_TIMEOUT=30
NAVIDROME_VERIFY_SSL=true

# =============================================================================
# STEPHEYBOT MUSIC SETTINGS
# =============================================================================

# Server configuration
PORT=8083
DATABASE_URL=sqlite:data/stepheybot-music.db

# Data sources
POPULATE_SAMPLE_DATA=false

# Background tasks
ENABLE_BACKGROUND_TASKS=true
SYNC_INTERVAL=30
RECOMMENDATION_INTERVAL=60

# =============================================================================
# LOGGING & DEBUGGING
# =============================================================================

# Log level (trace, debug, info, warn, error)
RUST_LOG=info

# Environment
RUST_ENV=development
DEBUG=true

# Enable detailed error backtraces
RUST_BACKTRACE=1

# =============================================================================
# FRONTEND DEVELOPMENT
# =============================================================================

# Enable CORS for frontend development
ENABLE_CORS=true
CORS_ORIGINS=http://localhost:5173,http://localhost:3000
EOF

    echo -e "${GREEN}✅ Configuration saved to $CONFIG_FILE${NC}"
    echo ""
}

# Function to test the integration
test_integration() {
    echo -e "${BLUE}🤖 Testing StepheyBot Music integration...${NC}"

    if [ ! -f "Cargo.toml" ]; then
        echo -e "${YELLOW}⚠️  Not in StepheyBot Music directory${NC}"
        echo -e "${CYAN}   Please run this script from the music-recommender directory${NC}"
        return 1
    fi

    echo -e "${CYAN}   Building StepheyBot Music...${NC}"
    if cargo build --quiet --bin stepheybot-music 2>/dev/null; then
        echo -e "${GREEN}   ✅ Build successful${NC}"
    else
        echo -e "${RED}   ❌ Build failed${NC}"
        echo -e "${YELLOW}   You may need to update the main.rs to use the Navidrome integration${NC}"
        return 1
    fi

    # Test with a quick run
    echo -e "${CYAN}   Testing server startup...${NC}"
    timeout 10 cargo run --quiet > /tmp/stepheybot-test.log 2>&1 &
    local server_pid=$!

    sleep 5

    if kill -0 $server_pid 2>/dev/null; then
        echo -e "${GREEN}   ✅ Server started successfully${NC}"

        # Test health endpoint
        if curl -s http://localhost:8083/health >/dev/null 2>&1; then
            echo -e "${GREEN}   ✅ Health endpoint responding${NC}"
        fi

        # Test Navidrome status endpoint
        local status_response=$(curl -s http://localhost:8083/api/v1/navidrome/status 2>/dev/null || echo "")
        if echo "$status_response" | grep -q "navidrome" 2>/dev/null; then
            echo -e "${GREEN}   ✅ Navidrome integration active${NC}"
        fi

        kill $server_pid 2>/dev/null || true
        wait $server_pid 2>/dev/null || true
    else
        echo -e "${YELLOW}   ⚠️  Server startup test inconclusive${NC}"
    fi

    rm -f /tmp/stepheybot-test.log
    echo ""
}

# Function to show next steps
show_next_steps() {
    echo -e "${PINK}🎉 Navidrome Integration Setup Complete!${NC}"
    echo ""
    echo -e "${PURPLE}📋 Next Steps:${NC}"
    echo ""
    echo -e "${CYAN}1. Start the development environment:${NC}"
    echo -e "${YELLOW}   ./start-dev.sh${NC}"
    echo ""
    echo -e "${CYAN}2. Open your browser to:${NC}"
    echo -e "${YELLOW}   Frontend: http://localhost:5173${NC}"
    echo -e "${YELLOW}   Backend:  http://localhost:8083${NC}"
    echo ""
    echo -e "${CYAN}3. Test API endpoints:${NC}"
    echo -e "${YELLOW}   curl http://localhost:8083/api/v1/navidrome/status${NC}"
    echo -e "${YELLOW}   curl http://localhost:8083/api/v1/library/stats${NC}"
    echo -e "${YELLOW}   curl http://localhost:8083/api/v1/recommendations/user1${NC}"
    echo ""
    echo -e "${CYAN}4. Monitor the logs:${NC}"
    echo -e "${YELLOW}   tail -f backend.log frontend.log${NC}"
    echo ""
    echo -e "${PURPLE}🔗 Useful Commands:${NC}"
    echo -e "${CYAN}   Test connection:     ${YELLOW}./test-navidrome.sh${NC}"
    echo -e "${CYAN}   Reconfigure:         ${YELLOW}./setup-navidrome.sh${NC}"
    echo -e "${CYAN}   System test:         ${YELLOW}./test-system.sh${NC}"
    echo ""
    echo -e "${GREEN}🎵 Your music library is now connected to AI recommendations! 🎵${NC}"
}

# Function to show troubleshooting info
show_troubleshooting() {
    echo -e "${YELLOW}🔧 Troubleshooting Tips:${NC}"
    echo ""
    echo -e "${CYAN}If Navidrome connection fails:${NC}"
    echo -e "${YELLOW}   • Check Navidrome is running: curl $NAVIDROME_URL${NC}"
    echo -e "${YELLOW}   • Verify credentials in Navidrome web interface${NC}"
    echo -e "${YELLOW}   • Check firewall/network settings${NC}"
    echo ""
    echo -e "${CYAN}If StepheyBot Music fails to start:${NC}"
    echo -e "${YELLOW}   • Check logs: tail -f backend.log${NC}"
    echo -e "${YELLOW}   • Verify Rust installation: cargo --version${NC}"
    echo -e "${YELLOW}   • Try: cargo clean && cargo build${NC}"
    echo ""
    echo -e "${CYAN}For configuration issues:${NC}"
    echo -e "${YELLOW}   • Edit .env file manually${NC}"
    echo -e "${YELLOW}   • Restore backup: cp .env.backup .env${NC}"
    echo -e "${YELLOW}   • Check environment variables: env | grep NAVIDROME${NC}"
    echo ""
}

# Main execution flow
main() {
    show_banner
    check_dependencies
    load_existing_config
    detect_navidrome
    prompt_configuration

    if test_navidrome_connection; then
        create_config_file
        test_integration
        show_next_steps
    else
        echo ""
        echo -e "${RED}❌ Navidrome connection test failed${NC}"
        show_troubleshooting
        echo ""
        echo -e "${BLUE}Would you like to save the configuration anyway? (y/n):${NC}"
        read -r save_anyway
        if [[ $save_anyway =~ ^[Yy]$ ]]; then
            create_config_file
            echo -e "${YELLOW}Configuration saved. You can edit .env manually and test again.${NC}"
        fi
        exit 1
    fi
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "StepheyBot Music - Navidrome Integration Setup"
        echo ""
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --test-only    Only test existing configuration"
        echo "  --reconfigure  Force reconfiguration"
        echo ""
        echo "This script helps you connect StepheyBot Music to your Navidrome instance."
        echo "It will detect running Navidrome servers, test connectivity, and create"
        echo "the necessary configuration files."
        exit 0
        ;;
    --test-only)
        show_banner
        load_existing_config
        if [ -n "$NAVIDROME_URL" ] && [ -n "$USERNAME" ] && [ -n "$PASSWORD" ]; then
            test_navidrome_connection
        else
            echo -e "${RED}❌ No existing configuration found${NC}"
            echo -e "${CYAN}Run without --test-only to configure${NC}"
            exit 1
        fi
        exit 0
        ;;
    --reconfigure)
        show_banner
        rm -f "$CONFIG_FILE"
        main
        ;;
    "")
        main
        ;;
    *)
        echo -e "${RED}Unknown option: $1${NC}"
        echo "Use --help for usage information"
        exit 1
        ;;
esac
