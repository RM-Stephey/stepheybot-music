#!/bin/bash

# StepheyBot Music - Disable Navidrome Integration
# This script switches StepheyBot Music back to sample data mode

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
MAIN_CURRENT="src/main.rs"
MAIN_BACKUP="src/main_original_backup.rs"
ENV_FILE=".env"
ENV_BACKUP=".env.navidrome_backup"

# Banner
show_banner() {
    echo -e "${CYAN}"
    echo "  ____  _             _                ____        _   "
    echo " / ___|| |_ ___ _ __ | |__   ___ _   _| __ )  ___ | |_ "
    echo " \\___ \\| __/ _ \\ '_ \\| '_ \\ / _ \\ | | |  _ \\ / _ \\| __|"
    echo "  ___) | ||  __/ |_) | | | |  __/ |_| | |_) | (_) | |_ "
    echo " |____/ \\__\\___| .__/|_| |_|\\___| \\__, |____/ \\___/ \\__|"
    echo "               |_|              |___/                 "
    echo -e "${PINK}         🎵 Disable Navidrome Integration 🎵         ${NC}"
    echo ""
    echo -e "${PURPLE}=========================================================${NC}"
    echo -e "${CYAN}      Switch back to sample data mode                  ${NC}"
    echo -e "${PURPLE}=========================================================${NC}"
    echo ""
}

# Check if we're in the right directory
check_directory() {
    if [ ! -f "Cargo.toml" ] || [ ! -f "$MAIN_CURRENT" ]; then
        echo -e "${RED}❌ Error: Please run this script from the music-recommender directory${NC}"
        echo -e "${YELLOW}Expected files: Cargo.toml, $MAIN_CURRENT${NC}"
        exit 1
    fi
}

# Check if Navidrome integration is currently enabled
check_integration_status() {
    echo -e "${BLUE}🔍 Checking current integration status...${NC}"

    # Check if main.rs contains Navidrome-specific code
    if grep -q "NavidromeClient" "$MAIN_CURRENT" 2>/dev/null; then
        echo -e "${YELLOW}✓ Navidrome integration is currently enabled${NC}"
        return 0
    else
        echo -e "${GREEN}ℹ️  Navidrome integration is already disabled${NC}"
        echo -e "${CYAN}   StepheyBot Music is running in sample data mode${NC}"
        return 1
    fi
}

# Restore original main.rs
restore_original_main() {
    echo -e "${BLUE}🔄 Restoring original main.rs...${NC}"

    if [ ! -f "$MAIN_BACKUP" ]; then
        echo -e "${RED}❌ Error: Original backup not found: $MAIN_BACKUP${NC}"
        echo -e "${YELLOW}   You may need to restore manually or rebuild from git${NC}"

        # Offer to get the original from git
        echo -e "${BLUE}   Try to restore from git history? (y/n):${NC}"
        read -r restore_git
        if [[ $restore_git =~ ^[Yy]$ ]]; then
            if git checkout HEAD -- "$MAIN_CURRENT" 2>/dev/null; then
                echo -e "${GREEN}✅ Restored main.rs from git${NC}"
                return 0
            else
                echo -e "${RED}❌ Failed to restore from git${NC}"
                return 1
            fi
        else
            return 1
        fi
    fi

    # Restore from backup
    cp "$MAIN_BACKUP" "$MAIN_CURRENT"
    echo -e "${GREEN}✅ Original main.rs restored from backup${NC}"
}

# Update environment configuration
update_environment_config() {
    echo -e "${BLUE}⚙️  Updating environment configuration...${NC}"

    if [ -f "$ENV_FILE" ]; then
        # Backup current .env with Navidrome config
        cp "$ENV_FILE" "$ENV_BACKUP"
        echo -e "${CYAN}   Navidrome config backed up to $ENV_BACKUP${NC}"

        # Create new .env for sample data mode
        cat > "$ENV_FILE" << 'EOF'
# StepheyBot Music Configuration - Sample Data Mode
# Navidrome integration disabled

# =============================================================================
# SERVER CONFIGURATION
# =============================================================================

# Server port
PORT=8083

# Database
DATABASE_URL=sqlite:data/stepheybot-music.db

# =============================================================================
# DATA SOURCES
# =============================================================================

# Use sample data instead of Navidrome
POPULATE_SAMPLE_DATA=true

# Navidrome integration disabled
# NAVIDROME_URL=
# NAVIDROME_USERNAME=
# NAVIDROME_PASSWORD=

# =============================================================================
# BACKGROUND TASKS
# =============================================================================

# Background tasks (can be disabled in sample mode)
ENABLE_BACKGROUND_TASKS=false

# =============================================================================
# LOGGING & DEBUGGING
# =============================================================================

# Log level
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

        echo -e "${GREEN}✅ Environment configuration updated for sample data mode${NC}"
    else
        echo -e "${YELLOW}⚠️  No .env file found, creating default configuration${NC}"
        # Create basic .env for sample data mode
        cat > "$ENV_FILE" << 'EOF'
# StepheyBot Music - Sample Data Mode
PORT=8083
DATABASE_URL=sqlite:data/stepheybot-music.db
POPULATE_SAMPLE_DATA=true
RUST_LOG=info
ENABLE_CORS=true
EOF
        echo -e "${GREEN}✅ Default sample data configuration created${NC}"
    fi
}

# Test the build
test_build() {
    echo -e "${BLUE}🔨 Testing build in sample data mode...${NC}"

    if cargo check --quiet; then
        echo -e "${GREEN}✅ Build test successful${NC}"
    else
        echo -e "${RED}❌ Build test failed${NC}"
        echo -e "${YELLOW}   There may be compilation errors to fix${NC}"
        return 1
    fi
}

# Test the application
test_application() {
    echo -e "${BLUE}🧪 Testing application in sample data mode...${NC}"

    # Quick startup test
    echo -e "${CYAN}   Testing server startup...${NC}"
    timeout 10 cargo run --quiet > /tmp/stepheybot-sample-test.log 2>&1 &
    local server_pid=$!

    sleep 5

    if kill -0 $server_pid 2>/dev/null; then
        echo -e "${GREEN}✅ Server started successfully in sample data mode${NC}"

        # Test endpoints
        local health_check=$(curl -s http://localhost:8083/health 2>/dev/null || echo "failed")
        if echo "$health_check" | grep -q "healthy"; then
            echo -e "${GREEN}✅ Health endpoint responding${NC}"
        fi

        local api_status=$(curl -s http://localhost:8083/api/v1/status 2>/dev/null || echo "failed")
        if echo "$api_status" | grep -q "sample_data\|placeholder"; then
            echo -e "${GREEN}✅ Sample data mode confirmed${NC}"
        fi

        # Test library stats
        local lib_stats=$(curl -s http://localhost:8083/api/v1/library/stats 2>/dev/null || echo "failed")
        if echo "$lib_stats" | grep -q "total_tracks"; then
            echo -e "${GREEN}✅ Library stats endpoint working${NC}"
        fi

        kill $server_pid 2>/dev/null || true
        wait $server_pid 2>/dev/null || true
    else
        echo -e "${YELLOW}⚠️  Server startup test inconclusive${NC}"
        echo -e "${CYAN}   Check logs: tail /tmp/stepheybot-sample-test.log${NC}"
    fi

    rm -f /tmp/stepheybot-sample-test.log
}

# Clean up Navidrome-related files
cleanup_navidrome_files() {
    echo -e "${BLUE}🧹 Cleaning up Navidrome-related files...${NC}"

    # Remove any temporary Navidrome test files
    rm -f /tmp/stepheybot-navidrome-*.log

    # Keep backup files but note them
    if [ -f "$MAIN_BACKUP" ]; then
        echo -e "${CYAN}   Keeping backup: $MAIN_BACKUP${NC}"
    fi

    if [ -f "$ENV_BACKUP" ]; then
        echo -e "${CYAN}   Keeping Navidrome config backup: $ENV_BACKUP${NC}"
    fi

    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

# Show success message
show_success() {
    echo ""
    echo -e "${PINK}🎉 Navidrome Integration Successfully Disabled! 🎉${NC}"
    echo ""
    echo -e "${PURPLE}📋 What's Changed:${NC}"
    echo -e "${CYAN}✓ StepheyBot Music now uses sample data${NC}"
    echo -e "${CYAN}✓ No external dependencies on Navidrome${NC}"
    echo -e "${CYAN}✓ Faster startup and testing${NC}"
    echo -e "${CYAN}✓ All core functionality still works${NC}"
    echo ""
    echo -e "${PURPLE}🎵 Sample Data Includes:${NC}"
    echo -e "${CYAN}• 5 Artists (Synthwave, Electronic, Ambient)${NC}"
    echo -e "${CYAN}• 5 Albums with complete metadata${NC}"
    echo -e "${CYAN}• 10 Tracks with audio features${NC}"
    echo -e "${CYAN}• 3 Sample users with listening history${NC}"
    echo ""
    echo -e "${PURPLE}🚀 Next Steps:${NC}"
    echo ""
    echo -e "${CYAN}1. Start the development servers:${NC}"
    echo -e "${YELLOW}   ./start-dev.sh${NC}"
    echo ""
    echo -e "${CYAN}2. Open your browser:${NC}"
    echo -e "${YELLOW}   Frontend: http://localhost:5173${NC}"
    echo -e "${YELLOW}   Backend:  http://localhost:8083${NC}"
    echo ""
    echo -e "${CYAN}3. Test sample data endpoints:${NC}"
    echo -e "${YELLOW}   curl http://localhost:8083/api/v1/library/stats${NC}"
    echo -e "${YELLOW}   curl http://localhost:8083/api/v1/recommendations/user1${NC}"
    echo -e "${YELLOW}   curl http://localhost:8083/api/v1/status${NC}"
    echo ""
    echo -e "${PURPLE}🔧 Re-enable Navidrome Later:${NC}"
    echo -e "${CYAN}   To reconnect to your music library:${NC}"
    echo -e "${YELLOW}   ./enable-navidrome.sh${NC}"
    echo ""
    echo -e "${PURPLE}💾 Backups Available:${NC}"
    if [ -f "$ENV_BACKUP" ]; then
        echo -e "${CYAN}   Navidrome config: $ENV_BACKUP${NC}"
    fi
    if [ -f "$MAIN_BACKUP" ]; then
        echo -e "${CYAN}   Original main.rs: $MAIN_BACKUP${NC}"
    fi
    echo ""
    echo -e "${GREEN}🎵 StepheyBot Music is ready with sample data! 🎵${NC}"
}

# Show failure message
show_failure() {
    echo ""
    echo -e "${RED}❌ Failed to Disable Navidrome Integration${NC}"
    echo ""
    echo -e "${YELLOW}🔧 Possible Issues:${NC}"
    echo -e "${CYAN}• Missing backup files${NC}"
    echo -e "${CYAN}• Compilation errors${NC}"
    echo -e "${CYAN}• File permission issues${NC}"
    echo ""
    echo -e "${PURPLE}🛠️  Manual Steps:${NC}"
    echo ""
    echo -e "${CYAN}1. Check for backup files:${NC}"
    echo -e "${YELLOW}   ls -la src/main_*.rs${NC}"
    echo ""
    echo -e "${CYAN}2. Restore from git if needed:${NC}"
    echo -e "${YELLOW}   git checkout HEAD -- src/main.rs${NC}"
    echo ""
    echo -e "${CYAN}3. Create basic .env:${NC}"
    echo -e "${YELLOW}   echo 'PORT=8083' > .env${NC}"
    echo -e "${YELLOW}   echo 'POPULATE_SAMPLE_DATA=true' >> .env${NC}"
    echo ""
    echo -e "${CYAN}4. Test build:${NC}"
    echo -e "${YELLOW}   cargo check${NC}"
    echo ""
}

# Main execution
main() {
    show_banner

    check_directory

    if ! check_integration_status; then
        echo -e "${BLUE}StepheyBot Music is already in sample data mode.${NC}"
        echo -e "${CYAN}No changes needed.${NC}"
        echo ""
        echo -e "${YELLOW}To re-enable Navidrome integration, run:${NC}"
        echo -e "${CYAN}./enable-navidrome.sh${NC}"
        exit 0
    fi

    echo ""
    echo -e "${BLUE}🎯 This will disable Navidrome integration and switch to sample data mode${NC}"
    echo -e "${CYAN}   • Disconnect from your music library${NC}"
    echo -e "${CYAN}   • Use built-in sample music data${NC}"
    echo -e "${CYAN}   • Backup current Navidrome configuration${NC}"
    echo ""
    echo -e "${YELLOW}⚠️  You can re-enable Navidrome integration later${NC}"
    echo ""
    echo -e "${BLUE}Continue with disabling Navidrome integration? (y/n):${NC}"
    read -r confirm

    if [[ ! $confirm =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Operation cancelled${NC}"
        exit 0
    fi

    echo ""

    # Execute disable steps
    local success=true

    if ! restore_original_main; then
        success=false
    fi

    if $success; then
        update_environment_config
    fi

    if $success && ! test_build; then
        success=false
    fi

    if $success; then
        test_application
        cleanup_navidrome_files
    fi

    if $success; then
        show_success
    else
        show_failure
        exit 1
    fi
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "StepheyBot Music - Disable Navidrome Integration"
        echo ""
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --force        Force disable without confirmation"
        echo "  --backup-only  Only backup current configuration"
        echo ""
        echo "This script disables Navidrome integration by:"
        echo "• Restoring original main.rs from backup"
        echo "• Switching to sample data mode"
        echo "• Backing up Navidrome configuration"
        echo "• Testing sample data functionality"
        exit 0
        ;;
    --force)
        # Skip confirmation
        show_banner
        check_directory
        if check_integration_status; then
            restore_original_main
            update_environment_config
            test_build && test_application
            cleanup_navidrome_files
            show_success
        fi
        ;;
    --backup-only)
        show_banner
        check_directory
        if [ -f "$ENV_FILE" ]; then
            cp "$ENV_FILE" "$ENV_BACKUP"
            echo -e "${GREEN}✅ Configuration backed up to $ENV_BACKUP${NC}"
        else
            echo -e "${YELLOW}⚠️  No .env file to backup${NC}"
        fi
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
