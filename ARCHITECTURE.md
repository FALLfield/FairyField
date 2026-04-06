# FairyField Architecture

## Technology Stack

### Backend: Rust
- **Framework**: Tauri v2
- **Location**: `src-tauri/`
- **Main Files**:
  - `src-tauri/src/main.rs` - Application entry point
  - `src-tauri/src/lib.rs` - Core application logic and Tauri commands
  - `src-tauri/Cargo.toml` - Rust dependencies and configuration

### Frontend: TypeScript + Vue 3
- **Language**: TypeScript
- **Framework**: Vue 3 with Composition API
- **Build Tool**: Vite
- **Location**: `src/`
- **Main Files**:
  - `src/main.ts` - Frontend entry point
  - `src/App.vue` - Root Vue component with TypeScript
  - `tsconfig.json` - TypeScript configuration
  - `vite.config.ts` - Vite build configuration

## Verified Configuration

### TypeScript Setup ✓
- TypeScript compiler: v5.6.2
- Strict mode enabled
- Vue TypeScript support via `vue-tsc`
- Type checking integrated in build process

### Rust Setup ✓
- Rust edition: 2021
- Tauri v2 framework
- Serde for serialization
- Tauri plugin system enabled

### Build Verification ✓
- Frontend build: Successfully compiles TypeScript and Vue components
- Backend build: Successfully compiles Rust code with all dependencies
- Integration: Tauri API properly configured for Rust-TypeScript communication

## Communication Layer

The frontend and backend communicate through Tauri's IPC (Inter-Process Communication):
- Frontend uses `@tauri-apps/api` to invoke Rust commands
- Backend exposes commands via `#[tauri::command]` macro
- Example: `greet` command demonstrates bidirectional communication

## Project Structure

```
FairyField/
├── src/                    # TypeScript frontend
│   ├── main.ts            # Entry point
│   ├── App.vue            # Root component
│   └── assets/            # Static assets
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   └── lib.rs         # Core logic
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── package.json           # Node dependencies
├── tsconfig.json          # TypeScript config
└── vite.config.ts         # Build config
```
