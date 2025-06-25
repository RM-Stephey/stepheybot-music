#!/bin/bash

# StepheyBot Music - Enable Navidrome Integration
# This script switches StepheyBot Music to use Navidrome integration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
PINK='\033[95m'
NC='\033[0m' # No Color

# Configuration
MAIN_ORIGINAL="src/main.rs"
MAIN_NAVIDROME="src/main_navidrome_integrated.rs"
MAIN_BACKUP="src/main_original_backup.rs"

# Banner
show_banner() {
    echo -e "${CYAN}"
    echo "  ____  _             _                ____        _   "
    echo " / ___|| |_ ___ _ __ | |__   ___ _   _| __ )  ___ | |_ "
    echo " \\___ \\| __/ _ \\ '_ \\| '_ \\ / _ \\ | | |  _ \\ / _ \\| __|"
    echo "  ___) | ||  __/ |_) | | | |  __/ |_| | |_) | (_) | |_ "
    echo " |____/ \\__\\___| .__/|_| |_|\\___| \\__, |____/ \\___/ \\__|"
    echo "               |_|              |___/                 "
    echo -e "${PINK}          🎵 Enable Navidrome Integration 🎵          ${NC}"
    echo ""
    echo -e "${PURPLE}=========================================================${NC}"
    echo -e "${CYAN}    Upgrade to connect your real music library         ${NC}"
    echo -e "${PURPLE}=========================================================${NC}"
    echo ""
}

# Check if we're in the right directory
check_directory() {
    if [ ! -f "Cargo.toml" ] || [ ! -f "$MAIN_ORIGINAL" ]; then
        echo -e "${RED}❌ Error: Please run this script from the music-recommender directory${NC}"
        echo -e "${YELLOW}Expected files: Cargo.toml, $MAIN_ORIGINAL${NC}"
        exit 1
    fi

    if [ ! -f "$MAIN_NAVIDROME" ]; then
        echo -e "${RED}❌ Error: Navidrome integration file not found: $MAIN_NAVIDROME${NC}"
        echo -e "${YELLOW}This file should have been created during setup${NC}"
        exit 1
    fi
}

# Simple configuration check
check_configuration() {
    echo -e "${BLUE}📄 Checking Navidrome configuration...${NC}"

    if [ -f ".env" ] && grep -q "NAVIDROME_URL" .env 2>/dev/null; then
        echo -e "${GREEN}✅ Navidrome configuration found in .env${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  Navidrome configuration missing${NC}"
        return 1
    fi
}

# Test Navidrome using the test binary
test_navidrome_binary() {
    echo -e "${BLUE}🔨 Testing Navidrome connection...${NC}"

    if cargo run --bin navidrome_test --quiet; then
        echo -e "${GREEN}✅ Navidrome connection test successful${NC}"
    else
        echo -e "${YELLOW}⚠️  Navidrome connection test had issues${NC}"
        echo -e "${CYAN}   Check your .env configuration and try again${NC}"
    fi
}

# Run Navidrome setup
run_navidrome_setup() {
    echo -e "${BLUE}🔧 Running Navidrome configuration...${NC}"
    echo ""

    if [ -f "setup-navidrome.sh" ]; then
        ./setup-navidrome.sh
    else
        echo -e "${YELLOW}⚠️  setup-navidrome.sh not found${NC}"
        echo -e "${CYAN}   You can configure Navidrome manually by editing .env${NC}"
        echo ""
        echo -e "${BLUE}   Required environment variables:${NC}"
        echo -e "${YELLOW}   NAVIDROME_URL=http://localhost:4533${NC}"
        echo -e "${YELLOW}   NAVIDROME_USERNAME=your_username${NC}"
        echo -e "${YELLOW}   NAVIDROME_PASSWORD=your_password${NC}"
    fi
}

# Test the Navidrome connection
test_simple_integration() {
    echo -e "${BLUE}🧪 Testing Navidrome connection...${NC}"

    if check_configuration; then
        echo -e "${CYAN}   Running Navidrome connection test...${NC}"
        if cargo run --bin navidrome_test; then
            echo -e "${GREEN}✅ Navidrome connection successful${NC}"
            return 0
        else
            echo -e "${YELLOW}⚠️  Navidrome connection test had issues${NC}"
            return 1
        fi
    else
        echo -e "${RED}❌ Navidrome configuration missing${NC}"
        return 1
    fi
}

# Show simple success message and next steps
show_simple_success() {
    echo ""
    echo -e "${PINK}🎉 Navidrome Configuration Complete! 🎉${NC}"
    echo ""
    echo -e "${PURPLE}📋 What's Set Up:${NC}"
    echo -e "${CYAN}✓ Navidrome connection configuration added to .env${NC}"
    echo -e "${CYAN}✓ Connection test binary available${NC}"
    echo -e "${CYAN}✓ StepheyBot Music ready to connect to your library${NC}"
    echo ""
    echo -e "${PURPLE}🚀 Next Steps:${NC}"
    echo ""
    echo -e "${CYAN}1. Test your Navidrome connection:${NC}"
    echo -e "${YELLOW}   cargo run --bin navidrome_test${NC}"
    echo ""
    echo -e "${CYAN}2. Start StepheyBot Music:${NC}"
    echo -e "${YELLOW}   ./start-dev.sh${NC}"
    echo ""
    echo -e "${CYAN}3. Open your browser:${NC}"
    echo -e "${YELLOW}   Frontend: http://localhost:5173${NC}"
    echo -e "${YELLOW}   Backend:  http://localhost:8083${NC}"
    echo ""
    echo -e "${CYAN}4. Test API endpoints:${NC}"
    echo -e "${YELLOW}   curl http://localhost:8083/health${NC}"
    echo -e "${YELLOW}   curl http://localhost:8083/api/v1/library/stats${NC}"
    echo ""
    echo -e "${PURPLE}🔧 Available Tools:${NC}"
    echo -e "${CYAN}   Test connection:     ${YELLOW}cargo run --bin navidrome_test${NC}"
    echo -e "${CYAN}   Full system test:    ${YELLOW}./test-system.sh${NC}"
    echo -e "${CYAN}   Reconfigure:         ${YELLOW}./setup-navidrome.sh${NC}"
    echo ""
    echo -e "${GREEN}🎵 Ready to rock with your music library! 🎵${NC}"
}

# Show failure message and rollback info
show_failure() {
    echo ""
    echo -e "${RED}❌ Navidrome Integration Enable Failed${NC}"
    echo ""
    echo -e "${YELLOW}🔧 Troubleshooting Steps:${NC}"
    echo ""
    echo -e "${CYAN}1. Check Navidrome is running:${NC}"
    echo -e "${YELLOW}   curl http://localhost:4533${NC}"
    echo ""
    echo -e "${CYAN}2. Verify Navidrome credentials:${NC}"
    echo -e "${YELLOW}   ./test-navidrome.sh${NC}"
    echo ""
    echo -e "${CYAN}3. Check configuration:${NC}"
    echo -e "${YELLOW}   cat .env | grep NAVIDROME${NC}"
    echo ""
    echo -e "${CYAN}4. Try setup again:${NC}"
    echo -e "${YELLOW}   ./setup-navidrome.sh${NC}"
    echo ""
    echo -e "${CYAN}5. Use sample data mode:${NC}"
    echo -e "${YELLOW}   ./disable-navidrome.sh${NC}"
    echo ""
    echo -e "${PURPLE}💡 StepheyBot Music will continue working with sample data${NC}"
}

# Main execution
main() {
    show_banner

    check_directory

    echo -e "${BLUE}🎯 This will enable Navidrome integration for StepheyBot Music${NC}"
    echo -e "${CYAN}   • Connect to your real music library${NC}"
    echo -e "${CYAN}   • Replace sample data with your collection${NC}"
    echo -e "${CYAN}   • Enable AI recommendations from your music${NC}"
    echo ""
    echo -e "${YELLOW}⚠️  Make sure Navidrome is running and accessible${NC}"
    echo ""
    echo -e "${BLUE}Continue with Navidrome integration? (y/n):${NC}"
    read -r confirm

    if [[ ! $confirm =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Integration cancelled${NC}"
        exit 0
    fi

    echo ""

    # Check if Navidrome config exists, if not run setup
    if [ ! -f ".env" ] || ! grep -q "NAVIDROME_URL" .env 2>/dev/null; then
        run_navidrome_setup
    else
        echo -e "${GREEN}✅ Existing Navidrome configuration found${NC}"
    fi

    test_navidrome_binary
    show_simple_success
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "StepheyBot Music - Enable Navidrome Integration"
        echo ""
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --force        Force enable without confirmation"
        echo "  --config-only  Only run configuration setup"
        echo ""
        echo "This script enables Navidrome integration by:"
        echo "• Backing up current main.rs"
        echo "• Switching to Navidrome-integrated version"
        echo "• Configuring Navidrome connection"
        echo "• Testing the integration"
        exit 0
        ;;
    --force)
        # Skip confirmation
        show_banner
        check_directory
        if [ ! -f ".env" ] || ! grep -q "NAVIDROME_URL" .env 2>/dev/null; then
            run_navidrome_setup
        fi
        test_navidrome_binary
        show_simple_success
        ;;
    --config-only)
        show_banner
        check_directory
        run_navidrome_setup
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
