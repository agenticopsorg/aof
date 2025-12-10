# AOF Desktop GUI - Completion Specification

## Overview
Complete the Tauri desktop application by wiring up all features with the AOF backend.

## Current Status (80% Complete)
✅ Tauri v2 + React + TypeScript foundation
✅ Agent execution UI (run, stop, monitor)
✅ Configuration editor with YAML validation
✅ Basic command handlers (agent, config, mcp)
✅ State management setup
✅ Responsive UI with Tailwind CSS

## Remaining Features (20% to 100%)

### 1. MCP Tools Browser (Priority: HIGH)
**Location:** `ui/src/App.tsx` - MCP tab
**Backend:** `src/commands/mcp.rs`

**Requirements:**
- List connected MCP servers with status indicators
- Connect/disconnect to MCP servers via stdio/HTTP
- Browse available tools from each server
- Display tool schemas (name, description, parameters)
- Test tool execution with parameter input
- Show tool execution results
- Persist MCP connections across sessions

**UI Components:**
```typescript
- MCPServerList: List of connected servers
- MCPToolBrowser: Tool catalog with search/filter
- MCPToolTester: Interactive tool testing panel
- MCPConnectionDialog: Server connection form
```

### 2. LLM Provider Management (Priority: HIGH)
**Location:** New Settings tab
**Backend:** Integrate with `aof-llm` crate

**Requirements:**
- Provider selector (OpenAI, Anthropic, Ollama, Groq, AWS Bedrock)
- API key configuration per provider
- Model selection dropdown (filtered by provider)
- Test connection button with validation
- Save/load provider configurations
- Default provider setting
- Secure key storage (use Tauri's secure storage)

**UI Components:**
```typescript
- ProviderSelector: Dropdown with provider logos
- APIKeyInput: Secure password input with toggle
- ModelSelector: Filtered model list
- ConnectionTester: Validate API keys
```

### 3. Real-time Streaming UI (Priority: HIGH)
**Location:** Enhance agent output panel
**Backend:** Hook into streaming events from `aof-llm`

**Requirements:**
- Token-by-token streaming display
- Streaming progress indicator
- Token count meter (input/output)
- Cost estimation (based on model pricing)
- Pause/resume streaming
- Stream speed indicator (tokens/sec)
- Syntax highlighting for code blocks

**UI Components:**
```typescript
- StreamingOutput: Token-by-token renderer
- TokenMeter: Visual token usage display
- CostEstimator: Real-time cost calculator
- StreamControls: Pause/resume/cancel
```

### 4. Memory/Context Viewer (Priority: MEDIUM)
**Location:** New Memory tab
**Backend:** `aof-memory` crate integration

**Requirements:**
- Display agent conversation history
- Show memory entries with timestamps
- Search/filter memory entries
- Clear memory option
- Memory size visualization
- Export memory to JSON/MD
- Memory statistics (entries, tokens used)

**UI Components:**
```typescript
- MemoryTimeline: Chronological conversation view
- MemorySearch: Search and filter UI
- MemoryStats: Usage statistics dashboard
- MemoryExporter: Export dialog
```

### 5. Platform Integrations UI (Priority: MEDIUM)
**Location:** New Integrations tab
**Backend:** `aof-triggers` crate

**Requirements:**
- Slack bot configuration (webhook URL, token)
- Telegram bot setup (bot token, chat ID)
- WhatsApp integration setup
- Webhook URL display and testing
- Integration status indicators
- Test message sending
- Integration logs viewer

**UI Components:**
```typescript
- IntegrationCard: Per-platform setup card
- WebhookTester: Test webhook endpoints
- IntegrationLogs: Activity log viewer
- ConnectionStatus: Real-time status
```

### 6. AgentFlow Visual Editor (Priority: LOW)
**Location:** New Flows tab
**Backend:** New `agentflow` commands

**Requirements:**
- Drag-drop node canvas (using @xyflow/react)
- Node types: Agent, Trigger, Condition, Action
- Connect nodes to create workflows
- Save/load flow definitions
- Execute flow and visualize progress
- Flow validation
- Export flow as YAML

**Dependencies:** Need to add `@xyflow/react` to package.json

**UI Components:**
```typescript
- FlowCanvas: React Flow editor
- NodePalette: Available node types
- FlowControls: Save/load/execute
- FlowValidator: Visual validation feedback
```

### 7. Settings Panel (Priority: HIGH)
**Location:** Expand Settings button to full tab
**Backend:** `src/state.rs` - AppSettings

**Requirements:**
- General settings (theme, log level)
- API keys management for all providers
- Default model/temperature settings
- Auto-save toggle
- Import/export settings
- Reset to defaults option
- Keyboard shortcuts configuration

**UI Components:**
```typescript
- SettingsSection: Collapsible setting groups
- KeyValueEditor: Key-value pair editor
- ThemeSelector: Dark/light/auto
- ShortcutEditor: Keyboard shortcut config
```

### 8. Error Handling & Notifications (Priority: HIGH)
**Location:** Add global notification system
**Backend:** Tauri events for error broadcasting

**Requirements:**
- Toast notifications (success, error, warning, info)
- Error boundary for React components
- Global error handler for Tauri commands
- Notification queue with auto-dismiss
- Copy error to clipboard
- Error reporting option

**Dependencies:** Consider adding `react-hot-toast` or `sonner`

**UI Components:**
```typescript
- ToastContainer: Global notification container
- ErrorBoundary: React error boundary
- ErrorDialog: Detailed error modal
```

### 9. Agent Templates Library (Priority: MEDIUM)
**Location:** New Templates tab
**Backend:** Embedded template YAMLs

**Requirements:**
- Pre-built agent templates (K8s helper, code reviewer, etc.)
- Template categories (DevOps, Development, Support)
- Template preview with description
- One-click template loading
- Custom template saving
- Template sharing/export
- Search and filter templates

**UI Components:**
```typescript
- TemplateGallery: Grid of template cards
- TemplatePreview: Template details modal
- TemplateEditor: Customize before use
- TemplateTags: Category and tag filters
```

### 10. System Monitoring Dashboard (Priority: LOW)
**Location:** New Monitoring tab
**Backend:** Collect metrics from orchestrator

**Requirements:**
- Active agents count
- Total token usage (input/output)
- Cost tracking by provider
- Average response time
- Success/failure rate
- Historical metrics charts
- Export metrics to CSV

**Dependencies:** Consider adding `recharts` or `chart.js`

**UI Components:**
```typescript
- MetricCards: Key metrics display
- UsageChart: Token/cost over time
- AgentActivityLog: Recent agent runs
- CostBreakdown: Cost by model/provider
```

## Implementation Priority

### Phase 1: Core Features (Target: Day 1)
1. LLM Provider Management
2. Real-time Streaming UI
3. Settings Panel
4. Error Handling & Notifications

### Phase 2: Extended Features (Target: Day 2)
5. MCP Tools Browser
6. Memory/Context Viewer
7. Agent Templates Library

### Phase 3: Advanced Features (Target: Day 3)
8. Platform Integrations UI
9. System Monitoring Dashboard
10. AgentFlow Visual Editor

## Testing Checklist

- [ ] All Tauri commands return proper errors
- [ ] React components handle loading states
- [ ] Form validation works correctly
- [ ] Real-time events update UI
- [ ] Settings persist across restarts
- [ ] API keys stored securely
- [ ] Navigation works smoothly
- [ ] Responsive layout on different screen sizes
- [ ] Dark theme looks good
- [ ] No console errors or warnings
- [ ] Build succeeds without errors
- [ ] E2E workflow: Create agent → Configure → Run → Monitor → Review results

## Build & Release

```bash
# Development
cd crates/aof-gui
cargo tauri dev

# Production build
cargo tauri build

# Artifacts
target/release/bundle/
  ├── dmg/      # macOS
  ├── appimage/ # Linux
  └── msi/      # Windows
```

## Success Criteria
- All 10 features implemented and working
- Clean build with zero errors
- Complete E2E test scenario passes
- Documentation updated
- Ready for v1.0.0 release
