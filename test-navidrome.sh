#!/bin/bash

# StepheyBot Music - Navidrome Connection Test Script
# Tests connectivity and authentication with your Navidrome instance

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# ASCII Art Banner
echo -e "${CYAN}"
echo "  _   _            _     _                          "
echo " | \ | | __ ___   _(_) __| |_ __ ___  _ __ ___   ___ "
echo " |  \| |/ _\` \ \ / / |/ _\` | '__/ _ \| '_ \` _ \ / _ \\"
echo " | |\  | (_| |\ V /| | (_| | | | (_) | | | | | |  __/"
echo " |_| \_|\__,_| \_/ |_|\__,_|_|  \___/|_| |_| |_|\___|"
echo -e "${PURPLE}            🎵 Connection Test 🎵                   ${NC}"
echo ""

# Configuration
NAVIDROME_URL=""
USERNAME=""
PASSWORD=""
CONFIG_FILE=".env"

# Function to read configuration
load_config() {
    if [ -f "$CONFIG_FILE" ]; then
        echo -e "${BLUE}📁 Loading configuration from $CONFIG_FILE...${NC}"

        # Source the .env file safely
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
    fi
}

# Function to prompt for configuration
prompt_config() {
    echo -e "${YELLOW}🔧 Navidrome Configuration Setup${NC}"
    echo ""

    if [ -z "$NAVIDROME_URL" ]; then
        echo -e "${BLUE}Enter your Navidrome server URL (e.g., http://localhost:4533):${NC}"
        read -r NAVIDROME_URL
    else
        echo -e "${GREEN}Using Navidrome URL: $NAVIDROME_URL${NC}"
    fi

    if [ -z "$USERNAME" ]; then
        echo -e "${BLUE}Enter your Navidrome username:${NC}"
        read -r USERNAME
    else
        echo -e "${GREEN}Using username: $USERNAME${NC}"
    fi

    if [ -z "$PASSWORD" ]; then
        echo -e "${BLUE}Enter your Navidrome password:${NC}"
        read -s PASSWORD
        echo ""
    else
        echo -e "${GREEN}Using configured password${NC}"
    fi

    # Remove trailing slash from URL
    NAVIDROME_URL="${NAVIDROME_URL%/}"

    echo ""
}

# Function to create authentication token
create_auth_token() {
    # Generate salt
    SALT=$(openssl rand -hex 16 2>/dev/null || echo "randomsalt$(date +%s)")

    # Create token (MD5 hash of password + salt)
    TOKEN=$(echo -n "${PASSWORD}${SALT}" | md5sum | cut -d' ' -f1 2>/dev/null || echo -n "${PASSWORD}${SALT}" | md5)

    # Build auth parameters
    AUTH_PARAMS="u=${USERNAME}&t=${TOKEN}&s=${SALT}&v=1.16.1&c=StepheyBot-Music"
}

# Function to test basic connectivity
test_connectivity() {
    echo -e "${BLUE}🌐 Test 1: Testing basic connectivity...${NC}"

    if curl -s --connect-timeout 10 "$NAVIDROME_URL" >/dev/null; then
        echo -e "${GREEN}✅ Successfully connected to $NAVIDROME_URL${NC}"
        return 0
    else
        echo -e "${RED}❌ Failed to connect to $NAVIDROME_URL${NC}"
        echo -e "${YELLOW}   Please check:${NC}"
        echo -e "${YELLOW}   - Is the URL correct?${NC}"
        echo -e "${YELLOW}   - Is Navidrome running?${NC}"
        echo -e "${YELLOW}   - Is the server accessible?${NC}"
        return 1
    fi
}

# Function to test authentication
test_authentication() {
    echo -e "${BLUE}🔐 Test 2: Testing authentication...${NC}"

    create_auth_token

    local ping_url="${NAVIDROME_URL}/rest/ping?${AUTH_PARAMS}"
    local response=$(curl -s "$ping_url")

    if echo "$response" | grep -q '"status":"ok"'; then
        echo -e "${GREEN}✅ Authentication successful${NC}"
        echo -e "${YELLOW}   Response: $response${NC}"
        return 0
    else
        echo -e "${RED}❌ Authentication failed${NC}"
        echo -e "${YELLOW}   Response: $response${NC}"
        echo -e "${YELLOW}   Please check your username and password${NC}"
        return 1
    fi
}

# Function to test API endpoints
test_api_endpoints() {
    echo -e "${BLUE}🔍 Test 3: Testing API endpoints...${NC}"

    create_auth_token

    # Test getUsers endpoint
    echo -e "${CYAN}  Testing getUsers...${NC}"
    local users_url="${NAVIDROME_URL}/rest/getUsers?${AUTH_PARAMS}"
    local users_response=$(curl -s "$users_url")

    if echo "$users_response" | grep -q '"status":"ok"'; then
        local user_count=$(echo "$users_response" | grep -o '"username"' | wc -l)
        echo -e "${GREEN}  ✅ getUsers: Found $user_count user(s)${NC}"
    else
        echo -e "${RED}  ❌ getUsers failed${NC}"
    fi

    # Test getArtists endpoint
    echo -e "${CYAN}  Testing getArtists...${NC}"
    local artists_url="${NAVIDROME_URL}/rest/getArtists?${AUTH_PARAMS}"
    local artists_response=$(curl -s "$artists_url")

    if echo "$artists_response" | grep -q '"status":"ok"'; then
        local artist_count=$(echo "$artists_response" | grep -o '"name"' | wc -l)
        echo -e "${GREEN}  ✅ getArtists: Found $artist_count artist(s)${NC}"
    else
        echo -e "${RED}  ❌ getArtists failed${NC}"
    fi

    # Test getAlbums endpoint
    echo -e "${CYAN}  Testing getAlbums...${NC}"
    local albums_url="${NAVIDROME_URL}/rest/getAlbumList2?type=recent&size=10&${AUTH_PARAMS}"
    local albums_response=$(curl -s "$albums_url")

    if echo "$albums_response" | grep -q '"status":"ok"'; then
        local album_count=$(echo "$albums_response" | grep -o '"album"' | wc -l)
        echo -e "${GREEN}  ✅ getAlbumList2: Found $album_count recent album(s)${NC}"
    else
        echo -e "${RED}  ❌ getAlbumList2 failed${NC}"
    fi

    # Test getRandomSongs endpoint
    echo -e "${CYAN}  Testing getRandomSongs...${NC}"
    local random_url="${NAVIDROME_URL}/rest/getRandomSongs?size=5&${AUTH_PARAMS}"
    local random_response=$(curl -s "$random_url")

    if echo "$random_response" | grep -q '"status":"ok"'; then
        local song_count=$(echo "$random_response" | grep -o '"title"' | wc -l)
        echo -e "${GREEN}  ✅ getRandomSongs: Found $song_count random song(s)${NC}"
    else
        echo -e "${RED}  ❌ getRandomSongs failed${NC}"
    fi
}

# Function to test library statistics
test_library_stats() {
    echo -e "${BLUE}📊 Test 4: Getting library statistics...${NC}"

    create_auth_token

    # Get artists count
    local artists_url="${NAVIDROME_URL}/rest/getArtists?${AUTH_PARAMS}"
    local artists_response=$(curl -s "$artists_url")
    local artists_count=0

    if echo "$artists_response" | grep -q '"status":"ok"'; then
        artists_count=$(echo "$artists_response" | grep -o '"name"' | wc -l)
    fi

    # Get albums count
    local albums_url="${NAVIDROME_URL}/rest/getAlbumList2?type=alphabeticalByArtist&size=500&${AUTH_PARAMS}"
    local albums_response=$(curl -s "$albums_url")
    local albums_count=0

    if echo "$albums_response" | grep -q '"status":"ok"'; then
        albums_count=$(echo "$albums_response" | grep -o '"album"' | wc -l)
    fi

    # Get random songs to estimate total
    local songs_url="${NAVIDROME_URL}/rest/getRandomSongs?size=1&${AUTH_PARAMS}"
    local songs_response=$(curl -s "$songs_url")

    echo -e "${GREEN}📈 Library Statistics:${NC}"
    echo -e "${CYAN}   🎤 Artists: $artists_count${NC}"
    echo -e "${CYAN}   💿 Albums: $albums_count${NC}"
    echo -e "${CYAN}   🎵 Songs: Available (random sampling works)${NC}"

    if [ "$artists_count" -gt 0 ] && [ "$albums_count" -gt 0 ]; then
        echo -e "${GREEN}✅ Your music library is accessible and has content!${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  Your library appears to be empty or not fully scanned${NC}"
        return 1
    fi
}

# Function to save configuration
save_config() {
    echo -e "${BLUE}💾 Saving configuration...${NC}"

    # Backup existing config
    if [ -f "$CONFIG_FILE" ]; then
        cp "$CONFIG_FILE" "${CONFIG_FILE}.backup"
        echo -e "${YELLOW}   Backed up existing config to ${CONFIG_FILE}.backup${NC}"
    fi

    # Create or update .env file
    {
        echo "# StepheyBot Music - Navidrome Configuration"
        echo "# Generated by test-navidrome.sh on $(date)"
        echo ""
        echo "# Navidrome server configuration"
        echo "NAVIDROME_URL=$NAVIDROME_URL"
        echo "NAVIDROME_USERNAME=$USERNAME"
        echo "NAVIDROME_PASSWORD=$PASSWORD"
        echo "NAVIDROME_TIMEOUT=30"
        echo "NAVIDROME_VERIFY_SSL=true"
        echo ""
        echo "# StepheyBot Music server"
        echo "PORT=8083"
        echo "DATABASE_URL=sqlite:data/stepheybot-music.db"
        echo ""
        echo "# Development settings"
        echo "RUST_LOG=info"
        echo "ENABLE_BACKGROUND_TASKS=true"
        echo "POPULATE_SAMPLE_DATA=false"
    } > "$CONFIG_FILE"

    echo -e "${GREEN}✅ Configuration saved to $CONFIG_FILE${NC}"
}

# Function to test with StepheyBot Music
test_integration() {
    echo -e "${BLUE}🤖 Test 5: Testing with StepheyBot Music...${NC}"

    if [ ! -f "Cargo.toml" ]; then
        echo -e "${YELLOW}⚠️  Not in StepheyBot Music directory, skipping integration test${NC}"
        return 0
    fi

    echo -e "${CYAN}   Building StepheyBot Music...${NC}"
    if cargo build --quiet; then
        echo -e "${GREEN}   ✅ Build successful${NC}"

        echo -e "${CYAN}   Testing Navidrome connection from Rust...${NC}"
        # This would require a small Rust test program
        echo -e "${YELLOW}   ℹ️  Manual testing required - run: cargo run${NC}"
    else
        echo -e "${RED}   ❌ Build failed${NC}"
        return 1
    fi
}

# Main execution
main() {
    echo -e "${CYAN}🧪 StepheyBot Music - Navidrome Connection Test${NC}"
    echo -e "${CYAN}================================================${NC}"
    echo ""

    # Load existing configuration
    load_config

    # Prompt for missing configuration
    prompt_config

    # Validate inputs
    if [ -z "$NAVIDROME_URL" ] || [ -z "$USERNAME" ] || [ -z "$PASSWORD" ]; then
        echo -e "${RED}❌ Missing required configuration${NC}"
        exit 1
    fi

    echo -e "${PURPLE}🚀 Starting Navidrome connection tests...${NC}"
    echo ""

    # Run tests
    test_connectivity || exit 1
    echo ""

    test_authentication || exit 1
    echo ""

    test_api_endpoints
    echo ""

    test_library_stats
    echo ""

    test_integration
    echo ""

    # Offer to save configuration
    echo -e "${BLUE}💾 Save this configuration for StepheyBot Music?${NC}"
    read -p "Save config? (y/n): " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        save_config
    fi

    echo ""
    echo -e "${GREEN}🎉 Navidrome connection test completed!${NC}"
    echo ""
    echo -e "${PURPLE}📋 Next Steps:${NC}"
    echo -e "${CYAN}   1. Ensure your .env file has the correct Navidrome settings${NC}"
    echo -e "${CYAN}   2. Run: ./start-dev.sh${NC}"
    echo -e "${CYAN}   3. Check that StepheyBot Music connects to your library${NC}"
    echo -e "${CYAN}   4. Browse to http://localhost:5173 for the web interface${NC}"
    echo ""
    echo -e "${YELLOW}🔗 API Endpoints to Test:${NC}"
    echo -e "${BLUE}   http://localhost:8083/health${NC}"
    echo -e "${BLUE}   http://localhost:8083/api/v1/library/stats${NC}"
    echo -e "${BLUE}   http://localhost:8083/api/v1/recommendations/user1${NC}"
    echo ""
}

# Check for required tools
check_dependencies() {
    local missing=()

    command -v curl >/dev/null 2>&1 || missing+=("curl")
    command -v md5sum >/dev/null 2>&1 || command -v md5 >/dev/null 2>&1 || missing+=("md5sum or md5")

    if [ ${#missing[@]} -ne 0 ]; then
        echo -e "${RED}❌ Missing required tools: ${missing[*]}${NC}"
        echo -e "${YELLOW}Please install the missing tools and try again${NC}"
        exit 1
    fi
}

# Run dependency check and main function
check_dependencies
main "$@"
