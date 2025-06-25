#!/bin/bash

# StepheyBot Music - System Test Script
# Tests backend API and frontend build to verify everything works

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🧪 StepheyBot Music - System Test${NC}"
echo -e "${CYAN}=================================${NC}"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Please run this script from the music-recommender directory${NC}"
    exit 1
fi

# Test 1: Backend Build
echo -e "${BLUE}🔧 Test 1: Building Rust Backend...${NC}"
if cargo build --quiet; then
    echo -e "${GREEN}✅ Backend builds successfully${NC}"
else
    echo -e "${RED}❌ Backend build failed${NC}"
    exit 1
fi

# Test 2: Start Backend and Test APIs
echo -e "${BLUE}🚀 Test 2: Starting Backend Server...${NC}"

# Start backend in background
cargo run > test-backend.log 2>&1 &
BACKEND_PID=$!

# Wait for backend to start
sleep 5

# Check if backend is still running
if ! kill -0 $BACKEND_PID 2>/dev/null; then
    echo -e "${RED}❌ Backend failed to start${NC}"
    echo "Backend log:"
    cat test-backend.log
    exit 1
fi

echo -e "${GREEN}✅ Backend started (PID: $BACKEND_PID)${NC}"

# Test 3: API Health Check
echo -e "${BLUE}🔍 Test 3: Testing API Health Check...${NC}"
if curl -s http://localhost:8083/health >/dev/null; then
    echo -e "${GREEN}✅ Health check endpoint working${NC}"
    HEALTH_RESPONSE=$(curl -s http://localhost:8083/health)
    echo -e "${YELLOW}   Response: $HEALTH_RESPONSE${NC}"
else
    echo -e "${RED}❌ Health check failed${NC}"
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi

# Test 4: API Status Endpoint
echo -e "${BLUE}🔍 Test 4: Testing API Status Endpoint...${NC}"
if curl -s http://localhost:8083/api/v1/status >/dev/null; then
    echo -e "${GREEN}✅ API status endpoint working${NC}"
    STATUS_RESPONSE=$(curl -s http://localhost:8083/api/v1/status | head -100)
    echo -e "${YELLOW}   Response: $STATUS_RESPONSE${NC}"
else
    echo -e "${RED}❌ API status endpoint failed${NC}"
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi

# Test 5: Library Stats
echo -e "${BLUE}🔍 Test 5: Testing Library Stats...${NC}"
if curl -s http://localhost:8083/api/v1/library/stats >/dev/null; then
    echo -e "${GREEN}✅ Library stats endpoint working${NC}"
    STATS_RESPONSE=$(curl -s http://localhost:8083/api/v1/library/stats)
    echo -e "${YELLOW}   Response: $STATS_RESPONSE${NC}"
else
    echo -e "${RED}❌ Library stats endpoint failed${NC}"
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi

# Test 6: Recommendations Endpoint
echo -e "${BLUE}🔍 Test 6: Testing Recommendations...${NC}"
if curl -s http://localhost:8083/api/v1/recommendations/user1?limit=3 >/dev/null; then
    echo -e "${GREEN}✅ Recommendations endpoint working${NC}"
    RECS_RESPONSE=$(curl -s http://localhost:8083/api/v1/recommendations/user1?limit=3 | head -200)
    echo -e "${YELLOW}   Response: $RECS_RESPONSE${NC}"
else
    echo -e "${RED}❌ Recommendations endpoint failed${NC}"
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi

# Test 7: Frontend Dependencies
echo -e "${BLUE}📦 Test 7: Checking Frontend Dependencies...${NC}"
if [ -d "frontend" ]; then
    cd frontend
    if [ ! -d "node_modules" ]; then
        echo -e "${YELLOW}   Installing frontend dependencies...${NC}"
        if npm install --legacy-peer-deps --quiet; then
            echo -e "${GREEN}✅ Frontend dependencies installed${NC}"
        else
            echo -e "${RED}❌ Failed to install frontend dependencies${NC}"
            cd ..
            kill $BACKEND_PID 2>/dev/null || true
            exit 1
        fi
    else
        echo -e "${GREEN}✅ Frontend dependencies already installed${NC}"
    fi
    cd ..
else
    echo -e "${RED}❌ Frontend directory not found${NC}"
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi

# Test 8: Frontend Build
echo -e "${BLUE}🎨 Test 8: Testing Frontend Build...${NC}"
cd frontend
if npm run build --quiet; then
    echo -e "${GREEN}✅ Frontend builds successfully${NC}"
else
    echo -e "${RED}❌ Frontend build failed${NC}"
    cd ..
    kill $BACKEND_PID 2>/dev/null || true
    exit 1
fi
cd ..

# Cleanup
echo -e "${BLUE}🧹 Cleaning up...${NC}"
kill $BACKEND_PID 2>/dev/null || true
wait $BACKEND_PID 2>/dev/null || true
rm -f test-backend.log

echo ""
echo -e "${GREEN}🎉 All tests passed! StepheyBot Music is ready to rock! 🎉${NC}"
echo ""
echo -e "${PURPLE}📋 Quick Start Commands:${NC}"
echo -e "   ${YELLOW}./start-dev.sh${NC}          - Start both backend and frontend"
echo -e "   ${YELLOW}cargo run${NC}               - Start backend only"
echo -e "   ${YELLOW}cd frontend && npm run dev${NC} - Start frontend only"
echo ""
echo -e "${PURPLE}🔗 Access URLs:${NC}"
echo -e "   ${BLUE}Backend API: http://localhost:8083${NC}"
echo -e "   ${BLUE}Frontend UI: http://localhost:5173${NC}"
echo ""
echo -e "${CYAN}🎵 Ready to experience the future of music recommendations! 🎵${NC}"
