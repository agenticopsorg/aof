# Telegram Bot Setup Guide for AOF

## Prerequisites

1. **Create a Telegram Bot**
   - Open Telegram and search for `@BotFather`
   - Send `/newbot` and follow instructions
   - Save the bot token (format: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

2. **Set Up Webhook Endpoint**
   - Need a public HTTPS URL (e.g., `https://yourdomain.com/webhook/telegram`)
   - Must have valid SSL certificate
   - Can use ngrok for local testing

## Quick Start

### 1. Configure Telegram Platform

```rust
use aof_triggers::platforms::{PlatformConfig, TelegramPlatform, TelegramConfig};

// Basic configuration
let config = PlatformConfig {
    platform: "telegram".to_string(),
    api_token: Some("YOUR_BOT_TOKEN".to_string()),
    webhook_secret: Some("YOUR_SECRET_TOKEN".to_string()),
    webhook_url: Some("https://yourdomain.com/webhook/telegram".to_string()),
};

let platform = TelegramPlatform::new(config);
```

### 2. Extended Configuration (with Security)

```rust
// Extended configuration with whitelisting
let telegram_config = TelegramConfig {
    bot_token: "YOUR_BOT_TOKEN".to_string(),
    webhook_url: "https://yourdomain.com/webhook/telegram".to_string(),
    webhook_secret: Some("YOUR_SECRET_TOKEN".to_string()),
    allowed_users: Some(vec![
        123456789,  // Your Telegram user ID
        987654321,  // Team member's user ID
    ]),
    allowed_groups: Some(vec![
        -100123456789,  // Private group ID
    ]),
    api_endpoint: None,  // Use default api.telegram.org
};

let platform_config = PlatformConfig {
    platform: "telegram".to_string(),
    api_token: Some(telegram_config.bot_token.clone()),
    webhook_secret: telegram_config.webhook_secret.clone(),
    webhook_url: Some(telegram_config.webhook_url.clone()),
};

let platform = TelegramPlatform::new_with_config(platform_config, telegram_config);
```

### 3. Set Up Webhook

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize platform
    let platform = TelegramPlatform::new(config);

    // Register webhook with Telegram
    platform.set_webhook().await?;

    println!("✅ Webhook configured successfully!");

    Ok(())
}
```

## Webhook Handler

### Basic Axum Server Example

```rust
use axum::{
    extract::{State, Json},
    http::{StatusCode, HeaderMap},
    routing::post,
    Router,
};
use serde_json::Value;
use aof_triggers::platforms::{TelegramPlatform, Platform};

struct AppState {
    telegram: TelegramPlatform,
}

async fn telegram_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<StatusCode, StatusCode> {
    // Verify webhook signature
    let signature = headers
        .get("X-Telegram-Bot-Api-Secret-Token")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let payload_bytes = serde_json::to_vec(&payload)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let is_valid = state.telegram
        .verify_signature(&payload_bytes, signature)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !is_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Parse message
    let message = state.telegram
        .parse_message(&payload_bytes)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Process message asynchronously
    tokio::spawn(async move {
        if let Err(e) = process_telegram_message(state.telegram, message).await {
            eprintln!("Error processing message: {}", e);
        }
    });

    Ok(StatusCode::OK)
}

async fn process_telegram_message(
    platform: TelegramPlatform,
    message: Message,
) -> Result<(), Box<dyn std::error::Error>> {
    // Extract command
    let text = &message.text;

    // Simple command routing
    if text.starts_with("/start") {
        let help_text = TelegramPlatform::create_help_text();
        platform.send_message(
            message.chat_id.parse()?,
            &help_text,
            Some("MarkdownV2"),
        ).await?;
    } else if text.starts_with("/agent") {
        // Handle agent commands
        handle_agent_command(&platform, &message).await?;
    } else if text.starts_with("/task") {
        // Handle task commands
        handle_task_command(&platform, &message).await?;
    } else {
        // Unknown command
        platform.send_message(
            message.chat_id.parse()?,
            "Unknown command. Send /help for available commands.",
            None,
        ).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create platform
    let platform = TelegramPlatform::new(config);

    // Set webhook
    platform.set_webhook().await?;

    // Create app state
    let state = AppState { telegram: platform };

    // Build router
    let app = Router::new()
        .route("/webhook/telegram", post(telegram_webhook))
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    println!("🚀 Server listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
```

## Command Handling Examples

### Agent Commands

```rust
async fn handle_agent_command(
    platform: &TelegramPlatform,
    message: &Message,
) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = message.text.split_whitespace().collect();

    match parts.get(1).map(|s| *s) {
        Some("run") => {
            // /agent run sre-bot "check health"
            let agent_name = parts.get(2).unwrap_or(&"");
            let prompt = parts.get(3..).map(|p| p.join(" ")).unwrap_or_default();

            // Start agent task
            let task_id = start_agent_task(agent_name, &prompt).await?;

            let response = format!(
                "✅ *Agent started*\n\nTask ID: `{}`\nAgent: `{}`\n\nUse `/task status {}` to check progress",
                TelegramPlatform::escape_markdown(&task_id),
                TelegramPlatform::escape_markdown(agent_name),
                TelegramPlatform::escape_markdown(&task_id)
            );

            platform.send_message(
                message.chat_id.parse()?,
                &response,
                Some("MarkdownV2"),
            ).await?;
        }
        Some("list") => {
            // /agent list
            let agents = list_available_agents().await?;
            let mut response = String::from("*Available Agents:*\n\n");

            for agent in agents {
                response.push_str(&format!(
                    "• `{}` \\- {}\n",
                    TelegramPlatform::escape_markdown(&agent.name),
                    TelegramPlatform::escape_markdown(&agent.description)
                ));
            }

            platform.send_message(
                message.chat_id.parse()?,
                &response,
                Some("MarkdownV2"),
            ).await?;
        }
        Some("status") => {
            // /agent status agent-123
            let agent_id = parts.get(2).unwrap_or(&"");
            let status = get_agent_status(agent_id).await?;

            let response = format!(
                "*Agent Status*\n\nID: `{}`\nStatus: `{}`\nUptime: {} seconds",
                TelegramPlatform::escape_markdown(agent_id),
                TelegramPlatform::escape_markdown(&status.state),
                status.uptime_seconds
            );

            platform.send_message(
                message.chat_id.parse()?,
                &response,
                Some("MarkdownV2"),
            ).await?;
        }
        _ => {
            platform.send_message(
                message.chat_id.parse()?,
                "Usage: `/agent run <name> \"<prompt>\"` or `/agent list`",
                Some("Markdown"),
            ).await?;
        }
    }

    Ok(())
}
```

### Task Commands with Buttons

```rust
async fn handle_task_command(
    platform: &TelegramPlatform,
    message: &Message,
) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = message.text.split_whitespace().collect();

    match parts.get(1).map(|s| *s) {
        Some("status") => {
            let task_id = parts.get(2).unwrap_or(&"");
            let task = get_task_status(task_id).await?;

            // Create response with action buttons
            let response_text = format!(
                "*Task Status*\n\nID: `{}`\nStatus: `{}`\nProgress: {}%",
                TelegramPlatform::escape_markdown(task_id),
                TelegramPlatform::escape_markdown(&task.status),
                task.progress
            );

            // Note: Full button support requires integration with response module
            // This example shows the API structure
            platform.send_message(
                message.chat_id.parse()?,
                &response_text,
                Some("MarkdownV2"),
            ).await?;
        }
        Some("cancel") => {
            let task_id = parts.get(2).unwrap_or(&"");
            cancel_task(task_id).await?;

            let response = format!(
                "✅ Task `{}` has been cancelled",
                TelegramPlatform::escape_markdown(task_id)
            );

            platform.send_message(
                message.chat_id.parse()?,
                &response,
                Some("MarkdownV2"),
            ).await?;
        }
        _ => {
            platform.send_message(
                message.chat_id.parse()?,
                "Usage: `/task status <id>` or `/task cancel <id>`",
                Some("Markdown"),
            ).await?;
        }
    }

    Ok(())
}
```

## Testing Locally with ngrok

### 1. Install ngrok
```bash
# macOS
brew install ngrok

# Or download from https://ngrok.com/download
```

### 2. Start ngrok tunnel
```bash
ngrok http 8080
```

This will output:
```
Forwarding  https://abc123.ngrok.io -> http://localhost:8080
```

### 3. Update webhook URL
Use the ngrok URL in your configuration:
```rust
webhook_url: "https://abc123.ngrok.io/webhook/telegram".to_string()
```

### 4. Test commands
Send messages to your bot on Telegram:
- `/start` - See welcome message
- `/help` - View command list
- `/agent list` - Test agent command

## Security Best Practices

### 1. Use Webhook Secret
Always configure a webhook secret:
```rust
webhook_secret: Some(generate_random_secret(32))
```

### 2. Enable User Whitelisting
Restrict access to authorized users:
```rust
allowed_users: Some(vec![YOUR_USER_ID])
```

To get your Telegram user ID:
- Message `@userinfobot` on Telegram
- Or use `message.user_id` from first message

### 3. Use Environment Variables
Never hardcode tokens:
```rust
use std::env;

let bot_token = env::var("TELEGRAM_BOT_TOKEN")?;
let webhook_secret = env::var("TELEGRAM_WEBHOOK_SECRET")?;
```

### 4. HTTPS Only
Telegram requires HTTPS for webhooks:
- Use Let's Encrypt for free SSL certificates
- Or use a reverse proxy (nginx, Caddy)

## Production Deployment

### Docker Example

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/aof-bot /usr/local/bin/
CMD ["aof-bot"]
```

### Environment Variables
```bash
TELEGRAM_BOT_TOKEN=123456789:ABCdefGHIjklMNOpqrsTUVwxyz
TELEGRAM_WEBHOOK_SECRET=your-secret-token
TELEGRAM_WEBHOOK_URL=https://yourdomain.com/webhook/telegram
TELEGRAM_ALLOWED_USERS=123456789,987654321
```

### systemd Service
```ini
[Unit]
Description=AOF Telegram Bot
After=network.target

[Service]
Type=simple
User=aof
WorkingDirectory=/opt/aof-bot
EnvironmentFile=/etc/aof-bot/config.env
ExecStart=/usr/local/bin/aof-bot
Restart=always

[Install]
WantedBy=multi-user.target
```

## Monitoring

### Health Check Endpoint
```rust
async fn health_check() -> StatusCode {
    StatusCode::OK
}

let app = Router::new()
    .route("/webhook/telegram", post(telegram_webhook))
    .route("/health", get(health_check));
```

### Metrics
Consider adding:
- Message processing time
- Command success/failure rates
- Active user count
- Error rates

### Logging
```rust
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_target(false)
    .with_thread_ids(true)
    .with_level(true)
    .init();
```

## Troubleshooting

### Webhook Not Receiving Messages

1. **Check webhook status:**
```bash
curl https://api.telegram.org/bot<TOKEN>/getWebhookInfo
```

2. **Verify HTTPS:**
- Must have valid SSL certificate
- Self-signed certificates won't work

3. **Check firewall:**
- Allow inbound HTTPS (443)
- Or custom port if using reverse proxy

### Unauthorized Errors

1. **Verify bot token:**
```bash
curl https://api.telegram.org/bot<TOKEN>/getMe
```

2. **Check webhook secret:**
- Must match configuration
- Case-sensitive

3. **Verify user ID:**
- Use `@userinfobot` to get correct ID
- Check `allowed_users` list

### Message Not Sent

1. **Check response format:**
- Telegram is strict about MarkdownV2
- Use `escape_markdown()` for user content

2. **Verify chat_id:**
- Must be valid integer
- Negative for groups

3. **Check bot permissions:**
- Bot must be added to group
- Must have send message permission

## Additional Resources

- [Telegram Bot API Documentation](https://core.telegram.org/bots/api)
- [AOF Triggers Module](../aof/crates/aof-triggers/)
- [Example Bot Implementation](../examples/telegram-bot/)

---

*For questions or issues, please refer to the main AOF documentation or open an issue on GitHub.*
