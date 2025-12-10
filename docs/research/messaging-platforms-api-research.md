# Messaging Platform APIs Research Report for AOF Integration

**Date:** December 9, 2025
**Researcher:** Hive Mind Research Agent
**Swarm ID:** swarm-1765276942953-cr5mbgm0i
**Task:** Comprehensive analysis of messaging platform APIs for AOF integration

---

## Executive Summary

This report provides a comprehensive analysis of four major messaging platform APIs (WhatsApp, Telegram, Slack, Discord) for integration with the Agent Orchestration Framework (AOF). Each platform offers unique capabilities, authentication mechanisms, and integration patterns suitable for building intelligent agent-driven messaging applications.

**Key Findings:**
- **WhatsApp Business API** requires Meta Business account and offers both Cloud and On-Premise options
- **Telegram Bot API** provides the simplest authentication with bot tokens and flexible webhook/polling options
- **Slack Events API** offers Socket Mode for firewall-friendly integration and rich OAuth scopes
- **Discord Gateway API** requires privileged intents for message content access and uses WebSocket connections
- **Rust ecosystem** has mature crates for Telegram and emerging support for other platforms

---

## 1. WhatsApp Business API

### 1.1 Authentication

**Cloud API vs On-Premise:**
- **Cloud API** (Recommended for 2025): Hosted on Meta's infrastructure, no server maintenance required
- **On-Premise**: Self-hosted solution with more control but higher maintenance burden
- Meta is steering businesses toward Cloud API for faster setup and fewer technical issues

**Authentication Method:**
- **Access Token**: Long-lived API key for backend authentication
- **Temporary Token**: Available from Developer Portal > WhatsApp > Getting Started
- **Permanent Token**: Generated via System User creation (recommended for production)
- **Phone Number ID**: Required identifier found in Developer Portal

**Prerequisites:**
- Meta Business account (Business Manager) with admin access
- Meta Developer account
- Phone number capable of receiving SMS/voice for verification
- Public HTTPS endpoint for webhooks

### 1.2 Webhook Setup

**Configuration Process:**
1. Navigate to Developer Portal > WhatsApp > Configuration
2. Subscribe webhook URL on your controlled server
3. Verify webhook by echoing back challenge code (GET request)
4. Select event subscriptions (messages, message status, etc.)
5. Set VERIFY_TOKEN environment variable for validation

**Webhook URL Format:**
```
https://<your-host-url>/webhook
```

**Verification Requirements:**
- Must respond to GET request with `hub.challenge` parameter
- Must return HTTP 200 for all webhook POSTs
- Must have valid SSL certificate (HTTPS required)

### 1.3 Message Formats

**Webhook JSON Structure:**
```json
{
  "object": "whatsapp_business_account",
  "entry": [{
    "changes": [{
      "field": "messages",
      "value": {
        "messaging_product": "whatsapp",
        "metadata": {
          "display_phone_number": "15551234567",
          "phone_number_id": "123456789"
        },
        "messages": [{
          "from": "15559876543",
          "id": "wamid.XXX==",
          "timestamp": "1234567890",
          "type": "text",
          "text": {
            "body": "Hello, agent!"
          }
        }],
        "contacts": [{
          "profile": {
            "name": "John Doe"
          },
          "wa_id": "15559876543"
        }]
      }
    }]
  }]
}
```

**Message Types:**
- **Text messages**: Simple text content
- **Interactive messages**: Buttons, lists, and reply buttons
- **Templates**: Pre-approved message templates
- **Media**: Images, documents, audio, video
- **Status updates**: Delivery, read receipts, errors

**Interactive Message Response:**
```json
{
  "type": "interactive",
  "interactive": {
    "type": "button_reply",
    "button_reply": {
      "id": "button-1",
      "title": "Click me"
    }
  },
  "context": {
    "from": "15551234567",
    "id": "wamid.original_message"
  }
}
```

### 1.4 Rate Limits and Best Practices

**Pricing (as of July 1, 2025):**
- Per-message pricing model
- Categories: Marketing, Utility, Authentication, Service
- **Service messages** (user-initiated support): FREE
- Costs vary by message category and region

**Rate Limits:**
- Start with conservative limits, increase with tier upgrades
- Rate limits based on conversation categories
- Monitor for 429 errors and implement exponential backoff

**Best Practices:**
- Use message templates for marketing messages
- Leverage service conversations for cost efficiency
- Implement webhook signature verification
- Return HTTP 200 immediately to acknowledge receipt
- Process messages asynchronously to avoid timeouts
- Store webhook payloads for retry logic

### 1.5 Recommended Rust Crate

**Crate:** `whatsapp-cloud-api`
**Repository:** [https://crates.io/crates/whatsapp-cloud-api](https://crates.io/crates/whatsapp-cloud-api)

**Status:** Available on crates.io
**Maturity:** Early stage, limited documentation
**Alternative:** Build custom HTTP client using `reqwest` and `serde_json`

### 1.6 Required Credentials and Setup

**Production Setup:**
1. Create Meta Business Account
2. Create Meta App in Developer Portal
3. Add WhatsApp product to app
4. Configure webhook URL and verify token
5. Generate permanent access token via System User
6. Store credentials securely:
   - `WHATSAPP_ACCESS_TOKEN`
   - `WHATSAPP_PHONE_NUMBER_ID`
   - `WHATSAPP_WEBHOOK_VERIFY_TOKEN`
   - `WHATSAPP_BUSINESS_ACCOUNT_ID`

### 1.7 Example Trigger Patterns

```
User: /agent run daily-report
Response: "Generating your daily report..."

User: @agent task create "Review documentation"
Response: "Task created: Review documentation (#123)"

User: "Show my tasks"
Response: Interactive list with buttons for each task
```

---

## 2. Telegram Bot API

### 2.1 Authentication

**Bot Token Management:**
- Create bot via [@BotFather](https://t.me/botfather)
- Receive unique bot token (format: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)
- Token acts as both authentication and authorization
- No OAuth flow required - simplest authentication model

**Bot Creation Process:**
1. Message @BotFather on Telegram
2. Use `/newbot` command
3. Choose bot name and username
4. Receive bot token immediately
5. Configure bot settings (description, commands, etc.)

### 2.2 getUpdates vs Webhooks

**Two Mutually Exclusive Methods:**

**Method 1: getUpdates (Long Polling)**
- Bot connects to Telegram servers
- Retrieves up to 100 unconfirmed updates
- Use `offset` parameter to confirm updates
- Suitable for development and low-traffic bots
- No public endpoint required

**Method 2: Webhooks (Push)**
- Telegram connects to your server
- Sends HTTPS POST with JSON-serialized Update
- Real-time, efficient for production
- Requires public HTTPS endpoint

**Critical Limitation:**
- **Cannot use both simultaneously**
- Using getUpdates while webhook is active returns 409 error
- Must call `deleteWebhook` before switching to polling

**Webhook Configuration:**
```bash
curl -X POST "https://api.telegram.org/bot<TOKEN>/setWebhook" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://yourserver.com/webhook",
    "secret_token": "your-secret-token"
  }'
```

**Supported Webhook Ports:** 443, 80, 88, 8443
**IP Ranges:** 149.154.160.0/20 and 91.108.4.0/22
**TLS Requirements:** TLS 1.2+ required

### 2.3 Message Types and Reply Keyboards

**Update JSON Structure:**
```json
{
  "update_id": 123456789,
  "message": {
    "message_id": 1234,
    "from": {
      "id": 987654321,
      "is_bot": false,
      "first_name": "John",
      "username": "johndoe"
    },
    "chat": {
      "id": 987654321,
      "first_name": "John",
      "username": "johndoe",
      "type": "private"
    },
    "date": 1234567890,
    "text": "/agent run task"
  }
}
```

**Message Types:**
- **text**: Regular text messages
- **photo**: Images with optional caption
- **document**: Files (PDF, ZIP, etc.)
- **audio**: Audio files
- **video**: Video files
- **voice**: Voice messages
- **location**: GPS coordinates
- **contact**: Contact information
- **poll**: Polls and quizzes
- **dice**: Dice, darts, bowling animations
- **sticker**: Stickers
- **callback_query**: Inline button responses

**Reply Keyboards:**

**Custom Keyboard:**
```json
{
  "keyboard": [
    [{"text": "/agent list"}, {"text": "/agent create"}],
    [{"text": "/agent status"}]
  ],
  "resize_keyboard": true,
  "one_time_keyboard": true
}
```

**Inline Keyboard:**
```json
{
  "inline_keyboard": [
    [
      {"text": "Run Task", "callback_data": "run_task_123"},
      {"text": "Cancel", "callback_data": "cancel_task_123"}
    ]
  ]
}
```

### 2.4 Inline Mode

**Inline Queries:**
- Users type `@yourbotname query` in any chat
- Bot receives inline query webhook
- Responds with results (articles, photos, etc.)
- Users select result to share in chat

**Inline Query Structure:**
```json
{
  "update_id": 123456789,
  "inline_query": {
    "id": "query-id",
    "from": {
      "id": 987654321,
      "first_name": "John"
    },
    "query": "search term",
    "offset": ""
  }
}
```

**Use Cases for AOF:**
- Quick task searches across chats
- Agent command suggestions
- Context-aware command completion

### 2.5 Command Handling Patterns

**Bot Commands:**
- Commands start with `/` (e.g., `/start`, `/help`)
- Set commands via @BotFather or `setMyCommands` API
- Auto-completion in Telegram clients
- Scope commands per user, chat, or globally

**Command Registration:**
```json
{
  "commands": [
    {"command": "agent", "description": "Manage AI agents"},
    {"command": "task", "description": "Task management"},
    {"command": "run", "description": "Execute agent task"},
    {"command": "status", "description": "Check agent status"}
  ]
}
```

**Parsing Command Arguments:**
```
/agent run daily-report --format pdf
         ^^^^ ^^^^^^^^^^^^ ^^^^^^^^^^^
         cmd  arg1         arg2
```

### 2.6 Rate Limits and Best Practices

**Official Rate Limits:**
- **Individual chats**: 1 message per second
- **Groups/channels**: 20 messages per minute
- **Overall**: ~30 messages per second across all chats
- **Bulk notifications**: ~30 users per second

**Rate Limit Error Handling:**
```json
{
  "ok": false,
  "error_code": 429,
  "description": "Too Many Requests: retry after 5",
  "parameters": {
    "retry_after": 5
  }
}
```

**Best Practices:**
1. **Implement exponential backoff** for 429 errors
2. **Use request queuing** to stay within limits
3. **Spread bulk notifications** over 8-12 hours
4. **Handle 429 properly**: Wait specified seconds, then retry
5. **Use auto-retry plugins** for production bots
6. **Batch processing** when possible
7. **Maintain buffer** in rate limit calculations

**Important Notes:**
- Rate limits are flexible and adaptive
- Responding to user messages rarely hits limits
- Broadcasting to many users triggers rate limits
- Hitting limits doesn't cause bans (ignoring them does)
- Updates stored for 24 hours if not retrieved

### 2.7 Recommended Rust Crates

**Primary Recommendation: `teloxide`**
**Repository:** [https://github.com/teloxide/teloxide](https://github.com/teloxide/teloxide)

**Features:**
- High-level, ergonomic API design
- Covers almost entire Telegram Bot API
- Functional reactive design pattern
- Full dialogue management subsystem
- Strongly typed bot commands
- Active development and maintenance
- Excellent documentation

**Example Usage:**
```rust
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_message(msg.chat.id, "Hello from AOF!").await?;
        Ok(())
    })
    .await;
}
```

**Alternative: `telegram-bot`**
**Repository:** [https://crates.io/crates/telegram-bot](https://crates.io/crates/telegram-bot)

**Features:**
- Lower-level control
- E2E testing support with fake Telegram server
- Tracing framework integration
- Good for custom implementations

**Alternative: `telegram-bot-api`**
**Repository:** [https://crates.io/crates/telegram-bot-api](https://crates.io/crates/telegram-bot-api)

**Features:**
- Close mapping to official Telegram API
- Work in progress but functional
- Good for minimal dependencies

### 2.8 Required Credentials and Setup

**Minimal Setup:**
1. Message @BotFather on Telegram
2. Create bot with `/newbot`
3. Save bot token securely
4. Configure webhook or use polling

**Environment Variables:**
```bash
TELEGRAM_BOT_TOKEN=123456789:ABCdefGHIjklMNOpqrsTUVwxyz
TELEGRAM_WEBHOOK_URL=https://yourserver.com/telegram/webhook
TELEGRAM_WEBHOOK_SECRET=your-random-secret-token
```

**Optional Settings:**
- Bot profile photo
- Bot description and about text
- Command menu
- Inline mode settings
- Privacy mode (group message access)

### 2.9 Example Trigger Patterns

**Direct Commands:**
```
/agent run daily-report
/agent list --status active
/agent create "New automated task"
/task assign @johndoe "Review PR"
/status
```

**Inline Mode:**
```
@aofbot search tasks status:pending
@aofbot quick-run daily-report
@aofbot help commands
```

**Interactive Buttons:**
```
User: /agent list
Bot: "Your agents:
     [Agent-1] [Agent-2] [Agent-3]
     [Create New] [Help]"
```

---

## 3. Slack Events API

### 3.1 App Creation and OAuth Scopes

**App Creation Process:**
1. Visit [api.slack.com/apps](https://api.slack.com/apps)
2. Click "Create New App"
3. Choose "From scratch" or use manifest
4. Select workspace for development
5. Configure app settings and features

**OAuth Scopes System:**
- Granular permission model
- Bot scopes vs User scopes
- Scopes tied to event subscriptions
- Must reinstall app when scopes change

**Key OAuth Scopes:**
- `app_mentions:read` - See @mentions of app
- `channels:history` - Read message content in public channels
- `channels:read` - View basic channel info
- `chat:write` - Send messages as bot
- `commands` - Add slash commands
- `im:history` - Read direct messages
- `im:read` - View direct message info
- `im:write` - Start direct messages
- `users:read` - View users in workspace
- `incoming-webhook` - Create incoming webhooks

**Recent Scope Changes (2025):**
- May 29, 2025: New rate limits for non-Marketplace apps
- `conversations.history` limited to 1 req/min for commercial apps
- `conversations.replies` limited to 1 req/min for commercial apps
- Internal/custom apps not affected

### 3.2 Event Subscriptions

**Two Event Types:**

**Bot Events:**
- Subscribe to events on behalf of bot user
- No additional scopes required beyond `bot`
- Bot must be added to channels to receive events

**Common Bot Events:**
- `message.channels` - Message posted to public channel
- `message.im` - Direct message to bot
- `message.groups` - Message in private channel
- `app_mention` - Bot mentioned with @
- `member_joined_channel` - User joined channel
- `reaction_added` - Emoji reaction added

**Team Events:**
- Require corresponding OAuth scope
- Perspectival to member installing app
- More privileged access

**Event Subscription Setup:**
1. Navigate to "Event Subscriptions" in app config
2. Enable events
3. Provide Request URL (or use Socket Mode)
4. Subscribe to specific bot/team events
5. Reinstall app to apply changes

**Event JSON Structure:**
```json
{
  "token": "verification-token",
  "team_id": "T1234567890",
  "api_app_id": "A1234567890",
  "event": {
    "type": "message",
    "channel": "C1234567890",
    "user": "U1234567890",
    "text": "/agent run task",
    "ts": "1234567890.123456",
    "event_ts": "1234567890.123456",
    "channel_type": "channel"
  },
  "type": "event_callback",
  "event_id": "Ev1234567890",
  "event_time": 1234567890
}
```

### 3.3 Socket Mode vs HTTP Endpoints

**HTTP Endpoints (Traditional):**
- Slack sends POST to your public URL
- Requires public, HTTPS endpoint
- Must verify requests via signing secret
- Must respond within 3 seconds with HTTP 200
- URL verification challenge on setup

**Socket Mode (Modern):**
- Uses WebSocket URL instead of HTTP
- No public endpoint required
- Firewall-friendly
- Real-time bidirectional communication
- URL refreshes regularly via `apps.connections.open`

**Socket Mode Setup:**
1. Go to Settings > Basic Information
2. Add App-Level Token with `connections:write` scope
3. Go to Settings > Socket Mode
4. Toggle "Enable Socket Mode"
5. Use Bolt framework or official SDKs

**When to Use Socket Mode:**
- Behind corporate firewall
- Local development without tunneling
- Security concerns about public endpoints
- No desire to manage HTTP infrastructure
- Real-time interactions preferred

**When to Use HTTP:**
- Need to scale horizontally
- Serverless architecture (AWS Lambda, etc.)
- Already have robust HTTP infrastructure
- Want stateless request handling

### 3.4 Block Kit for Rich Responses

**Block Kit Overview:**
- UI framework for Slack messages
- JSON-based component system
- Interactive elements (buttons, selects, datepickers)
- Rich layouts (sections, dividers, headers)
- Context and formatting

**Example Block Kit Message:**
```json
{
  "blocks": [
    {
      "type": "header",
      "text": {
        "type": "plain_text",
        "text": "AOF Agent Status"
      }
    },
    {
      "type": "section",
      "text": {
        "type": "mrkdwn",
        "text": "*Agent-1*: Running\n*Agent-2*: Idle"
      }
    },
    {
      "type": "actions",
      "elements": [
        {
          "type": "button",
          "text": {
            "type": "plain_text",
            "text": "Run Task"
          },
          "value": "run_task",
          "action_id": "button_run_task"
        },
        {
          "type": "button",
          "text": {
            "type": "plain_text",
            "text": "View Logs"
          },
          "value": "view_logs",
          "action_id": "button_view_logs",
          "style": "primary"
        }
      ]
    }
  ]
}
```

**Block Types:**
- **section**: Text with optional accessory
- **header**: Large header text
- **divider**: Visual separator
- **actions**: Interactive elements
- **context**: Contextual info/metadata
- **image**: Image display
- **input**: Form inputs (modals)
- **file**: File attachments

**Interactive Elements:**
- Buttons
- Select menus (static, users, channels)
- Datepickers
- Timepickers
- Radio buttons
- Checkboxes
- Overflow menu (three-dot menu)

### 3.5 Slash Commands for Triggers

**Slash Command Setup:**
1. Navigate to "Slash Commands" in app config
2. Click "Create New Command"
3. Define command (e.g., `/agent`)
4. Provide Request URL
5. Add description and usage hint
6. Save and reinstall app

**Slash Command Request:**
```
POST /slack/commands
Content-Type: application/x-www-form-urlencoded

token=verification-token
team_id=T1234567890
team_domain=example
channel_id=C1234567890
channel_name=general
user_id=U1234567890
user_name=johndoe
command=/agent
text=run daily-report
response_url=https://hooks.slack.com/commands/...
trigger_id=1234567890.123456
```

**Response Types:**

**Immediate Response:**
```json
{
  "response_type": "in_channel",
  "text": "Running daily report...",
  "blocks": [...]
}
```

**Delayed Response (use `response_url`):**
```bash
curl -X POST $response_url \
  -H "Content-Type: application/json" \
  -d '{"text": "Daily report completed!"}'
```

**Response Types:**
- `ephemeral`: Only visible to user who ran command (default)
- `in_channel`: Visible to entire channel

### 3.6 Rate Limits and Best Practices

**2025 Rate Limit Changes:**

**Non-Marketplace Apps (after May 29, 2025):**
- `conversations.history`: 1 request per minute
- `conversations.replies`: 1 request per minute
- `limit` parameter reduced to max 15 objects
- Applies to commercially distributed apps only

**Custom/Internal Apps:**
- `conversations.history`: 50+ requests per minute
- `conversations.replies`: 50+ requests per minute
- Not affected by Marketplace restrictions

**General Rate Limits:**
- **Message posting**: 1 message per second per channel
- **Web API methods**: Per-method, per-workspace limits
- **Rate limit windows**: Per minute
- **HTTP 429 response**: Includes `Retry-After` header

**Best Practices:**
1. **Design for 1 req/sec** per API method
2. **Allow temporary bursts** but return to limit
3. **Handle 429 gracefully**: Use `Retry-After` header
4. **Use response_url** for delayed responses (5 uses per command)
5. **Batch operations** when possible
6. **Cache frequently accessed data**
7. **Monitor rate limit headers**: `X-Rate-Limit-*`
8. **Consider Marketplace approval** for higher limits

**Rate Limit Headers:**
```
X-Rate-Limit-Limit: 50
X-Rate-Limit-Remaining: 45
X-Rate-Limit-Reset: 1234567890
```

**RTM API Limits:**
- 16KB max message size (including JSON)
- 4000 character limit for channel messages
- Client disconnected if exceeded

### 3.7 Recommended Rust Crate

**Crate:** `slack-rust-rs`
**Repository:** [https://crates.io/crates/slack-rust-rs](https://crates.io/crates/slack-rust-rs)

**Features:**
- Socket Mode support
- Events API support
- Web API support
- Work in progress but functional

**Status:** Under active development
**Maturity:** Early stage, limited production use

**Alternative Approach:**
- Use `reqwest` for HTTP API calls
- Use `tokio-tungstenite` for Socket Mode WebSocket
- Use `serde` for JSON serialization
- Build custom wrapper around Slack Web API

**Community SDK Status:**
- Rust Slack ecosystem less mature than Python/JavaScript
- Consider contributing to `slack-rust-rs`
- Official Slack SDKs: Bolt (JS), Bolt (Python), Bolt (Java)

### 3.8 Required Credentials and Setup

**Production Setup:**
1. Create Slack App at [api.slack.com/apps](https://api.slack.com/apps)
2. Configure OAuth scopes
3. Install app to workspace
4. Generate Bot User OAuth Token
5. (Optional) Create App-Level Token for Socket Mode
6. (Optional) Configure Signing Secret for HTTP verification

**Environment Variables:**
```bash
SLACK_BOT_TOKEN=YOUR_SLACK_BOT_TOKEN_HERE
SLACK_APP_TOKEN=YOUR_SLACK_APP_TOKEN_HERE  # Socket Mode only
SLACK_SIGNING_SECRET=YOUR_SLACK_SIGNING_SECRET_HERE  # HTTP verification
SLACK_CLIENT_ID=YOUR_SLACK_CLIENT_ID_HERE  # OAuth flow
SLACK_CLIENT_SECRET=YOUR_SLACK_CLIENT_SECRET_HERE  # OAuth flow
```

**Security Considerations:**
- Verify requests using signing secret (HTTP mode)
- Validate token in Socket Mode connections
- Use HTTPS for all webhook endpoints
- Rotate tokens periodically
- Store tokens in secure environment

### 3.9 Example Trigger Patterns

**Slash Commands:**
```
/agent run daily-report
/agent list --status active
/agent create --name "Automated Task"
/aof-task assign @johndoe "Review documentation"
/aof-status
```

**App Mentions:**
```
@AOFBot run daily report
@AOFBot show my tasks
@AOFBot help
```

**Direct Messages:**
```
User DMs bot: "run daily report"
Bot: "Running daily report for @user..."
```

**Interactive Buttons:**
```
User: /agent list
Bot: [Message with Block Kit]
     "Your agents:
     [Run Agent-1] [Stop Agent-1]
     [Run Agent-2] [Stop Agent-2]
     [Create New Agent]"
```

---

## 4. Discord Gateway API

### 4.1 Bot Token and Privileged Intents

**Bot Creation Process:**
1. Visit [Discord Developer Portal](https://discord.com/developers/applications)
2. Click "New Application"
3. Navigate to "Bot" section
4. Click "Add Bot"
5. Copy bot token (shown once)
6. Enable privileged intents as needed

**Bot Token Format:**
```
YOUR_DISCORD_BOT_TOKEN_HERE
```

**Token Security:**
- Treat as password - never commit to version control
- Regenerate if compromised
- Use environment variables or secret management
- Token grants full access to bot actions

### 4.2 Privileged Gateway Intents

**Three Privileged Intents:**

**1. Guild Members Intent (`GUILD_MEMBERS`)**
- Access to member join/leave events
- Member update events (nickname, roles, etc.)
- Member chunk requests
- **Required for 100+ servers**: Verification needed

**2. Guild Presences Intent (`GUILD_PRESENCES`)**
- User online/offline status
- Activity updates (playing, streaming)
- Rich presence data
- **Required for 100+ servers**: Verification needed

**3. Message Content Intent (`MESSAGE_CONTENT`)**
- Read message `content`, `embeds`, `attachments`, `components`
- **Does NOT affect**: Sending messages (always allowed)
- **Does NOT require for**: DMs, mentions, bot's own messages
- **Required for**: Prefix-based commands, content analysis
- **Verification required for 100+ servers**

**Enabling Privileged Intents:**
1. Go to Application > Bot section in Developer Portal
2. Scroll to "Privileged Gateway Intents"
3. Toggle required intents
4. For 100+ guilds: Submit verification form with justification

**Verification Requirements (100+ Guilds):**
- Document why each intent is needed
- Explain data usage and storage
- Describe data minimization practices
- Provide privacy policy
- May take several days for approval

### 4.3 Gateway Events vs HTTP API

**Gateway API (WebSocket):**
- Persistent WebSocket connection to Discord
- Real-time bidirectional communication
- Receives events as they occur
- Requires heartbeat to maintain connection
- Automatically reconnects on disconnect

**Common Gateway Events:**
- `MESSAGE_CREATE` - New message posted
- `MESSAGE_UPDATE` - Message edited
- `MESSAGE_DELETE` - Message deleted
- `GUILD_CREATE` - Bot added to server
- `GUILD_MEMBER_ADD` - User joined server
- `INTERACTION_CREATE` - Slash command/button used
- `READY` - Connection established

**Event Structure:**
```json
{
  "op": 0,
  "d": {
    "id": "1234567890",
    "channel_id": "1234567890",
    "author": {
      "id": "1234567890",
      "username": "johndoe",
      "discriminator": "1234",
      "avatar": "abc123"
    },
    "content": "/agent run task",
    "timestamp": "2025-12-09T10:00:00.000000+00:00",
    "type": 0
  },
  "s": 42,
  "t": "MESSAGE_CREATE"
}
```

**HTTP API (REST):**
- One-off requests for specific actions
- Send messages, manage channels, modify roles
- No event reception (use Gateway for events)
- Rate-limited per endpoint

**When to Use Each:**
- **Gateway**: Receiving events, real-time interactions, bot lifecycle
- **HTTP**: Sending messages, modifying resources, one-off operations
- **Both**: Most bots use Gateway for events + HTTP for responses

### 4.4 Message Content Intent Requirements

**What Message Content Intent Provides:**
- `message.content` field (the actual text)
- `message.embeds` array
- `message.attachments` array
- `message.components` array (buttons, selects)

**What Works WITHOUT Message Content Intent:**
- Reading messages in DMs (always allowed)
- Reading messages where bot is mentioned
- Reading bot's own messages
- Slash commands (uses interactions, not messages)
- Button/select interactions
- Modal submissions

**Migration to Slash Commands:**
- Discord strongly encourages slash commands over prefix commands
- Slash commands provide typed parameters
- Built-in validation and autocomplete
- Better UX with command discovery
- No privileged intent required

**Prefix Commands vs Slash Commands:**
```
# Prefix Command (requires MESSAGE_CONTENT intent)
User types: !agent run daily-report
Bot reads message.content: "!agent run daily-report"
Bot parses and responds

# Slash Command (NO intent required)
User types: /agent run task:daily-report
Discord sends INTERACTION_CREATE with structured data
Bot receives parsed parameters directly
```

### 4.5 Slash Commands (Application Commands)

**Command Types:**
- **Slash Commands**: `/command` in chat input
- **User Commands**: Right-click user → Apps → Command
- **Message Commands**: Right-click message → Apps → Command

**Creating Slash Commands:**

**Global Command (all servers):**
```http
POST /applications/{application.id}/commands
{
  "name": "agent",
  "description": "Manage AI agents",
  "options": [
    {
      "name": "run",
      "description": "Run an agent task",
      "type": 1,
      "options": [
        {
          "name": "task",
          "description": "Task name",
          "type": 3,
          "required": true
        }
      ]
    }
  ]
}
```

**Guild Command (single server):**
```http
POST /applications/{application.id}/guilds/{guild.id}/commands
{...}
```

**Command Option Types:**
- `1` - SUB_COMMAND
- `2` - SUB_COMMAND_GROUP
- `3` - STRING
- `4` - INTEGER
- `5` - BOOLEAN
- `6` - USER
- `7` - CHANNEL
- `8` - ROLE
- `9` - MENTIONABLE
- `10` - NUMBER
- `11` - ATTACHMENT

**Command Response:**
```json
{
  "type": 2,
  "data": {
    "id": "1234567890",
    "name": "agent",
    "options": [
      {
        "name": "run",
        "options": [
          {
            "name": "task",
            "value": "daily-report"
          }
        ]
      }
    ]
  }
}
```

### 4.6 Interaction Handling

**Interaction Types:**
- `APPLICATION_COMMAND` (type 2) - Slash command used
- `MESSAGE_COMPONENT` (type 3) - Button/select clicked
- `APPLICATION_COMMAND_AUTOCOMPLETE` (type 4) - Autocomplete triggered
- `MODAL_SUBMIT` (type 5) - Modal form submitted

**Interaction Response Requirements:**
- **MUST respond within 3 seconds** or interaction fails
- Use interaction token for responses
- Initial response types:
  - `PONG` (type 1) - Acknowledge ping
  - `CHANNEL_MESSAGE_WITH_SOURCE` (type 4) - Send message
  - `DEFERRED_CHANNEL_MESSAGE_WITH_SOURCE` (type 5) - Thinking...
  - `DEFERRED_UPDATE_MESSAGE` (type 6) - Thinking (component)
  - `UPDATE_MESSAGE` (type 7) - Update message (component)
  - `APPLICATION_COMMAND_AUTOCOMPLETE_RESULT` (type 8) - Autocomplete
  - `MODAL` (type 9) - Show modal

**Interaction Response:**
```http
POST /interactions/{interaction.id}/{interaction.token}/callback
{
  "type": 4,
  "data": {
    "content": "Running daily report...",
    "components": [
      {
        "type": 1,
        "components": [
          {
            "type": 2,
            "label": "Cancel",
            "style": 4,
            "custom_id": "cancel_task"
          }
        ]
      }
    ]
  }
}
```

**Follow-up Messages (after initial response):**
```http
POST /webhooks/{application.id}/{interaction.token}
{
  "content": "Daily report completed!"
}
```

### 4.7 Rate Limits and Best Practices

**Discord Rate Limits:**

**Global Rate Limit:**
- 50 requests per second across all endpoints
- Applies per bot token
- Exceeding triggers 429 with global flag

**Per-Route Rate Limits:**
- Vary by endpoint
- Typically 5-10 requests per second
- Share buckets between similar endpoints

**Webhook Rate Limits:**
- 5 requests per 2 seconds per webhook
- Failed requests count toward limit
- Separate rate limits per webhook URL

**Rate Limit Headers:**
```
X-RateLimit-Limit: 5
X-RateLimit-Remaining: 4
X-RateLimit-Reset: 1234567890.123
X-RateLimit-Reset-After: 2.5
X-RateLimit-Bucket: abc123def456
X-RateLimit-Global: false
```

**429 Response:**
```json
{
  "message": "You are being rate limited.",
  "retry_after": 2.5,
  "global": false
}
```

**Best Practices for 2025:**

1. **Use Slash Commands Instead of Prefix Commands**
   - No MESSAGE_CONTENT intent needed
   - Better UX with autocomplete
   - Built-in parameter validation
   - Command discovery in client

2. **Implement Proper Rate Limit Handling**
   - Track rate limit headers
   - Queue requests when near limit
   - Handle 429 with exponential backoff
   - Use `retry_after` from response

3. **Minimize MESSAGE_CONTENT Usage**
   - Use interactions for structured input
   - Use buttons/selects for choices
   - Use modals for complex forms
   - Only read message content when necessary

4. **Plan for Privileged Intent Verification**
   - Document data usage clearly
   - Implement data minimization
   - Prepare privacy policy
   - Submit verification early

5. **Use Components for Interaction**
   - Buttons for actions
   - Select menus for choices
   - Modals for forms
   - Context menus for shortcuts

6. **Gateway Connection Management**
   - Implement reconnection logic
   - Send heartbeats reliably
   - Handle RESUME properly
   - Monitor connection health

7. **Webhook Strategy**
   - Use webhooks for outbound notifications
   - Separate webhooks for different channels
   - Monitor webhook rate limits
   - Handle 429 per webhook

### 4.8 Recommended Rust Crate

**Primary Recommendation: `serenity`**
**Repository:** [https://github.com/serenity-rs/serenity](https://github.com/serenity-rs/serenity)

**Features:**
- Comprehensive Discord API coverage
- Gateway and HTTP support
- Slash commands support
- Framework for command handling
- Active maintenance and community
- Excellent documentation

**Example Usage:**
```rust
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::interactions::Interaction;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "agent" => "Running agent task...".to_string(),
                _ => "Unknown command".to_string(),
            };

            command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(content))
            })
            .await
            .ok();
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected token");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    client.start().await.ok();
}
```

**Alternative: `twilight`**
**Repository:** [https://github.com/twilight-rs/twilight](https://github.com/twilight-rs/twilight)

**Features:**
- Modular architecture (use only what you need)
- Lower-level control
- High performance
- Good for custom implementations
- Steeper learning curve

**Alternative: Custom Implementation**
- Use `tokio-tungstenite` for Gateway WebSocket
- Use `reqwest` for HTTP API
- Use `serde` for JSON
- More work but maximum control

### 4.9 Required Credentials and Setup

**Production Setup:**
1. Create Application in Discord Developer Portal
2. Add Bot to application
3. Copy bot token
4. Enable required privileged intents
5. Generate OAuth2 URL with scopes and permissions
6. Invite bot to servers

**Environment Variables:**
```bash
DISCORD_BOT_TOKEN=YOUR_DISCORD_BOT_TOKEN_HERE
DISCORD_APPLICATION_ID=YOUR_DISCORD_APPLICATION_ID_HERE
DISCORD_PUBLIC_KEY=YOUR_DISCORD_PUBLIC_KEY_HERE
```

**OAuth2 URL Generator:**
```
https://discord.com/api/oauth2/authorize?client_id={APPLICATION_ID}&permissions={PERMISSIONS}&scope=bot%20applications.commands
```

**Required Permissions (bits):**
- `VIEW_CHANNEL` (1024)
- `SEND_MESSAGES` (2048)
- `EMBED_LINKS` (16384)
- `READ_MESSAGE_HISTORY` (65536)
- `USE_SLASH_COMMANDS` (2147483648)

**Gateway Intents (for serenity):**
```rust
let intents = GatewayIntents::GUILDS
    | GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT;  // Only if needed
```

### 4.10 Example Trigger Patterns

**Slash Commands (Recommended):**
```
/agent run task:daily-report
/agent list status:active
/agent create name:"Automated Task"
/task assign user:@johndoe task:"Review documentation"
/aof-status
```

**Message Commands (Requires MESSAGE_CONTENT intent):**
```
User types: /agent run daily-report
Bot reads content and parses
```

**Button Interactions:**
```
User: /agent list
Bot: [Message with components]
     "Your agents:
     Agent-1: [Run] [Stop] [Logs]
     Agent-2: [Run] [Stop] [Logs]
     [Create New Agent]"
User clicks [Run] button
Bot receives interaction
```

**Context Menu Commands:**
```
User right-clicks message
User selects: Apps > Process with AOF
Bot receives MESSAGE_COMMAND interaction
```

---

## 5. Comparative Analysis

### 5.1 Authentication Complexity

| Platform | Complexity | Method | Setup Time |
|----------|-----------|--------|------------|
| Telegram | ⭐ Simplest | Bot token from BotFather | 2 minutes |
| Discord | ⭐⭐ Simple | Bot token from Developer Portal | 5 minutes |
| Slack | ⭐⭐⭐ Moderate | OAuth with scopes, bot token | 15 minutes |
| WhatsApp | ⭐⭐⭐⭐ Complex | Meta Business Account, System User | 30-60 minutes |

### 5.2 Integration Approach

| Platform | Best For | Webhook Support | Polling Support | Real-time |
|----------|----------|-----------------|-----------------|-----------|
| Telegram | Rapid prototyping | ✅ Yes | ✅ Yes (getUpdates) | ✅ |
| Discord | Gaming/Community | ✅ Limited (outbound only) | ❌ No | ✅ Gateway |
| Slack | Enterprise/Workplace | ✅ Yes | ❌ No | ✅ Socket Mode |
| WhatsApp | Customer service | ✅ Yes (required) | ❌ No | ✅ |

### 5.3 Message Richness

| Platform | Text | Images | Buttons | Forms | Commands |
|----------|------|--------|---------|-------|----------|
| Telegram | ✅ | ✅ | ✅ Inline keyboards | ❌ | ✅ Bot commands |
| Discord | ✅ | ✅ | ✅ Components | ✅ Modals | ✅ Slash commands |
| Slack | ✅ | ✅ | ✅ Block Kit | ✅ Modals | ✅ Slash commands |
| WhatsApp | ✅ | ✅ | ✅ Interactive | ❌ | ❌ |

### 5.4 Rate Limits Summary

| Platform | Primary Limit | Burst Allowed | Enforcement |
|----------|---------------|---------------|-------------|
| Telegram | 1 msg/sec (individual), 20/min (groups) | ✅ Short bursts | Soft (429 errors) |
| Discord | 5-10 req/sec per route | ❌ Strict | Hard (429 + bucket) |
| Slack | 1 msg/sec per channel | ✅ Temporary | Soft (429 + retry-after) |
| WhatsApp | Varies by tier | ❌ Strict | Hard (paid tiers) |

### 5.5 Rust Ecosystem Maturity

| Platform | Crate | Maturity | Community | Documentation |
|----------|-------|----------|-----------|---------------|
| Telegram | `teloxide` | ⭐⭐⭐⭐⭐ Excellent | Active | Comprehensive |
| Discord | `serenity` | ⭐⭐⭐⭐⭐ Excellent | Very Active | Excellent |
| Slack | `slack-rust-rs` | ⭐⭐ Early | Small | Limited |
| WhatsApp | `whatsapp-cloud-api` | ⭐ Nascent | Minimal | Minimal |

### 5.6 Use Case Recommendations

**Telegram:**
- ✅ Rapid MVP development
- ✅ Developer-focused bots
- ✅ Personal automation
- ✅ Easy polling for development
- ❌ Enterprise compliance needs

**Discord:**
- ✅ Community engagement
- ✅ Gaming integrations
- ✅ Real-time collaboration
- ✅ Rich interactions (buttons, modals)
- ❌ Business/customer service

**Slack:**
- ✅ Workplace automation
- ✅ Team productivity tools
- ✅ Enterprise integrations
- ✅ Behind firewall (Socket Mode)
- ❌ Consumer-facing applications

**WhatsApp:**
- ✅ Customer service automation
- ✅ Transactional messaging
- ✅ Large user base reach
- ✅ International markets
- ❌ Complex chatbot workflows (limited interactions)

---

## 6. Rust Integration Architecture for AOF

### 6.1 Recommended Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    AOF Core (Rust)                           │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Agent Orchestration Layer                  │ │
│  │        (Task Management, State, Routing)               │ │
│  └────────────────────────────────────────────────────────┘ │
│                              │                               │
│  ┌───────────────────────────┴────────────────────────────┐ │
│  │           Platform Adapter Interface (Trait)           │ │
│  └───────────┬──────────┬──────────┬──────────┬──────────┘ │
│              │          │          │          │              │
│  ┌───────────▼──┐  ┌───▼──────┐  ┌▼────────┐  ┌▼────────┐ │
│  │  Telegram    │  │ Discord  │  │  Slack  │  │WhatsApp │ │
│  │   Adapter    │  │ Adapter  │  │ Adapter │  │ Adapter │ │
│  │              │  │          │  │         │  │         │ │
│  │  (teloxide)  │  │(serenity)│  │ (HTTP)  │  │ (HTTP)  │ │
│  └──────────────┘  └──────────┘  └─────────┘  └─────────┘ │
└─────────────────────────────────────────────────────────────┘
                          │
          ┌───────────────┼───────────────┐
          │               │               │
    ┌─────▼─────┐   ┌────▼────┐   ┌─────▼──────┐
    │ Telegram  │   │ Discord │   │   Slack    │
    │    API    │   │Gateway  │   │  Events    │
    └───────────┘   └─────────┘   └────────────┘
```

### 6.2 Platform Adapter Trait

```rust
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    /// Initialize connection and start receiving events
    async fn connect(&mut self) -> Result<(), AdapterError>;

    /// Send a text message to a chat/channel
    async fn send_message(
        &self,
        chat_id: &str,
        message: &str,
    ) -> Result<MessageId, AdapterError>;

    /// Send a rich message with buttons, embeds, etc.
    async fn send_rich_message(
        &self,
        chat_id: &str,
        content: RichMessage,
    ) -> Result<MessageId, AdapterError>;

    /// Handle incoming message (called by adapter implementation)
    async fn handle_message(&self, message: IncomingMessage) -> Result<(), AdapterError>;

    /// Handle interaction (button click, slash command, etc.)
    async fn handle_interaction(&self, interaction: Interaction) -> Result<(), AdapterError>;

    /// Get platform name
    fn platform_name(&self) -> &str;

    /// Health check
    async fn is_connected(&self) -> bool;
}

pub struct IncomingMessage {
    pub id: String,
    pub chat_id: String,
    pub user_id: String,
    pub username: Option<String>,
    pub content: String,
    pub timestamp: u64,
    pub platform: String,
    pub raw: Value, // Platform-specific raw data
}

pub struct RichMessage {
    pub text: String,
    pub buttons: Vec<Button>,
    pub embeds: Vec<Embed>,
    pub components: Vec<Component>,
}

pub struct Interaction {
    pub id: String,
    pub interaction_type: InteractionType,
    pub user_id: String,
    pub data: Value,
    pub platform: String,
}

pub enum InteractionType {
    ButtonClick,
    SlashCommand,
    SelectMenu,
    ModalSubmit,
}
```

### 6.3 Example: Telegram Adapter Implementation

```rust
use teloxide::prelude::*;
use async_trait::async_trait;

pub struct TelegramAdapter {
    bot: Bot,
    message_handler: Arc<dyn MessageHandler>,
}

impl TelegramAdapter {
    pub fn new(token: &str, handler: Arc<dyn MessageHandler>) -> Self {
        Self {
            bot: Bot::new(token),
            message_handler: handler,
        }
    }
}

#[async_trait]
impl PlatformAdapter for TelegramAdapter {
    async fn connect(&mut self) -> Result<(), AdapterError> {
        let bot = self.bot.clone();
        let handler = self.message_handler.clone();

        tokio::spawn(async move {
            teloxide::repl(bot, move |bot: Bot, msg: Message| {
                let handler = handler.clone();
                async move {
                    let incoming = IncomingMessage {
                        id: msg.id.to_string(),
                        chat_id: msg.chat.id.to_string(),
                        user_id: msg.from()
                            .map(|u| u.id.to_string())
                            .unwrap_or_default(),
                        username: msg.from()
                            .and_then(|u| u.username.clone()),
                        content: msg.text().unwrap_or("").to_string(),
                        timestamp: msg.date as u64,
                        platform: "telegram".to_string(),
                        raw: serde_json::to_value(&msg).unwrap(),
                    };

                    handler.handle_message(incoming).await.ok();
                    Ok(())
                }
            })
            .await;
        });

        Ok(())
    }

    async fn send_message(
        &self,
        chat_id: &str,
        message: &str,
    ) -> Result<MessageId, AdapterError> {
        let chat_id = chat_id.parse::<i64>()
            .map_err(|_| AdapterError::InvalidChatId)?;

        let msg = self.bot
            .send_message(ChatId(chat_id), message)
            .await
            .map_err(|e| AdapterError::SendFailed(e.to_string()))?;

        Ok(MessageId {
            platform: "telegram".to_string(),
            id: msg.id.to_string(),
        })
    }

    async fn send_rich_message(
        &self,
        chat_id: &str,
        content: RichMessage,
    ) -> Result<MessageId, AdapterError> {
        let chat_id = chat_id.parse::<i64>()
            .map_err(|_| AdapterError::InvalidChatId)?;

        // Convert RichMessage to Telegram InlineKeyboard
        let keyboard = content.buttons
            .chunks(2)
            .map(|row| {
                row.iter()
                    .map(|btn| InlineKeyboardButton::callback(
                        &btn.text,
                        &btn.id,
                    ))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let msg = self.bot
            .send_message(ChatId(chat_id), &content.text)
            .reply_markup(InlineKeyboardMarkup::new(keyboard))
            .await
            .map_err(|e| AdapterError::SendFailed(e.to_string()))?;

        Ok(MessageId {
            platform: "telegram".to_string(),
            id: msg.id.to_string(),
        })
    }

    fn platform_name(&self) -> &str {
        "telegram"
    }

    async fn is_connected(&self) -> bool {
        self.bot.get_me().await.is_ok()
    }
}
```

### 6.4 Example: Discord Adapter Implementation

```rust
use serenity::async_trait;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub struct DiscordAdapter {
    client: Client,
    message_handler: Arc<dyn MessageHandler>,
}

struct Handler {
    message_handler: Arc<dyn MessageHandler>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let incoming = IncomingMessage {
                id: command.id.to_string(),
                chat_id: command.channel_id.to_string(),
                user_id: command.user.id.to_string(),
                username: Some(command.user.name.clone()),
                content: format!(
                    "/{} {}",
                    command.data.name,
                    command.data.options
                        .iter()
                        .map(|o| format!("{}:{}", o.name, o.value.as_ref().unwrap()))
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                timestamp: command.id.created_at().unix_timestamp() as u64,
                platform: "discord".to_string(),
                raw: serde_json::to_value(&command).unwrap(),
            };

            self.message_handler.handle_message(incoming).await.ok();
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let incoming = IncomingMessage {
            id: msg.id.to_string(),
            chat_id: msg.channel_id.to_string(),
            user_id: msg.author.id.to_string(),
            username: Some(msg.author.name.clone()),
            content: msg.content.clone(),
            timestamp: msg.timestamp.unix_timestamp() as u64,
            platform: "discord".to_string(),
            raw: serde_json::to_value(&msg).unwrap(),
        };

        self.message_handler.handle_message(incoming).await.ok();
    }
}

#[async_trait]
impl PlatformAdapter for DiscordAdapter {
    async fn connect(&mut self) -> Result<(), AdapterError> {
        self.client
            .start()
            .await
            .map_err(|e| AdapterError::ConnectionFailed(e.to_string()))
    }

    async fn send_message(
        &self,
        chat_id: &str,
        message: &str,
    ) -> Result<MessageId, AdapterError> {
        let channel_id = chat_id.parse::<u64>()
            .map_err(|_| AdapterError::InvalidChatId)?;

        let http = &self.client.cache_and_http.http;
        let channel = ChannelId(channel_id);

        let msg = channel
            .say(http, message)
            .await
            .map_err(|e| AdapterError::SendFailed(e.to_string()))?;

        Ok(MessageId {
            platform: "discord".to_string(),
            id: msg.id.to_string(),
        })
    }

    fn platform_name(&self) -> &str {
        "discord"
    }

    async fn is_connected(&self) -> bool {
        self.client.cache_and_http.http
            .get_current_user()
            .await
            .is_ok()
    }
}
```

### 6.5 Configuration Management

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MessagingConfig {
    pub platforms: Vec<PlatformConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum PlatformConfig {
    Telegram {
        enabled: bool,
        token: String,
        webhook_url: Option<String>,
        use_polling: bool,
    },
    Discord {
        enabled: bool,
        token: String,
        application_id: String,
        intents: Vec<String>,
    },
    Slack {
        enabled: bool,
        bot_token: String,
        app_token: Option<String>, // For Socket Mode
        signing_secret: String,
        use_socket_mode: bool,
    },
    WhatsApp {
        enabled: bool,
        access_token: String,
        phone_number_id: String,
        webhook_verify_token: String,
        business_account_id: String,
    },
}

impl MessagingConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}
```

### 6.6 Example Configuration File

```toml
# config/messaging.toml

[[platforms]]
type = "Telegram"
enabled = true
token = "${TELEGRAM_BOT_TOKEN}"
webhook_url = "https://api.yourdomain.com/telegram/webhook"
use_polling = false

[[platforms]]
type = "Discord"
enabled = true
token = "${DISCORD_BOT_TOKEN}"
application_id = "${DISCORD_APPLICATION_ID}"
intents = ["GUILDS", "GUILD_MESSAGES", "MESSAGE_CONTENT"]

[[platforms]]
type = "Slack"
enabled = true
bot_token = "${SLACK_BOT_TOKEN}"
app_token = "${SLACK_APP_TOKEN}"
signing_secret = "${SLACK_SIGNING_SECRET}"
use_socket_mode = true

[[platforms]]
type = "WhatsApp"
enabled = false
access_token = "${WHATSAPP_ACCESS_TOKEN}"
phone_number_id = "${WHATSAPP_PHONE_NUMBER_ID}"
webhook_verify_token = "${WHATSAPP_WEBHOOK_VERIFY_TOKEN}"
business_account_id = "${WHATSAPP_BUSINESS_ACCOUNT_ID}"
```

---

## 7. Integration Recommendations for AOF

### 7.1 Phase 1: Foundation (Weeks 1-2)

**Goals:**
- Implement platform adapter trait
- Create Telegram adapter (simplest to start)
- Build message routing infrastructure
- Set up webhook server (if needed)

**Deliverables:**
- `PlatformAdapter` trait definition
- `TelegramAdapter` implementation using `teloxide`
- Basic message handler interface
- Configuration system for credentials

### 7.2 Phase 2: Core Platforms (Weeks 3-5)

**Goals:**
- Add Discord adapter
- Add Slack adapter (Socket Mode)
- Implement rich message support
- Add interaction handling (buttons, slash commands)

**Deliverables:**
- `DiscordAdapter` implementation using `serenity`
- `SlackAdapter` implementation (custom HTTP client)
- Rich message abstraction layer
- Button/interaction callback routing

### 7.3 Phase 3: WhatsApp (Weeks 6-7)

**Goals:**
- Add WhatsApp adapter
- Implement webhook verification
- Handle message templates
- Manage conversation pricing model

**Deliverables:**
- `WhatsAppAdapter` implementation (custom HTTP client)
- Webhook verification middleware
- Template message support
- Cost tracking for conversations

### 7.4 Phase 4: Advanced Features (Weeks 8-10)

**Goals:**
- Multi-platform command routing
- Unified trigger pattern system
- Rate limiting and queue management
- Monitoring and metrics

**Deliverables:**
- Command parser supporting multiple platforms
- Rate limiter per platform
- Message queue with retry logic
- Prometheus metrics exporter

### 7.5 Immediate Next Steps

1. **Create Platform Adapter Trait**
   - Define common interface for all platforms
   - Include async methods for sending/receiving
   - Support both text and rich messages

2. **Implement Telegram Adapter First**
   - Use `teloxide` crate (mature, well-documented)
   - Support both polling (dev) and webhooks (prod)
   - Test with simple echo bot

3. **Build Webhook Server (if needed)**
   - Use `axum` or `actix-web` for HTTP server
   - Implement signature verification for each platform
   - Route webhooks to appropriate adapter

4. **Configuration System**
   - Support environment variables and config files
   - Secure credential management
   - Per-platform enable/disable flags

5. **Testing Strategy**
   - Mock platform APIs for unit tests
   - Integration tests with test bots
   - Load testing for rate limits

---

## 8. Key Considerations and Challenges

### 8.1 Authentication Complexity

**Challenge:** Each platform has unique auth requirements
**Solution:**
- Centralized credential management
- Secure environment variable handling
- Rotation strategy for long-lived tokens
- Clear documentation for setup

### 8.2 Rate Limiting

**Challenge:** Different rate limits across platforms
**Solution:**
- Per-platform rate limiter implementation
- Message queue with priority
- Exponential backoff on 429 errors
- Monitor rate limit headers

### 8.3 Message Format Differences

**Challenge:** Platforms have different message structures
**Solution:**
- Abstract `RichMessage` format internally
- Platform-specific renderers
- Graceful degradation (buttons → text on unsupported platforms)
- Test matrix for cross-platform consistency

### 8.4 Webhook vs Polling

**Challenge:** Different connection models
**Solution:**
- Support both in adapter trait
- Configurable per platform
- Polling for development/testing
- Webhooks for production

### 8.5 Privileged Access

**Challenge:** Discord MESSAGE_CONTENT, WhatsApp Business verification
**Solution:**
- Clear documentation on requirements
- Slash commands for Discord (no intent needed)
- Verification process guide for WhatsApp
- Fallback strategies

### 8.6 Rust Ecosystem Maturity

**Challenge:** Slack and WhatsApp have limited Rust support
**Solution:**
- Use `reqwest` + `serde` for HTTP APIs
- Contribute to existing crates (`slack-rust-rs`)
- Consider wrapping official SDKs via FFI (if needed)
- Build thin custom clients

### 8.7 Testing Strategy

**Challenge:** Testing against live APIs is difficult
**Solution:**
- Mock HTTP responses in unit tests
- Use test bots for integration tests
- Record/replay HTTP interactions
- Separate test environments

### 8.8 Error Handling

**Challenge:** Network failures, rate limits, invalid tokens
**Solution:**
- Comprehensive error types per adapter
- Retry logic with exponential backoff
- Dead letter queue for failed messages
- Monitoring and alerting

---

## 9. Appendix: Quick Reference

### 9.1 Crates Summary

| Platform | Crate | Version | Maturity | Recommendation |
|----------|-------|---------|----------|----------------|
| Telegram | `teloxide` | 0.12+ | ⭐⭐⭐⭐⭐ | Highly Recommended |
| Discord | `serenity` | 0.12+ | ⭐⭐⭐⭐⭐ | Highly Recommended |
| Slack | `slack-rust-rs` | 0.x | ⭐⭐ | Use with caution, or build custom |
| WhatsApp | `whatsapp-cloud-api` | 0.x | ⭐ | Build custom with `reqwest` |

### 9.2 Credential Checklist

**Telegram:**
- [ ] Bot token from @BotFather

**Discord:**
- [ ] Bot token from Developer Portal
- [ ] Application ID
- [ ] Privileged intents enabled (if needed)

**Slack:**
- [ ] Bot User OAuth Token
- [ ] App-Level Token (Socket Mode)
- [ ] Signing Secret
- [ ] OAuth scopes configured

**WhatsApp:**
- [ ] Meta Business Account created
- [ ] Meta App created
- [ ] Access token (permanent)
- [ ] Phone number ID
- [ ] Webhook verify token
- [ ] Business Account ID

### 9.3 Rate Limit Quick Reference

| Platform | Limit | Window | Burst |
|----------|-------|--------|-------|
| Telegram Individual | 1 msg/sec | Rolling | Yes |
| Telegram Group | 20 msg | 1 min | No |
| Discord Per-Route | 5-10 req/sec | Bucket | No |
| Discord Global | 50 req/sec | Global | No |
| Slack Per-Channel | 1 msg/sec | Rolling | Yes |
| WhatsApp | Tier-based | Varies | No |

### 9.4 Webhook Port Requirements

| Platform | Supported Ports | TLS Required |
|----------|----------------|--------------|
| Telegram | 443, 80, 88, 8443 | TLS 1.2+ |
| Discord | Any | Yes (webhooks) |
| Slack | Any | Yes |
| WhatsApp | 443 | Yes |

### 9.5 Example Trigger Patterns Summary

**Universal Pattern for AOF Integration:**
```
/agent [action] [parameters]
@bot [natural language request]
[Interactive button/select menu]
```

**Platform-Specific Examples:**

**Telegram:**
```
/agent run daily-report
@aofbot show my tasks
```

**Discord:**
```
/agent run task:daily-report
/aof-task create name:"Review docs"
[Right-click message → Apps → Process with AOF]
```

**Slack:**
```
/agent run daily-report
@AOFBot help me with task management
[Click button on interactive message]
```

**WhatsApp:**
```
User: /agent run daily-report
Bot: [Interactive button list]
     "Choose action:
     [Run Report] [Schedule Report] [View History]"
```

---

## 10. Sources and References

### WhatsApp Business API
- [Implementing Webhooks From The WhatsApp Business Platform](https://business.whatsapp.com/blog/how-to-use-webhooks-from-whatsapp-business-api)
- [Webhooks for Conversational WhatsApp](https://academy.useinsider.com/docs/webhooks-for-conversational-whatsapp)
- [Setup WhatsApp Business API in 2025](https://botpenguin.com/blogs/setup-whatsapp-business-api)
- [How to Set Up WhatsApp Business API: Complete Guide (2025)](https://www.socialintents.com/blog/how-to-set-up-whatsapp-business-api/)
- [WhatsApp Inbound Message Webhook Examples](https://docs.ycloud.com/reference/whatsapp-inbound-message-webhook-examples)

### Telegram Bot API
- [Telegram Bot API Official Documentation](https://core.telegram.org/bots/api)
- [Telegram Bots FAQ](https://core.telegram.org/bots/faq)
- [Marvin's Marvellous Guide to All Things Webhook](https://core.telegram.org/bots/webhooks)
- [Understanding Telegram API Rate Limits](https://www.byteplus.com/en/topic/450604)
- [Telegram Limits](https://limits.tginfo.me/en)
- [Scaling Up IV: Flood Limits | grammY](https://grammy.dev/advanced/flood)

### Slack Events API
- [The Events API | Slack Developer Docs](https://docs.slack.dev/apis/events-api/)
- [Using Socket Mode | Slack Developer Docs](https://docs.slack.dev/apis/events-api/using-socket-mode/)
- [Rate limits | Slack Developer Docs](https://docs.slack.dev/apis/web-api/rate-limits/)
- [Rate limit changes for non-Marketplace apps](https://api.slack.com/changelog/2025-05-terms-rate-limit-update-and-faq)
- [Slack Webhooks: Complete Guide with Signature Verification [2025]](https://inventivehq.com/blog/slack-webhooks-guide)

### Discord Gateway API
- [Discord Bot Permissions and Intents Explained (2025)](https://friendify.net/blog/discord-bot-permissions-and-intents-explained-2025.html)
- [A Primer to Gateway Intents - discord.py](https://discordpy.readthedocs.io/en/stable/intents.html)
- [Discord API Guide: Bots, Webhooks & Best Practices](https://www.tokenmetrics.com/blog/mastering-discord-integrations-api-essentials)
- [Rate Limits - Discord Webhooks Guide](https://birdie0.github.io/discord-webhooks-guide/other/rate_limits.html)

### Rust Crates
- [teloxide - GitHub](https://github.com/teloxide/teloxide)
- [telegram-bot - crates.io](https://crates.io/crates/telegram-bot)
- [slack-rust-rs - crates.io](https://crates.io/crates/slack-rust-rs)
- [whatsapp-cloud-api - crates.io](https://crates.io/crates/whatsapp-cloud-api)
- [serenity - GitHub](https://github.com/serenity-rs/serenity)

---

## Conclusion

This comprehensive research provides a solid foundation for integrating messaging platform APIs into the AOF framework. Each platform offers unique strengths:

- **Telegram** provides the simplest integration with excellent Rust support
- **Discord** offers rich interactions and a mature ecosystem
- **Slack** excels in workplace automation with powerful OAuth and Socket Mode
- **WhatsApp** reaches the largest user base but requires more complex setup

The recommended approach is to:
1. Start with Telegram for rapid prototyping
2. Add Discord for rich interactions
3. Implement Slack for enterprise use cases
4. Consider WhatsApp for customer-facing applications

The platform adapter architecture allows AOF to support multiple messaging platforms through a unified interface, enabling agents to operate seamlessly across different communication channels.

**Next Steps:**
1. Review this research with the team
2. Prioritize platforms based on use cases
3. Begin implementation with Telegram adapter
4. Iterate and expand to additional platforms

---

**Research Completed:** December 9, 2025
**Researcher:** Hive Mind Research Agent (Swarm ID: swarm-1765276942953-cr5mbgm0i)
**Task ID:** task-1765277083524-gy58szddy
