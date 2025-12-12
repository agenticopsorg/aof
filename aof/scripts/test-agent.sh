#!/bin/bash
# Quick agent test to verify MCP initialization fix
# Run after building: cargo build --release
# Then: ./scripts/test-agent.sh

set -e

echo "🧪 Testing AOF Agent with MCP Initialization Fix"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Build if needed
if [ ! -f "target/release/aofctl" ]; then
    echo -e "${YELLOW}Building release binary...${NC}"
    cargo build --release
fi

# Test 1: Agent loads successfully
echo -e "${YELLOW}Test 1: Loading K8s agent...${NC}"
if timeout 30 ./target/release/aofctl run agent testframework/k8s-agent.yaml <<< "quit" 2>&1 | grep -q "Connected to agent"; then
    echo -e "${GREEN}✓ Agent loaded successfully${NC}"
else
    echo -e "${RED}✗ Agent failed to load${NC}"
    exit 1
fi
echo ""

# Test 2: Simple query execution
echo -e "${YELLOW}Test 2: Testing query execution...${NC}"
RESPONSE=$(timeout 10 ./target/release/aofctl run agent testframework/k8s-agent.yaml "What is Kubernetes?" 2>&1)

if echo "$RESPONSE" | grep -q "Agent: k8s-helper"; then
    echo -e "${GREEN}✓ Query executed successfully${NC}"
    echo "Sample response:"
    echo "$RESPONSE" | head -5
else
    echo -e "${YELLOW}⚠️  Check if OpenAI API key is set (OPENAI_API_KEY)${NC}"
    echo "Response: $RESPONSE"
fi
echo ""

# Test 3: Check for errors
echo -e "${YELLOW}Test 3: Checking for initialization errors...${NC}"
ERROR_CHECK=$(./target/release/aofctl run agent testframework/k8s-agent.yaml "test" 2>&1 || true)

if echo "$ERROR_CHECK" | grep -q "MCP client not initialized"; then
    echo -e "${RED}✗ MCP initialization bug still present${NC}"
    exit 1
elif echo "$ERROR_CHECK" | grep -q "Tool.*execution error"; then
    echo -e "${YELLOW}⚠️  Tool execution error (may be expected without API keys)${NC}"
else
    echo -e "${GREEN}✓ No initialization errors detected${NC}"
fi
echo ""

echo -e "${GREEN}✅ Agent testing complete!${NC}"
echo ""
echo "Notes:"
echo "  - Set OPENAI_API_KEY=your-key to test with actual API"
echo "  - K8s agent uses 'openai:gpt-4' model"
echo "  - MCP server initialized before tool execution"
echo ""
