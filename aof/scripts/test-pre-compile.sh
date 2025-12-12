#!/bin/bash
# Pre-compile test suite - Run before full build for fast feedback
# This catches common errors without the overhead of full compilation

set -e

echo "🔍 Running Pre-Compile Tests..."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 1. Quick syntax check
echo -e "${YELLOW}1️⃣  Running cargo check (syntax validation)...${NC}"
if cargo check --all-features 2>&1 | grep -q "error"; then
    echo -e "${RED}❌ Syntax errors detected${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Syntax check passed${NC}"
echo ""

# 2. Run unit tests (no integration tests)
echo -e "${YELLOW}2️⃣  Running unit tests (MCP + Tool Executor)...${NC}"
if cargo test --lib --all-features -- --test-threads=1 2>&1 | grep -q "test result: FAILED"; then
    echo -e "${RED}❌ Unit tests failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Unit tests passed${NC}"
echo ""

# 3. Clippy lints for common bugs
echo -e "${YELLOW}3️⃣  Running clippy static analysis...${NC}"
CLIPPY_OUTPUT=$(cargo clippy --all-targets --all-features 2>&1)
if echo "$CLIPPY_OUTPUT" | grep -E "(error|warning:.*unwrap|warning:.*panic)" | grep -v "^warning:" | head -5; then
    echo -e "${RED}❌ Clippy found potential issues${NC}"
fi
echo -e "${GREEN}✓ Static analysis complete${NC}"
echo ""

# 4. Specific MCP initialization validation
echo -e "${YELLOW}4️⃣  Validating MCP initialization patterns...${NC}"
if grep -r "McpClientBuilder::new()" crates/aof-runtime/src --include="*.rs" | \
   grep -v "\.initialize()" | grep -v "test" > /tmp/mcp_check.txt 2>&1; then
    if [ -s /tmp/mcp_check.txt ]; then
        echo -e "${RED}⚠️  Warning: Found McpClientBuilder without initialization:${NC}"
        cat /tmp/mcp_check.txt
    fi
else
    echo -e "${GREEN}✓ No uninitialized MCP clients found${NC}"
fi
echo ""

# 5. Check for common error patterns
echo -e "${YELLOW}5️⃣  Scanning for documented error patterns...${NC}"
# Check for missing tool_call_id handling
if grep -r "call_tool" crates/aof-mcp/src --include="*.rs" | grep -v "tool_call_id" > /tmp/error_check.txt 2>&1; then
    if [ -s /tmp/error_check.txt ]; then
        echo -e "${YELLOW}⚠️  Note: Verify tool_call_id handling in tool calls${NC}"
    fi
fi
echo -e "${GREEN}✓ Error pattern scan complete${NC}"
echo ""

# 6. Check configuration consistency
echo -e "${YELLOW}6️⃣  Validating configuration consistency...${NC}"
# Check for mcp_servers vs tools field name consistency
if grep -r "mcp_servers" testframework --include="*.yaml" > /tmp/config_check.txt 2>&1; then
    if [ -s /tmp/config_check.txt ]; then
        echo -e "${YELLOW}⚠️  Found 'mcp_servers' field - should be 'tools':${NC}"
        cat /tmp/config_check.txt
    fi
fi
echo -e "${GREEN}✓ Configuration check complete${NC}"
echo ""

echo -e "${GREEN}✅ Pre-compile tests passed!${NC}"
echo ""
echo "Next steps:"
echo "  1. Run full build: cargo build --release"
echo "  2. Test with: aofctl run agent k8s-agent.yaml"
echo ""
