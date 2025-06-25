#!/bin/bash

# StepheyBot Music - Build and Deploy Script
# This script builds the frontend, backend, and deploys the service

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "\n${PURPLE}=== $1 ===${NC}\n"
}

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FRONTEND_DIR="$SCRIPT_DIR/frontend"
DOCKER_DIR="$(dirname "$SCRIPT_DIR")"

print_header "🎵 StepheyBot Music - Build & Deploy"

# Check if we're in the right directory
if [[ ! -f "$SCRIPT_DIR/Cargo.toml" ]]; then
    print_error "Cargo.toml not found. Are you in the right directory?"
    exit 1
fi

if [[ ! -f "$FRONTEND_DIR/package.json" ]]; then
    print_error "Frontend package.json not found at $FRONTEND_DIR"
    exit 1
fi

# Parse command line arguments
BUILD_FRONTEND=true
BUILD_BACKEND=true
DEPLOY=true
SHOW_LOGS=false
CLEAN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --frontend-only)
            BUILD_BACKEND=false
            DEPLOY=false
            shift
            ;;
        --backend-only)
            BUILD_FRONTEND=false
            DEPLOY=false
            shift
            ;;
        --no-deploy)
            DEPLOY=false
            shift
            ;;
        --logs)
            SHOW_LOGS=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --frontend-only    Only build frontend"
            echo "  --backend-only     Only build backend"
            echo "  --no-deploy        Skip Docker deployment"
            echo "  --logs             Show logs after deployment"
            echo "  --clean            Clean build artifacts first"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Clean build artifacts if requested
if [[ "$CLEAN" == true ]]; then
    print_header "🧹 Cleaning Build Artifacts"

    print_status "Cleaning Rust target directory..."
    if [[ -d "$SCRIPT_DIR/target" ]]; then
        rm -rf "$SCRIPT_DIR/target"
        print_success "Rust target directory cleaned"
    fi

    print_status "Cleaning frontend build directory..."
    if [[ -d "$FRONTEND_DIR/build" ]]; then
        rm -rf "$FRONTEND_DIR/build"
        print_success "Frontend build directory cleaned"
    fi

    if [[ -d "$FRONTEND_DIR/node_modules" ]]; then
        print_status "Cleaning node_modules..."
        rm -rf "$FRONTEND_DIR/node_modules"
        print_success "node_modules cleaned"
    fi
fi

# Build Frontend
if [[ "$BUILD_FRONTEND" == true ]]; then
    print_header "🎨 Building Frontend (Svelte)"

    cd "$FRONTEND_DIR"

    # Install dependencies
    print_status "Installing npm dependencies..."
    npm install --legacy-peer-deps
    print_success "Dependencies installed"

    # Build frontend
    print_status "Building Svelte application..."
    npm run build
    print_success "Frontend built successfully"

    # Verify build output
    if [[ -f "build/index.html" ]]; then
        print_success "Frontend build artifacts verified"
    else
        print_error "Frontend build failed - index.html not found"
        exit 1
    fi

    cd "$SCRIPT_DIR"
fi

# Build Backend
if [[ "$BUILD_BACKEND" == true ]]; then
    print_header "🦀 Building Backend (Rust)"

    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust."
        exit 1
    fi

    print_status "Building Rust application..."
    cargo build --release
    print_success "Backend built successfully"

    # Verify binary
    if [[ -f "target/release/stepheybot-music" ]]; then
        print_success "Backend binary verified"
    else
        print_error "Backend build failed - binary not found"
        exit 1
    fi
fi

# Deploy with Docker
if [[ "$DEPLOY" == true ]]; then
    print_header "🐳 Deploying with Docker"

    cd "$DOCKER_DIR"

    # Check if docker-compose.yml exists
    if [[ ! -f "docker-compose.yml" ]]; then
        print_error "docker-compose.yml not found in $DOCKER_DIR"
        exit 1
    fi

    # Stop existing container
    print_status "Stopping existing stepheybot-music container..."
    docker-compose stop stepheybot-music || true

    # Remove old container
    print_status "Removing old container..."
    docker-compose rm -f stepheybot-music || true

    # Build new image
    print_status "Building new Docker image..."
    docker-compose build stepheybot-music
    print_success "Docker image built"

    # Start the service
    print_status "Starting stepheybot-music service..."
    docker-compose up -d stepheybot-music
    print_success "Service started"

    # Wait for service to be ready
    print_status "Waiting for service to be ready..."
    sleep 10

    # Check health
    print_status "Checking service health..."
    for i in {1..30}; do
        if curl -s http://localhost:8083/health > /dev/null 2>&1; then
            print_success "Service is healthy!"
            break
        fi
        if [[ $i -eq 30 ]]; then
            print_warning "Service health check timeout"
        fi
        sleep 2
    done

    cd "$SCRIPT_DIR"
fi

# Show status
print_header "📊 Service Status"

# Check if service is running
if docker ps | grep -q "stepheybot_music_brain"; then
    print_success "StepheyBot Music container is running"

    # Get container info
    CONTAINER_ID=$(docker ps --format "table {{.ID}}\t{{.Names}}" | grep stepheybot_music_brain | awk '{print $1}')
    print_status "Container ID: $CONTAINER_ID"

    # Check endpoints
    print_status "Testing endpoints..."

    if curl -s http://localhost:8083/health > /dev/null; then
        print_success "✓ Health endpoint responding"
    else
        print_warning "✗ Health endpoint not responding"
    fi

    if curl -s http://localhost:8083/api/v1/status > /dev/null; then
        print_success "✓ API status endpoint responding"
    else
        print_warning "✗ API status endpoint not responding"
    fi

    if curl -s http://localhost:8083/api/v1/library/stats > /dev/null; then
        print_success "✓ Library stats endpoint responding"
    else
        print_warning "✗ Library stats endpoint not responding"
    fi

else
    print_error "StepheyBot Music container is not running"

    # Show recent logs
    print_status "Recent logs:"
    docker-compose logs --tail=20 stepheybot-music
fi

# Show logs if requested
if [[ "$SHOW_LOGS" == true ]]; then
    print_header "📝 Service Logs"
    docker-compose logs -f stepheybot-music
fi

print_header "✨ Build and Deploy Complete!"

echo -e "${CYAN}🎵 StepheyBot Music Endpoints:${NC}"
echo -e "  ${GREEN}Frontend:${NC}     http://localhost:8083/"
echo -e "  ${GREEN}Health:${NC}       http://localhost:8083/health"
echo -e "  ${GREEN}API Status:${NC}   http://localhost:8083/api/v1/status"
echo -e "  ${GREEN}Library Stats:${NC} http://localhost:8083/api/v1/library/stats"
echo -e "  ${GREEN}Admin Panel:${NC}  http://localhost:8083/admin/"

echo -e "\n${YELLOW}💡 Tips:${NC}"
echo -e "  • Use ${BLUE}--logs${NC} to follow service logs"
echo -e "  • Use ${BLUE}--clean${NC} to clean build artifacts"
echo -e "  • Use ${BLUE}--frontend-only${NC} for quick frontend iterations"
echo -e "  • Check ${BLUE}docker-compose logs stepheybot-music${NC} for troubleshooting"

echo -e "\n${PURPLE}🚀 Ready to rock, Stephey!${NC}"
