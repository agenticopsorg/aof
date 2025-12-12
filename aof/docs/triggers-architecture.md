# AOF Triggers Architecture

**Version**: 1.0
**Status**: Design Proposal
**Author**: System Architect
**Date**: 2025-12-09

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Architecture Overview](#architecture-overview)
3. [Component Design](#component-design)
4. [Data Flow](#data-flow)
5. [Core Traits and Types](#core-traits-and-types)
6. [Integration Points](#integration-points)
7. [Security Model](#security-model)
8. [Implementation Guide](#implementation-guide)
9. [Future Enhancements](#future-enhancements)

---

## Executive Summary

The **aof-triggers** crate provides a unified, platform-agnostic messaging integration layer for AOF. It enables users to interact with AOF agents, tasks, and workflows through various messaging platforms (Slack, Discord, Telegram, etc.) using a consistent command interface.

**Key Design Principles**:
- **Platform Agnostic**: Single architecture supports multiple messaging platforms
- **Zero-Copy**: Leverages Rust's ownership model for efficient message handling
- **Async-First**: Built on tokio for high-concurrency scenarios
- **Type-Safe**: Strong typing for commands, responses, and platform adapters
- **Extensible**: Plugin architecture for new platforms and command types

---

## Architecture Overview

### System Context Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                      External Systems                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │  Slack   │  │ Discord  │  │ Telegram │  │  Custom  │       │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘       │
└───────┼─────────────┼─────────────┼─────────────┼──────────────┘
        │             │             │             │
        │   Webhook/API Calls       │             │
        └─────────────┼─────────────┼─────────────┘
                      │             │
        ┌─────────────▼─────────────▼─────────────┐
        │        aof-triggers (Platform Layer)     │
        │  ┌────────────────────────────────────┐ │
        │  │   TriggerRouter                    │ │
        │  │   - Command parsing                │ │
        │  │   - Platform abstraction           │ │
        │  │   - Response formatting            │ │
        │  └──────────────┬─────────────────────┘ │
        └─────────────────┼───────────────────────┘
                          │
        ┌─────────────────▼───────────────────────┐
        │    AOF Core Runtime (aof-runtime)       │
        │  ┌────────────────────────────────────┐ │
        │  │  RuntimeOrchestrator               │ │
        │  │  - Agent execution                 │ │
        │  │  - Task management                 │ │
        │  │  - Session handling                │ │
        │  └────────────┬───────────────────────┘ │
        └─────────────────┼───────────────────────┘
                          │
        ┌─────────────────▼───────────────────────┐
        │    AOF Components                       │
        │  ┌──────────┐  ┌──────────┐  ┌───────┐ │
        │  │ aof-core │  │ aof-mcp  │  │ aof-  │ │
        │  │ (Agents) │  │ (Tools)  │  │ llm   │ │
        │  └──────────┘  └──────────┘  └───────┘ │
        └─────────────────────────────────────────┘
```

### Component Hierarchy

```
aof-triggers/
├── Platform Layer (External interface)
│   ├── SlackAdapter
│   ├── DiscordAdapter
│   ├── TelegramAdapter
│   └── CustomAdapter
│
├── Command Layer (Business logic)
│   ├── CommandParser
│   ├── CommandValidator
│   ├── CommandExecutor
│   └── ResponseFormatter
│
├── Session Layer (User context)
│   ├── SessionManager
│   ├── AuthManager
│   └── RateLimiter
│
└── Integration Layer (AOF bridge)
    ├── RuntimeBridge
    ├── AgentInvoker
    ├── TaskManager
    └── FlowController
```

---

## Component Design

### 1. Platform Adapters

Each messaging platform has a dedicated adapter that implements the `TriggerPlatform` trait.

```
┌──────────────────────────────────────────────────────┐
│              TriggerPlatform Trait                   │
├──────────────────────────────────────────────────────┤
│  + receive_message() -> TriggerMessage              │
│  + send_response(response: TriggerResponse)         │
│  + platform_id() -> String                          │
│  + capabilities() -> PlatformCapabilities           │
└──────────────────────────────────────────────────────┘
                        ▲
                        │
        ┌───────────────┼───────────────┬─────────────┐
        │               │               │             │
┌───────▼──────┐ ┌─────▼──────┐ ┌─────▼──────┐ ┌───▼──────┐
│SlackAdapter  │ │Discord     │ │Telegram    │ │Custom    │
│              │ │Adapter     │ │Adapter     │ │Adapter   │
├──────────────┤ ├────────────┤ ├────────────┤ ├──────────┤
│webhook_server│ │gateway_ws  │ │bot_api     │ │http_api  │
│auth_oauth2   │ │auth_token  │ │auth_token  │ │auth_key  │
│format_blocks │ │format_embed│ │format_kbd  │ │format_md │
└──────────────┘ └────────────┘ └────────────┘ └──────────┘
```

### 2. Command Router

The central command processing engine.

```
┌────────────────────────────────────────────────┐
│           TriggerRouter                        │
├────────────────────────────────────────────────┤
│  - platform_adapters: HashMap<String, Box<..>> │
│  - command_handlers: HashMap<String, Handler>  │
│  - session_manager: Arc<SessionManager>        │
│  - runtime_bridge: Arc<RuntimeBridge>          │
├────────────────────────────────────────────────┤
│  + register_platform(adapter)                  │
│  + route_message(msg: TriggerMessage)          │
│  + execute_command(cmd: TriggerCommand)        │
│  + format_response(result) -> TriggerResponse  │
└────────────────────────────────────────────────┘
```

### 3. Session Management

Handles user sessions across platforms with context preservation.

```
┌────────────────────────────────────────────────┐
│         SessionManager                         │
├────────────────────────────────────────────────┤
│  - sessions: DashMap<SessionId, Session>       │
│  - auth_manager: Arc<AuthManager>              │
│  - rate_limiter: Arc<RateLimiter>              │
├────────────────────────────────────────────────┤
│  + create_session(user, platform) -> SessionId │
│  + get_session(id) -> Option<Session>          │
│  + update_context(id, AgentContext)            │
│  + expire_session(id)                          │
└────────────────────────────────────────────────┘
                    │
                    │ contains
                    ▼
┌────────────────────────────────────────────────┐
│            Session                             │
├────────────────────────────────────────────────┤
│  - id: SessionId                               │
│  - user: UserIdentity                          │
│  - platform: String                            │
│  - context: AgentContext                       │
│  - created_at: Instant                         │
│  - last_active: Instant                        │
│  - permissions: Permissions                    │
└────────────────────────────────────────────────┘
```

### 4. Runtime Bridge

Connects trigger commands to AOF runtime operations.

```
┌────────────────────────────────────────────────┐
│         RuntimeBridge                          │
├────────────────────────────────────────────────┤
│  - orchestrator: Arc<RuntimeOrchestrator>      │
│  - agent_registry: Arc<AgentRegistry>          │
│  - task_tracker: Arc<TaskTracker>              │
├────────────────────────────────────────────────┤
│  + invoke_agent(name, ctx) -> AofResult<..>    │
│  + create_task(spec) -> TaskHandle             │
│  + query_task_status(id) -> TaskStatus         │
│  + control_task(id, action) -> AofResult<()>   │
└────────────────────────────────────────────────┘
```

---

## Data Flow

### Message Processing Pipeline

```
1. RECEIVE
   User sends: "/agent run code-reviewer analyze main.rs"
   Platform: Slack

   │
   ▼

2. PARSE
   TriggerMessage {
     platform: "slack",
     user: "user123",
     channel: "team-dev",
     content: "/agent run code-reviewer analyze main.rs",
     timestamp: ...
   }

   │
   ▼

3. AUTHENTICATE & AUTHORIZE
   SessionManager checks:
   - Valid user session?
   - Permission to run agents?
   - Rate limit not exceeded?

   │
   ▼

4. PARSE COMMAND
   TriggerCommand {
     command_type: Agent,
     target: AgentTarget {
       operation: Run,
       agent_name: "code-reviewer",
       input: "analyze main.rs"
     },
     session_id: "sess_abc123",
     metadata: {...}
   }

   │
   ▼

5. EXECUTE
   RuntimeBridge:
   - Load agent from registry
   - Create AgentContext from session
   - Execute agent via RuntimeOrchestrator
   - Collect results

   │
   ▼

6. FORMAT RESPONSE
   TriggerResponse {
     success: true,
     message: "Analysis complete",
     data: {
       "issues": [...],
       "suggestions": [...]
     },
     format: Platform-specific (Slack blocks)
   }

   │
   ▼

7. SEND
   Platform adapter sends formatted response
   back to Slack channel
```

### Agent Invocation Flow

```
TriggerRouter                RuntimeBridge           RuntimeOrchestrator
     │                            │                         │
     │ execute_command()          │                         │
     ├───────────────────────────>│                         │
     │                            │                         │
     │                            │ invoke_agent()          │
     │                            ├────────────────────────>│
     │                            │                         │
     │                            │                         │ load_agent()
     │                            │                         ├─────────────┐
     │                            │                         │             │
     │                            │                         │<────────────┘
     │                            │                         │
     │                            │                         │ agent.execute(ctx)
     │                            │                         ├─────────────┐
     │                            │                         │             │
     │                            │      AgentResult        │<────────────┘
     │                            │<────────────────────────┤
     │                            │                         │
     │     TriggerResult          │                         │
     │<───────────────────────────┤                         │
     │                            │                         │
     │ format_response()          │                         │
     ├─────────────┐              │                         │
     │             │              │                         │
     │<────────────┘              │                         │
     │                            │                         │
     │ send via platform          │                         │
     ├──────────────────────────> Platform API
```

---

## Core Traits and Types

### 1. Platform Trait

```rust
/// Platform adapter trait for messaging integrations
#[async_trait]
pub trait TriggerPlatform: Send + Sync {
    /// Unique platform identifier (e.g., "slack", "discord")
    fn platform_id(&self) -> &str;

    /// Platform capabilities (formatting, media, etc.)
    fn capabilities(&self) -> PlatformCapabilities;

    /// Start listening for messages
    async fn start(&mut self) -> AofResult<()>;

    /// Stop the platform adapter
    async fn stop(&mut self) -> AofResult<()>;

    /// Receive a message (pulled from platform)
    async fn receive_message(&mut self) -> AofResult<Option<TriggerMessage>>;

    /// Send a response back to the platform
    async fn send_response(&self, response: TriggerResponse) -> AofResult<()>;

    /// Verify webhook signature (for webhook-based platforms)
    fn verify_signature(&self, payload: &[u8], signature: &str) -> bool {
        true // Default: no verification
    }
}
```

### 2. Message Types

```rust
/// Incoming message from a platform
#[derive(Debug, Clone)]
pub struct TriggerMessage {
    /// Platform identifier
    pub platform: String,

    /// User who sent the message
    pub user: UserIdentity,

    /// Channel/conversation identifier
    pub channel: ChannelIdentity,

    /// Message content (raw text)
    pub content: String,

    /// Message timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Platform-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Thread/reply context
    pub thread_context: Option<ThreadContext>,
}

/// User identity across platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
    /// Platform-specific user ID
    pub platform_user_id: String,

    /// Display name
    pub display_name: String,

    /// Optional email
    pub email: Option<String>,

    /// Platform this identity belongs to
    pub platform: String,
}

/// Channel/conversation identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelIdentity {
    /// Platform-specific channel ID
    pub platform_channel_id: String,

    /// Channel name/title
    pub name: String,

    /// Channel type (DM, group, public)
    pub channel_type: ChannelType,
}

/// Outgoing response to a platform
#[derive(Debug, Clone)]
pub struct TriggerResponse {
    /// Target channel for response
    pub channel: ChannelIdentity,

    /// Optional thread to reply in
    pub thread_id: Option<String>,

    /// Response message
    pub message: ResponseMessage,

    /// Success/error status
    pub status: ResponseStatus,

    /// Platform-specific formatting
    pub format_hint: FormatHint,
}

/// Response message content
#[derive(Debug, Clone)]
pub struct ResponseMessage {
    /// Plain text content
    pub text: String,

    /// Optional rich content (tables, code blocks, etc.)
    pub rich_content: Option<RichContent>,

    /// Optional attachments (files, images)
    pub attachments: Vec<Attachment>,
}
```

### 3. Command Types

```rust
/// Parsed command from a trigger message
#[derive(Debug, Clone)]
pub struct TriggerCommand {
    /// Command type
    pub command_type: CommandType,

    /// Target of the command
    pub target: CommandTarget,

    /// Session context
    pub session_id: SessionId,

    /// Command metadata
    pub metadata: CommandMetadata,
}

/// Command type enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandType {
    /// Agent operations
    Agent,
    /// Task operations
    Task,
    /// Fleet operations (future)
    Fleet,
    /// Flow operations (future)
    Flow,
    /// Help/documentation
    Help,
    /// Status/health check
    Status,
}

/// Command target with operation-specific data
#[derive(Debug, Clone)]
pub enum CommandTarget {
    /// Agent command
    Agent(AgentCommand),
    /// Task command
    Task(TaskCommand),
    /// Fleet command (future)
    Fleet(FleetCommand),
    /// Flow command (future)
    Flow(FlowCommand),
    /// Help request
    Help(HelpQuery),
    /// Status check
    Status(StatusQuery),
}

/// Agent-specific command
#[derive(Debug, Clone)]
pub struct AgentCommand {
    /// Operation (run, list, describe)
    pub operation: AgentOperation,
    /// Agent name
    pub agent_name: Option<String>,
    /// Input for the agent
    pub input: Option<String>,
    /// Configuration overrides
    pub config_overrides: HashMap<String, serde_json::Value>,
}

/// Task-specific command
#[derive(Debug, Clone)]
pub struct TaskCommand {
    /// Operation (create, status, cancel, list)
    pub operation: TaskOperation,
    /// Task ID
    pub task_id: Option<String>,
    /// Task specification (for create)
    pub task_spec: Option<String>,
}

/// Agent operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentOperation {
    Run,
    List,
    Describe,
    Stop,
}

/// Task operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskOperation {
    Create,
    Status,
    Cancel,
    List,
    Logs,
}
```

### 4. Session Types

```rust
/// User session with context preservation
#[derive(Debug, Clone)]
pub struct Session {
    /// Unique session ID
    pub id: SessionId,

    /// User identity
    pub user: UserIdentity,

    /// Platform this session is on
    pub platform: String,

    /// Current agent context
    pub context: AgentContext,

    /// Session creation time
    pub created_at: std::time::Instant,

    /// Last activity time
    pub last_active: std::time::Instant,

    /// User permissions
    pub permissions: Permissions,

    /// Rate limit state
    pub rate_limit: RateLimitState,
}

/// User permissions
#[derive(Debug, Clone, Default)]
pub struct Permissions {
    /// Can run agents
    pub can_run_agents: bool,

    /// Can manage tasks
    pub can_manage_tasks: bool,

    /// Can deploy fleets
    pub can_deploy_fleets: bool,

    /// Can control flows
    pub can_control_flows: bool,

    /// Maximum concurrent executions
    pub max_concurrent: usize,

    /// Allowed agent patterns
    pub allowed_agents: Vec<String>,
}

/// Rate limit state
#[derive(Debug, Clone)]
pub struct RateLimitState {
    /// Requests in current window
    pub requests: usize,

    /// Window start time
    pub window_start: std::time::Instant,

    /// Max requests per window
    pub max_requests: usize,

    /// Window duration (seconds)
    pub window_duration: u64,
}
```

---

## Integration Points

### 1. RuntimeOrchestrator Integration

```rust
/// Bridge to AOF runtime
pub struct RuntimeBridge {
    orchestrator: Arc<RuntimeOrchestrator>,
    agent_registry: Arc<RwLock<HashMap<String, AgentRef>>>,
    task_tracker: Arc<DashMap<String, TaskHandle>>,
}

impl RuntimeBridge {
    /// Invoke an agent by name
    pub async fn invoke_agent(
        &self,
        agent_name: &str,
        ctx: AgentContext,
    ) -> AofResult<String> {
        // 1. Look up agent in registry
        let agents = self.agent_registry.read().await;
        let agent = agents.get(agent_name)
            .ok_or_else(|| AofError::not_found(format!("Agent '{}' not found", agent_name)))?;

        // 2. Clone context for execution
        let mut execution_ctx = ctx.clone();

        // 3. Execute via orchestrator
        let result = self.orchestrator.execute_agent(
            agent.clone(),
            &mut execution_ctx,
        ).await?;

        Ok(result)
    }

    /// Create a new task
    pub async fn create_task(
        &self,
        task_spec: TaskSpec,
    ) -> AofResult<TaskHandle> {
        let task = Task::new(task_spec);
        let handle = self.orchestrator.spawn_task(task).await?;

        // Track the task
        self.task_tracker.insert(handle.id().to_string(), handle.clone());

        Ok(handle)
    }

    /// Query task status
    pub async fn query_task_status(
        &self,
        task_id: &str,
    ) -> AofResult<TaskStatus> {
        let handle = self.task_tracker.get(task_id)
            .ok_or_else(|| AofError::not_found(format!("Task '{}' not found", task_id)))?;

        Ok(handle.status().await)
    }

    /// Control a task (cancel, pause, resume)
    pub async fn control_task(
        &self,
        task_id: &str,
        action: TaskAction,
    ) -> AofResult<()> {
        let handle = self.task_tracker.get(task_id)
            .ok_or_else(|| AofError::not_found(format!("Task '{}' not found", task_id)))?;

        match action {
            TaskAction::Cancel => handle.cancel().await?,
            TaskAction::Pause => handle.pause().await?,
            TaskAction::Resume => handle.resume().await?,
        }

        Ok(())
    }
}
```

### 2. Session to AgentContext Mapping

```rust
impl Session {
    /// Convert session context to AgentContext
    pub fn to_agent_context(&self, input: String) -> AgentContext {
        let mut ctx = AgentContext::new(input);

        // Restore conversation history
        ctx.messages = self.context.messages.clone();

        // Restore state
        ctx.state = self.context.state.clone();

        // Add session metadata
        ctx.set_state("session_id", self.id.to_string()).ok();
        ctx.set_state("user_id", self.user.platform_user_id.clone()).ok();
        ctx.set_state("platform", self.platform.clone()).ok();

        ctx
    }

    /// Update session from AgentContext after execution
    pub fn update_from_context(&mut self, ctx: &AgentContext) {
        self.context.messages = ctx.messages.clone();
        self.context.state = ctx.state.clone();
        self.context.metadata = ctx.metadata.clone();
        self.last_active = std::time::Instant::now();
    }
}
```

### 3. Command Parser

```rust
/// Parse a raw message into a TriggerCommand
pub struct CommandParser;

impl CommandParser {
    pub fn parse(msg: &TriggerMessage, session_id: SessionId) -> AofResult<TriggerCommand> {
        let content = msg.content.trim();

        // Check for command prefix
        if !content.starts_with('/') {
            return Err(AofError::validation("Not a command (missing '/' prefix)"));
        }

        // Parse command structure: /command_type target [args...]
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.is_empty() {
            return Err(AofError::validation("Empty command"));
        }

        let command_name = parts[0].trim_start_matches('/');

        let (command_type, target) = match command_name {
            "agent" => Self::parse_agent_command(&parts[1..])?,
            "task" => Self::parse_task_command(&parts[1..])?,
            "fleet" => Self::parse_fleet_command(&parts[1..])?,
            "flow" => Self::parse_flow_command(&parts[1..])?,
            "help" => (CommandType::Help, CommandTarget::Help(HelpQuery::default())),
            "status" => (CommandType::Status, CommandTarget::Status(StatusQuery::default())),
            _ => return Err(AofError::validation(format!("Unknown command: {}", command_name))),
        };

        Ok(TriggerCommand {
            command_type,
            target,
            session_id,
            metadata: CommandMetadata::from_message(msg),
        })
    }

    fn parse_agent_command(args: &[&str]) -> AofResult<(CommandType, CommandTarget)> {
        if args.is_empty() {
            return Err(AofError::validation("Agent command requires operation"));
        }

        let operation = match args[0] {
            "run" => AgentOperation::Run,
            "list" => AgentOperation::List,
            "describe" => AgentOperation::Describe,
            "stop" => AgentOperation::Stop,
            _ => return Err(AofError::validation(format!("Unknown agent operation: {}", args[0]))),
        };

        let agent_name = if args.len() > 1 { Some(args[1].to_string()) } else { None };
        let input = if args.len() > 2 { Some(args[2..].join(" ")) } else { None };

        Ok((
            CommandType::Agent,
            CommandTarget::Agent(AgentCommand {
                operation,
                agent_name,
                input,
                config_overrides: HashMap::new(),
            }),
        ))
    }

    fn parse_task_command(args: &[&str]) -> AofResult<(CommandType, CommandTarget)> {
        if args.is_empty() {
            return Err(AofError::validation("Task command requires operation"));
        }

        let operation = match args[0] {
            "create" => TaskOperation::Create,
            "status" => TaskOperation::Status,
            "cancel" => TaskOperation::Cancel,
            "list" => TaskOperation::List,
            "logs" => TaskOperation::Logs,
            _ => return Err(AofError::validation(format!("Unknown task operation: {}", args[0]))),
        };

        let task_id = if args.len() > 1 { Some(args[1].to_string()) } else { None };
        let task_spec = if args.len() > 2 { Some(args[2..].join(" ")) } else { None };

        Ok((
            CommandType::Task,
            CommandTarget::Task(TaskCommand {
                operation,
                task_id,
                task_spec,
            }),
        ))
    }
}
```

---

## Security Model

### 1. Authentication Flow

```
┌──────────┐                 ┌──────────────┐                ┌──────────────┐
│ Platform │                 │ AuthManager  │                │ UserRegistry │
└────┬─────┘                 └──────┬───────┘                └──────┬───────┘
     │                              │                               │
     │ Message with user_id         │                               │
     ├─────────────────────────────>│                               │
     │                              │                               │
     │                              │ lookup_user(user_id, platform)│
     │                              ├──────────────────────────────>│
     │                              │                               │
     │                              │         UserRecord            │
     │                              │<──────────────────────────────┤
     │                              │                               │
     │                              │ verify_permissions()          │
     │                              ├─────────────┐                 │
     │                              │             │                 │
     │                              │<────────────┘                 │
     │                              │                               │
     │      AuthResult              │                               │
     │<─────────────────────────────┤                               │
     │                              │                               │
```

### 2. Authorization Model

```rust
/// Permission check for operations
pub struct AuthManager {
    user_registry: Arc<RwLock<UserRegistry>>,
    permission_cache: Arc<DashMap<String, CachedPermissions>>,
}

impl AuthManager {
    /// Authenticate and authorize a user for an operation
    pub async fn authorize(
        &self,
        user: &UserIdentity,
        operation: &CommandType,
    ) -> AofResult<Permissions> {
        // 1. Check cache
        let cache_key = format!("{}:{}", user.platform_user_id, user.platform);
        if let Some(cached) = self.permission_cache.get(&cache_key) {
            if !cached.is_expired() {
                return Ok(cached.permissions.clone());
            }
        }

        // 2. Look up user in registry
        let registry = self.user_registry.read().await;
        let user_record = registry.get_user(&user.platform_user_id, &user.platform)
            .ok_or_else(|| AofError::unauthorized("User not registered"))?;

        // 3. Check if user is active
        if !user_record.is_active {
            return Err(AofError::unauthorized("User account is disabled"));
        }

        // 4. Get permissions
        let permissions = user_record.permissions.clone();

        // 5. Verify operation is allowed
        match operation {
            CommandType::Agent if !permissions.can_run_agents => {
                return Err(AofError::unauthorized("Not permitted to run agents"));
            }
            CommandType::Task if !permissions.can_manage_tasks => {
                return Err(AofError::unauthorized("Not permitted to manage tasks"));
            }
            CommandType::Fleet if !permissions.can_deploy_fleets => {
                return Err(AofError::unauthorized("Not permitted to deploy fleets"));
            }
            CommandType::Flow if !permissions.can_control_flows => {
                return Err(AofError::unauthorized("Not permitted to control flows"));
            }
            _ => {}
        }

        // 6. Cache permissions
        self.permission_cache.insert(
            cache_key,
            CachedPermissions::new(permissions.clone(), std::time::Duration::from_secs(300)),
        );

        Ok(permissions)
    }
}
```

### 3. Rate Limiting

```rust
/// Rate limiter to prevent abuse
pub struct RateLimiter {
    limits: Arc<DashMap<String, RateLimitState>>,
    default_config: RateLimitConfig,
}

impl RateLimiter {
    /// Check if request is allowed
    pub fn check_rate_limit(
        &self,
        user_id: &str,
        platform: &str,
    ) -> AofResult<()> {
        let key = format!("{}:{}", user_id, platform);

        let mut state = self.limits.entry(key)
            .or_insert_with(|| RateLimitState::new(&self.default_config));

        let now = std::time::Instant::now();

        // Reset window if expired
        if now.duration_since(state.window_start).as_secs() >= state.window_duration {
            state.requests = 0;
            state.window_start = now;
        }

        // Check limit
        if state.requests >= state.max_requests {
            return Err(AofError::rate_limited(
                format!("Rate limit exceeded: {} requests per {} seconds",
                    state.max_requests,
                    state.window_duration
                )
            ));
        }

        // Increment counter
        state.requests += 1;

        Ok(())
    }
}
```

### 4. Sensitive Data Handling

**Principles**:
- Never log user inputs that might contain secrets
- Redact sensitive fields in responses
- Encrypt session data at rest
- Use secure channels for webhook communication

```rust
/// Sanitize response for logging
fn sanitize_for_logging(response: &TriggerResponse) -> String {
    let mut sanitized = response.clone();

    // Redact potential secrets
    if let Some(ref mut rich) = sanitized.message.rich_content {
        rich.redact_secrets();
    }

    serde_json::to_string_pretty(&sanitized)
        .unwrap_or_else(|_| "<failed to sanitize>".to_string())
}
```

---

## Implementation Guide

### Directory Structure

```
aof/crates/aof-triggers/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                 # Public API exports
│   │
│   ├── platform/              # Platform adapters
│   │   ├── mod.rs
│   │   ├── trait.rs           # TriggerPlatform trait
│   │   ├── slack.rs           # Slack adapter
│   │   ├── discord.rs         # Discord adapter
│   │   ├── telegram.rs        # Telegram adapter
│   │   └── custom.rs          # Custom/HTTP adapter
│   │
│   ├── command/               # Command processing
│   │   ├── mod.rs
│   │   ├── parser.rs          # Command parser
│   │   ├── validator.rs       # Command validation
│   │   ├── executor.rs        # Command execution
│   │   └── types.rs           # Command types
│   │
│   ├── session/               # Session management
│   │   ├── mod.rs
│   │   ├── manager.rs         # SessionManager
│   │   ├── auth.rs            # AuthManager
│   │   ├── rate_limit.rs      # RateLimiter
│   │   └── types.rs           # Session types
│   │
│   ├── bridge/                # AOF runtime integration
│   │   ├── mod.rs
│   │   ├── runtime.rs         # RuntimeBridge
│   │   ├── agent.rs           # Agent invocation
│   │   └── task.rs            # Task management
│   │
│   ├── router.rs              # TriggerRouter
│   ├── response.rs            # Response formatting
│   ├── error.rs               # Trigger-specific errors
│   └── config.rs              # Configuration types
│
├── examples/
│   ├── slack_bot.rs           # Slack integration example
│   ├── discord_bot.rs         # Discord integration example
│   └── webhook_server.rs      # Generic webhook server
│
└── tests/
    ├── integration_tests.rs   # Integration tests
    ├── command_parsing.rs     # Command parser tests
    └── session_tests.rs       # Session tests
```

### Cargo.toml

```toml
[package]
name = "aof-triggers"
version = "0.1.0"
edition = "2021"
authors = ["AOF Contributors"]
license = "Apache-2.0"
description = "Unified messaging trigger system for AOF"

[dependencies]
# Workspace dependencies
aof-core = { workspace = true }
aof-runtime = { workspace = true }

# Async runtime
tokio = { workspace = true }
async-trait = { workspace = true }
futures = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# HTTP/Networking
reqwest = { workspace = true }
hyper = { workspace = true }
tower = { workspace = true }

# Error handling
thiserror = { workspace = true }
anyhow = { workspace = true }

# Logging
tracing = { workspace = true }

# Performance
dashmap = { workspace = true }
parking_lot = { workspace = true }

# Utilities
uuid = { workspace = true }
chrono = { workspace = true }
bytes = { workspace = true }

# Platform SDKs (optional)
slack-morphism = { version = "2.0", optional = true }
serenity = { version = "0.12", optional = true }
teloxide = { version = "0.12", optional = true }

# Crypto for signature verification
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"

[dev-dependencies]
tokio-test = "0.4"
mockito = "1.2"

[features]
default = ["slack", "discord", "telegram"]
slack = ["dep:slack-morphism"]
discord = ["dep:serenity"]
telegram = ["dep:teloxide"]
all-platforms = ["slack", "discord", "telegram"]
```

### Usage Example

```rust
use aof_triggers::{TriggerRouter, platform::SlackAdapter};
use aof_runtime::RuntimeOrchestrator;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize AOF runtime
    let orchestrator = Arc::new(RuntimeOrchestrator::new(/* config */));

    // 2. Create trigger router
    let mut router = TriggerRouter::builder()
        .with_orchestrator(orchestrator)
        .with_session_timeout(std::time::Duration::from_secs(3600))
        .with_rate_limit(100, std::time::Duration::from_secs(60))
        .build()?;

    // 3. Register platform adapters
    let slack = SlackAdapter::new(
        std::env::var("SLACK_BOT_TOKEN")?,
        std::env::var("SLACK_SIGNING_SECRET")?,
    );
    router.register_platform(Box::new(slack)).await?;

    // 4. Start router
    router.start().await?;

    // 5. Listen for messages
    loop {
        if let Some(msg) = router.receive_message().await? {
            // Messages are automatically processed by the router
            tracing::info!("Processed message from {}", msg.user.display_name);
        }
    }
}
```

---

## Future Enhancements

### Phase 2: Advanced Features

1. **Multi-Agent Conversations**
   - Support for agent-to-agent communication via triggers
   - Collaborative agent workflows initiated from chat

2. **Streaming Responses**
   - Real-time streaming of agent outputs to chat
   - Progress indicators for long-running operations

3. **Interactive Buttons/Menus**
   - Platform-specific interactive elements
   - Inline command confirmation/cancellation

4. **Voice Integration**
   - Voice command parsing
   - Text-to-speech responses

### Phase 3: Advanced Orchestration

1. **AgentFleet Support**
   - Deploy and manage agent fleets via chat
   - Fleet scaling and health monitoring

2. **AgentFlow Control**
   - Start/pause/resume complex workflows
   - Flow visualization in chat

3. **Scheduled Triggers**
   - Cron-like scheduling via chat commands
   - Recurring agent executions

### Phase 4: Analytics & Insights

1. **Usage Analytics**
   - Per-user/per-channel metrics
   - Popular agents and commands

2. **Performance Monitoring**
   - Response time tracking
   - Error rate monitoring

3. **Cost Tracking**
   - LLM token usage per user
   - Cost allocation by team/channel

---

## Architecture Decision Records

### ADR-001: Platform-Agnostic Design

**Status**: Accepted
**Context**: Need to support multiple messaging platforms with minimal code duplication
**Decision**: Use trait-based adapter pattern with unified command model
**Consequences**:
- ✅ Easy to add new platforms
- ✅ Single command parsing logic
- ❌ Platform-specific features require abstraction

### ADR-002: Session Management

**Status**: Accepted
**Context**: Users need context preservation across multiple interactions
**Decision**: In-memory session store with TTL-based expiration
**Consequences**:
- ✅ Fast session access
- ✅ Automatic cleanup
- ❌ Sessions lost on restart (future: persistent backend)

### ADR-003: Command Syntax

**Status**: Accepted
**Context**: Need consistent, discoverable command syntax
**Decision**: Slash-command style: `/command_type operation [args...]`
**Consequences**:
- ✅ Familiar to users (similar to Slack/Discord)
- ✅ Easy to parse
- ❌ Limited to text-based commands (future: interactive UI)

### ADR-004: Security Model

**Status**: Accepted
**Context**: Need to control access to AOF operations
**Decision**: User registry with role-based permissions + rate limiting
**Consequences**:
- ✅ Fine-grained access control
- ✅ Protection against abuse
- ❌ Requires initial user registration setup

---

## Glossary

- **Platform Adapter**: Component that interfaces with a specific messaging platform
- **Trigger**: An incoming message that initiates an AOF operation
- **Session**: A stateful user context that preserves conversation history
- **Command**: A parsed trigger message with structured operation data
- **RuntimeBridge**: Component that connects triggers to AOF runtime operations
- **TriggerRouter**: Central dispatcher that routes messages to appropriate handlers

---

## References

- [AOF Core Documentation](./aof-core.md)
- [AOF Runtime Documentation](./aof-runtime.md)
- [MCP Integration Guide](./mcp-integration.md)
- [Slack API Documentation](https://api.slack.com/)
- [Discord API Documentation](https://discord.com/developers/docs)
- [Telegram Bot API](https://core.telegram.org/bots/api)

---

**Document Version**: 1.0
**Last Updated**: 2025-12-09
**Next Review**: 2025-12-23
