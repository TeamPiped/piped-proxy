#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Starting SABR Test Environment${NC}"

# Function to cleanup background processes
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up...${NC}"
    if [ ! -z "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null
        echo -e "${YELLOW}Stopped server (PID: $SERVER_PID)${NC}"
    fi

    # Show server logs if they exist
    if [ -f "server.log" ]; then
        echo -e "${BLUE}📋 Server logs:${NC}"
        tail -20 server.log
    fi

    exit 0
}

# Set trap to cleanup on script exit
trap cleanup EXIT INT TERM

# Build and start the Rust server in the background
echo -e "${BLUE}🔨 Building Rust server (debug)...${NC}"
cargo build
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Failed to build server${NC}"
    exit 1
fi

echo -e "${BLUE}🌐 Starting server on port 8080...${NC}"
RUST_LOG=debug ./target/debug/piped-proxy >server.log 2>&1 &
SERVER_PID=$!

# Wait a moment for server to start
sleep 3

# Check if server is running
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo -e "${RED}❌ Server failed to start${NC}"
    if [ -f "server.log" ]; then
        echo -e "${RED}Server logs:${NC}"
        cat server.log
    fi
    exit 1
fi

echo -e "${GREEN}✅ Server started (PID: $SERVER_PID)${NC}"

# Test server connectivity
echo -e "${BLUE}🔍 Testing server connectivity...${NC}"
if curl -s http://127.0.0.1:8080 >/dev/null; then
    echo -e "${GREEN}✅ Server is responding${NC}"
else
    echo -e "${RED}❌ Server is not responding${NC}"
    exit 1
fi

# Change to sabr_test directory and run the test
echo -e "${BLUE}🧪 Running SABR test...${NC}"
cd sabr_test

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo -e "${BLUE}📦 Installing dependencies...${NC}"
    bun install
fi

# Run the test with provided arguments or defaults
echo -e "${GREEN}🎯 Starting SABR test...${NC}"
timeout 30 bun run index.ts --verbose --duration 5 "$@"
TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ SABR test completed successfully!${NC}"
elif [ $TEST_EXIT_CODE -eq 124 ]; then
    echo -e "${YELLOW}⏰ Test timed out (this might be expected)${NC}"
else
    echo -e "${RED}❌ SABR test failed with exit code: $TEST_EXIT_CODE${NC}"
fi

echo -e "${GREEN}🏁 Test completed${NC}"
