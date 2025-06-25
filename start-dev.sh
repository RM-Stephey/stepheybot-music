#!/bin/bash

# StepheyBot Music - Development Startup Script
# This script starts both the Rust backend and Svelte frontend in development mode

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
echo "  ____  _             _                ____        _   "
echo " / ___|| |_ ___ _ __ | |__   ___ _   _| __ )  ___ | |_ "
echo " \___ \| __/ _ \ '_ \| '_ \ / _ \ | | |  _ \ / _ \| __|"
echo "  ___) | ||  __/ |_) | | | |  __/ |_| | |_) | (_) | |_ "
echo " |____/ \__\___| .__/|_| |_|\___|\__, |____/ \___/ \__|"
echo "               |_|              |___/                 "
echo -e "${PURPLE}                    🎵 Music AI 🎵                    ${NC}"
echo ""

# Function to cleanup background processes
cleanup() {
    echo -e "\n${YELLOW}🛑 Shutting down servers...${NC}"

    # Kill background jobs
    jobs -p | xargs -r kill 2>/dev/null || true

    # Kill any remaining processes
    pkill -f "stepheybot-music" 2>/dev/null || true
    pkill -f "vite.*5173" 2>/dev/null || true

    echo -e "${GREEN}✅ Cleanup complete${NC}"
    exit 0
}

# Set trap for cleanup
trap cleanup SIGINT SIGTERM

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Please run this script from the music-recommender directory${NC}"
    echo -e "${YELLOW}Expected files: Cargo.toml, frontend/package.json${NC}"
    exit 1
fi

# Check if frontend directory exists
if [ ! -d "frontend" ]; then
    echo -e "${RED}❌ Error: Frontend directory not found${NC}"
    exit 1
fi

# Create data directory if it doesn't exist
mkdir -p data

echo -e "${BLUE}🔧 Starting StepheyBot Music Development Environment...${NC}"
echo ""

# Start the Rust backend
echo -e "${GREEN}🚀 Starting Rust Backend Server (Port 8083)...${NC}"
export RUST_LOG="stepheybot_music=info,tower_http=debug,sqlx=warn"
export DATABASE_URL="sqlite:data/stepheybot-music.db"
export PORT="8083"

cargo run > backend.log 2>&1 &
BACKEND_PID=$!

# Wait a moment for backend to start
sleep 3

# Check if backend started successfully
if ! kill -0 $BACKEND_PID 2>/dev/null; then
    echo -e "${RED}❌ Backend failed to start. Check backend.log for details.${NC}"
    tail -n 10 backend.log
    exit 1
fi

# Test backend health
echo -e "${YELLOW}🔍 Testing backend connection...${NC}"
if curl -s http://localhost:8083/health >/dev/null; then
    echo -e "${GREEN}✅ Backend is running and healthy${NC}"
else
    echo -e "${YELLOW}⚠️  Backend health check failed, but continuing...${NC}"
fi

echo ""

# Start the Svelte frontend
echo -e "${GREEN}🎨 Starting Svelte Frontend Server (Port 5173)...${NC}"
cd frontend

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}📦 Installing frontend dependencies...${NC}"
    npm install --legacy-peer-deps
fi

# Start the frontend dev server
npm run dev > ../frontend.log 2>&1 &
FRONTEND_PID=$!
cd ..

# Wait a moment for frontend to start
sleep 5

# Check if frontend started successfully
if ! kill -0 $FRONTEND_PID 2>/dev/null; then
    echo -e "${RED}❌ Frontend failed to start. Check frontend.log for details.${NC}"
    tail -n 10 frontend.log
    cleanup
    exit 1
fi

echo -e "${GREEN}✅ Frontend development server started${NC}"
echo ""

# Display access information
echo -e "${CYAN}🌟 StepheyBot Music Development Environment Ready! 🌟${NC}"
echo ""
echo -e "${PURPLE}📡 Backend API Server:${NC}"
echo -e "   🔗 Health Check: ${BLUE}http://localhost:8083/health${NC}"
echo -e "   🔗 API Status:   ${BLUE}http://localhost:8083/api/v1/status${NC}"
echo -e "   🔗 Library Stats: ${BLUE}http://localhost:8083/api/v1/library/stats${NC}"
echo ""
echo -e "${PURPLE}🎨 Frontend Web Interface:${NC}"
echo -e "   🔗 Local Access:  ${BLUE}http://localhost:5173${NC}"
echo -e "   🔗 Network Access: ${BLUE}http://$(hostname -I | awk '{print $1}'):5173${NC}"
echo ""
echo -e "${PURPLE}📋 Quick API Tests:${NC}"
echo -e "   ${YELLOW}curl http://localhost:8083/health${NC}"
echo -e "   ${YELLOW}curl http://localhost:8083/api/v1/library/stats${NC}"
echo -e "   ${YELLOW}curl http://localhost:8083/api/v1/recommendations/user1${NC}"
echo ""
echo -e "${PURPLE}📄 Log Files:${NC}"
echo -e "   Backend: ${BLUE}backend.log${NC}"
echo -e "   Frontend: ${BLUE}frontend.log${NC}"
echo ""
echo -e "${GREEN}🎵 Ready to rock with AI-powered music recommendations! 🎵${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop both servers${NC}"
echo ""

# Wait for user to stop the servers
wait $BACKEND_PID $FRONTEND_PID
