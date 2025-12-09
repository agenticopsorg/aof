# AOF Desktop - Development Guide

## Prerequisites

- **Rust**: Install via [rustup](https://rustup.rs/)
- **Node.js**: v18+ recommended
- **pnpm**: Package manager (`npm install -g pnpm`)
- **Tauri CLI**: `cargo install tauri-cli`

## Project Structure

```
aof-gui/
├── Cargo.toml          # Rust dependencies
├── tauri.conf.json     # Tauri configuration
├── src/                # Rust backend
│   ├── main.rs         # Entry point
│   ├── lib.rs          # Command registration
│   ├── state.rs        # App state management
│   └── commands/       # Tauri IPC commands
│       ├── agent.rs    # Agent run/stop/status
│       ├── config.rs   # Config validation/storage
│       └── mcp.rs      # MCP server connections
├── ui/                 # React frontend
│   ├── package.json    # Uses pnpm
│   ├── src/
│   │   └── App.tsx     # Main UI component
│   └── dist/           # Built frontend (generated)
├── icons/              # App icons
└── docs/               # Documentation
```

## Development Setup

```bash
cd aof/crates/aof-gui

# Install frontend dependencies
cd ui && pnpm install && cd ..

# Run in development mode (hot reload)
cargo tauri dev
```

## Building for Production

```bash
cd aof/crates/aof-gui

# Install dependencies if needed
cd ui && pnpm install && cd ..

# Build release bundle
cargo tauri build
```

Output locations:
- **macOS**: `target/release/bundle/dmg/AOF Desktop.dmg`
- **Windows**: `target/release/bundle/msi/AOF Desktop.msi`
- **Linux**: `target/release/bundle/appimage/aof-desktop.AppImage`

## Adding New Commands

1. Create handler in `src/commands/`:

```rust
#[tauri::command]
pub async fn my_command(
    state: State<'_, AppState>,
    param: String,
) -> Result<MyResponse, String> {
    // Implementation
}
```

2. Register in `src/lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    commands::my_command,
])
```

3. Call from frontend:

```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('my_command', { param: 'value' });
```

## Frontend Development

The UI uses:
- **React 18** with TypeScript
- **Vite** for bundling
- **Tailwind CSS** for styling
- **Lucide React** for icons

```bash
cd ui

# Run linter
pnpm lint

# Type check
pnpm typecheck

# Build only frontend
pnpm build
```

## Testing

```bash
# Check Rust compilation
cargo check -p aof-gui

# Run Rust tests
cargo test -p aof-gui

# Frontend type checking
cd ui && pnpm typecheck
```

## Debugging

- **Backend logs**: Set `RUST_LOG=debug` environment variable
- **Frontend**: Use browser DevTools (Cmd+Option+I in dev mode)
- **Tauri**: Check `~/.aof/logs/` for application logs
