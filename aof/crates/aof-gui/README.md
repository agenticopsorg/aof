# AOF Desktop GUI

Desktop GUI client for the Agent Operating Framework (AOF), built with Tauri v2, React, and TypeScript.

## Features

- 🖥️ Native desktop application (Windows, macOS, Linux)
- ⚡ Fast and lightweight with Tauri v2
- 🎨 Modern UI with React and Tailwind CSS
- 🔧 TypeScript for type safety
- 🚀 Vite for blazing-fast development

## Prerequisites

- Rust (latest stable)
- Node.js 18+ and npm
- Platform-specific dependencies:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: webkit2gtk, libayatana-appindicator
  - **Windows**: WebView2

## Quick Start

### Development

1. **Install frontend dependencies:**
   ```bash
   cd ui
   npm install
   cd ..
   ```

2. **Run in development mode:**
   ```bash
   cargo tauri dev
   ```

   This will:
   - Start the Vite dev server (React frontend)
   - Build the Rust backend
   - Launch the application with hot-reload

### Building

```bash
# Build for production
cargo tauri build

# Build artifacts will be in:
# target/release/bundle/
```

## Project Structure

```
aof-gui/
├── src/                 # Rust backend
│   ├── main.rs          # Entry point
│   ├── lib.rs           # Library exports
│   └── commands/        # Tauri command handlers
├── ui/                  # React frontend
│   ├── src/
│   │   ├── main.tsx     # React entry
│   │   ├── App.tsx      # Main component
│   │   └── components/  # UI components
│   ├── package.json
│   └── vite.config.ts
├── tauri.conf.json      # Tauri configuration
└── Cargo.toml           # Rust dependencies
```

## Available Commands

### Frontend (ui/)
- `npm run dev` - Start Vite dev server
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm run lint` - Run ESLint
- `npm run typecheck` - Run TypeScript type checking

### Backend (root)
- `cargo build` - Build Rust backend
- `cargo test` - Run tests
- `cargo tauri dev` - Run in development mode
- `cargo tauri build` - Build for production

## Integration with AOF Core

The GUI integrates with `aof-core` through:
- Direct crate dependencies in Cargo.toml
- Tauri commands that call AOF core functionality
- Shared data structures via serde serialization

## Configuration

### App Metadata
Edit `tauri.conf.json` to customize:
- App name and version
- Window size and behavior
- Bundle settings
- Security policies

### Frontend
Edit `ui/vite.config.ts` for:
- Build optimization
- Path aliases
- Plugin configuration

## Development Tips

1. **Hot Reload**: Changes to React code trigger instant updates
2. **DevTools**: Press F12 to open Chrome DevTools in development
3. **Debug Rust**: Use `println!` or proper logging with `log` crate
4. **Type Safety**: Use TypeScript interfaces that match Rust structs

## Next Steps

- Add more Tauri commands to expose AOF functionality
- Create React components for agent management
- Implement state management (Context API or Zustand)
- Add routing for multi-page navigation
- Create proper error handling and notifications

## Resources

- [Tauri Documentation](https://tauri.app/v2/)
- [React Documentation](https://react.dev/)
- [Vite Documentation](https://vitejs.dev/)
- [Tailwind CSS](https://tailwindcss.com/)
