# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

This repository contains a full-stack application with:

- **Backend**: Rust-based web service (Actix-web) in `src/backend/`
- **Crux Core**: Platform-agnostic business logic in `src/app/`
- **Frontend (Shell)**: Vue 3 TypeScript application in `src/ui/`
- **Shared Types**: TypeGen for generating TypeScript bindings in `src/shared_types/`

The project uses a Cargo workspace structure and implements the Crux framework's Core/Shell architecture for the frontend.

## Crux Architecture

The application follows the Crux pattern where:

- **Core** (`src/app/`): Contains all business logic, state management, and type definitions. Compiled to WASM for the web shell.
- **Shell** (`src/ui/`): Vue 3 UI that renders the Core's view model and processes effects (HTTP, WebSocket).
- **Shared Types** (`src/shared_types/`): Generates TypeScript bindings from Rust types using TypeGen.

### Core/Shell Communication

1. Shell sends Events to Core (serialized via bincode)
2. Core processes events, updates Model, returns Effects
3. Shell processes Effects (render UI, make HTTP requests, manage WebSocket)
4. Shell reads ViewModel from Core for rendering

## Build Commands

### Backend (Rust)

```bash
# Build from workspace root
cargo build

# Build with release optimizations
cargo build --release

# Build specific package
cargo build -p omnect-ui
cargo build -p omnect-ui-core

# Run tests
cargo test --features mock
cargo test -p omnect-ui-core

# Run clippy
cargo clippy --all-targets --features mock
cargo clippy -p omnect-ui-core

# Format code
cargo fmt
```

### Crux Core WASM

```bash
# Build WASM module (from src/app/ directory)
wasm-pack build --target web --out-dir ../ui/src/core/pkg
```

### Frontend (Vue 3)

```bash
# Build complete frontend (WASM + TypeScript types + UI) - recommended
./scripts/build-frontend.sh

# This script performs:
# 1. Builds WASM module with wasm-pack
# 2. Generates TypeScript types from Rust (cargo build -p shared_types)
# 3. Removes .js files to force Vite to use .ts sources
# 4. Installs dependencies and builds UI with bun

# Or manually from src/ui/ directory (use bun)
bun install          # Install dependencies
bun run dev          # Development server
bun run build        # Production build
bun run lint         # Lint code


# Run all e2e tests
./scripts/run-e2e-tests.sh

# Run a single e2e test
 ./scripts/run-e2e-tests.sh -g 'my-test'

# Run all e2e tests located in a file
 ./scripts/run-e2e-tests.sh my-tests.spec.ts
```

### TypeScript Type Generation (Manual)

```bash
# Generate TypeScript types from Rust
cargo build -p shared_types

# Remove .js files to force Vite to use .ts sources
find src/shared_types/generated/typescript -name "*.js" -delete
```

### Docker

```bash
# Build and run locally
./build-and-run-image.sh

# Build ARM64 image
./build-arm64-image.sh
```

## Project Structure

```text
omnect-ui/
├── Cargo.toml                    # Workspace root
├── dist/                         # Built frontend assets (gitignored)
├── src/
│   ├── app/                      # Crux Core (business logic)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # App struct, Capabilities, re-exports
│   │       ├── model.rs          # Model struct (application state)
│   │       ├── events.rs         # Event enum
│   │       ├── types/            # Domain types (organized by domain)
│   │       │   ├── mod.rs        # Module re-exports
│   │       │   ├── auth.rs       # Authentication types
│   │       │   ├── device.rs     # Device information types
│   │       │   ├── network.rs    # Network configuration types
│   │       │   ├── factory_reset.rs  # Factory reset types
│   │       │   ├── update.rs     # Update validation types
│   │       │   └── common.rs     # Common shared types
│   │       ├── wasm.rs           # WASM bindings
│       ├── update/           # Domain-based event handlers
│       │   ├── mod.rs        # Main dispatcher
│       │   ├── auth.rs       # Authentication handlers
│       │   ├── device.rs     # Device action handlers
│       │   ├── websocket.rs  # WebSocket/Centrifugo handlers
│       │   ├── ui.rs         # UI action handlers
│       │   └── device/
│       │       ├── mod.rs        # Device event dispatcher
│       │       ├── operations.rs # Device operations (reboot, factory reset, updates)
│       │       ├── reconnection.rs # Device reconnection handlers
│       │       └── network/      # Network configuration handlers
│       │           ├── mod.rs
│       │           ├── config.rs
│       │           ├── form.rs
│       │           └── verification.rs
│       └── capabilities/     # Custom capabilities (Centrifugo)
│   ├── backend/                  # Rust backend (Actix-web)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── services/         # Business logic services
│   │   │   ├── routes/           # HTTP route handlers
│   │   │   └── middleware/       # Auth middleware
│   │   ├── tests/                # Integration tests
│   │   └── config/               # Centrifugo config
│   ├── shared_types/             # TypeGen for TypeScript bindings
│   │   ├── Cargo.toml
│   │   ├── build.rs              # TypeGen build script
│   │   └── generated/            # Generated TypeScript types
│   │       └── typescript/
│   │           ├── types/        # Domain types
│   │           ├── bincode/      # Serialization
│   │           └── serde/        # De/serialization helpers
│   └── ui/                       # Vue 3 Shell
│       ├── package.json
│       └── src/
│           ├── composables/
│           │   ├── useCore.ts        # Core WASM bridge + effect handlers
│           │   └── useCentrifugo.ts  # WebSocket client (used by useCore)
│           ├── components/           # Vue components
│           ├── core/pkg/            # WASM package (gitignored)
│           └── types/               # UI-specific types
├── scripts/

├── Dockerfile                    # Multi-stage Docker build
└── CLAUDE.md                     # This file
```

## Migration Guide: Vue Components to Crux Core

### Full Crux Approach (Current)

All state lives in the Core, shell handles only effects:

1. **Initialize Core** - Load WASM module
2. **Subscribe to channels** - Core emits Centrifugo effects, shell executes
3. **Components read from viewModel** - Unified state from Core
4. **Events trigger Core logic** - Actions dispatched through Core, HTTP/WS handled by shell

### Example Migration (DeviceInfo)

```typescript
// Before: Direct Centrifugo subscriptions
const { subscribe, history } = useCentrifuge()
const online = ref(false)
const systemInfo = ref(undefined)

subscribe(updateOnlineStatus, CentrifugeSubscriptionType.OnlineStatus)

// After: Full Crux approach
const { viewModel, initialize, subscribeToChannels } = useCore()

onMounted(async () => {
  await initialize()
  subscribeToChannels()  // Core emits Centrifugo effects
})

const online = computed(() => viewModel.online_status?.iothub ?? false)
const systemInfo = computed(() => viewModel.system_info)
```

### Key Files

- `src/ui/src/composables/useCore.ts` - Core WASM bridge + HTTP/Centrifugo effect handlers
- `src/ui/src/components/DeviceInfoCore.vue` - Example migrated component
- `src/app/src/lib.rs` - App struct, Capabilities, and re-exports
- `src/app/src/model.rs` - Model struct (application state)
- `src/app/src/events.rs` - Event enum definitions
- `src/app/src/types/` - Domain types organized by domain (auth, device, network, etc.)
- `src/app/src/update/` - Domain-based event handlers (auth, device, websocket, ui)
- `src/app/src/capabilities/centrifugo.rs` - Custom Centrifugo capability
- `scripts/build-frontend.sh` - Build complete frontend (WASM + TypeScript types + UI)

## Code Architecture

1. Sort functions from public to private
2. Use static variables at function level if possible
3. Follow Crux Core/Shell separation for frontend state management

### Important: Always Use Crux Core for API Calls

**NEVER use direct `fetch()` calls in Vue components.** All API interactions must go through the Crux Core architecture:

❌ **Wrong** - Direct fetch in component:
```typescript
const res = await fetch("network", {
  method: "POST",
  body: JSON.stringify({ name: "eth0" })
})
```

✅ **Correct** - Use Crux Core events:
```typescript
const { setNetworkConfig } = useCore()
await setNetworkConfig(JSON.stringify({ name: "eth0" }))
```

**Field Naming Convention:**
- Backend APIs expect **snake_case** field names (e.g., `current_password`, `previous_ip`)
- Use `#[serde(rename_all = "camelCase")]` on Rust request structs when backend expects camelCase
- Always regenerate TypeScript types after modifying Rust types: `cargo build -p shared_types`

## QA, Pull Request and commit Guidelines

When submitting changes to this codebase:

1. Ensure all tests pass with `cargo test --features mock`
2. Ensure Crux Core tests pass with `cargo test -p omnect-ui-core`
3. If adding new functionality, include appropriate tests
4. Ensure `cargo clippy` succeeds without warnings
5. Ensure correct formatting with `cargo fmt`
6. don't append Claude related text to pull requests or commits, such as containing text like "Generated with" or "Co-Authored-By:"
7. append "Signed-off-by: Jan Zachmann <50990105+JanZachmann@users.noreply.github.com>" to commit messages
8. Always get approval for description part of commits and pull requests
9. use the GitHub API directly to update the PR description
10. target branch of PR is upstream/main
11. don't format! like this format!("{}", var) instead always write format!("{var}")
12. ods is the abrivation for omnect-device-service
