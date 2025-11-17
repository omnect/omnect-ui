# Crux Core for omnect-ui

This package contains the Crux Core for the omnect-ui application. It implements the business logic and state management in Rust, which can be compiled to WebAssembly for use in the Vue frontend.

## Architecture

The Crux Core follows the Model-View-Update pattern:

- **Model** - The complete application state (auth, device info, network status, etc.)
- **ViewModel** - Data needed by the UI to render
- **Events** - Actions that can occur in the application
- **Capabilities** - Side effects (HTTP requests, WebSocket, rendering)

## Key Files

- `src/lib.rs` - App struct, Capabilities, and re-exports
- `src/model.rs` - Model and ViewModel structs
- `src/events.rs` - Event enum definitions
- `src/types.rs` - Shared data types
- `src/update/` - Domain-based event handlers
  - `mod.rs` - Main dispatcher and view function
  - `auth.rs` - Authentication handlers (login, logout, password)
  - `device.rs` - Device action handlers (reboot, factory reset, network, updates)
  - `websocket.rs` - WebSocket/Centrifugo handlers
  - `ui.rs` - UI action handlers (clear error/success)
- `src/capabilities/centrifugo.rs` - Custom WebSocket capability (deprecated API, kept for Effect enum generation)
- `src/capabilities/centrifugo_command.rs` - Command-based WebSocket capability (new API)

## Building

### For Testing

```bash
cargo test -p omnect-ui-core
```

### For WASM (Web)

First, install wasm-pack:

```bash
cargo install wasm-pack
```

Then build:

```bash
cd src/app
wasm-pack build --target web --out-dir ../ui/src/core/pkg
```

This will generate the WASM module in `src/ui/src/core/pkg/`.

### Generate TypeScript Types

Make sure pnpm is in your PATH, then:

```bash
cargo build -p shared_types
```

This generates TypeScript types in `src/shared_types/generated/typescript/`.

## Integration with Vue

The Vue shell uses the `useCore()` composable (in `src/ui/src/composables/useCore.ts`) to interact with the Crux Core:

```typescript
const { viewModel, sendEvent, login, logout } = useCore()

// Send an event
await login('username', 'password')

// Access the view model
const isLoading = computed(() => viewModel.is_loading)
const errorMessage = computed(() => viewModel.error_message)
```

## Event Flow

1. User action in Vue component
2. Vue calls `sendEvent()` or convenience method
3. Event is serialized and sent to WASM core
4. Core updates Model and returns Effects
5. Effects are processed (HTTP requests, render updates, etc.)
6. ViewModel is updated and Vue re-renders

## Capabilities

### Render
Updates the ViewModel to trigger UI re-rendering.

### HTTP
Makes REST API calls to the backend. The shell handles the actual HTTP request and sends the response back to the core.

### Centrifugo
Manages WebSocket subscriptions for real-time updates. The shell handles the actual WebSocket connection.

## Testing

The core includes unit tests for business logic:

```bash
cargo test -p omnect-ui-core
```

Run with clippy:

```bash
cargo clippy -p omnect-ui-core -- -D warnings
```

## Future Work

- [x] Complete WASM integration with wasm-pack
- [x] Implement full effect processing in Vue shell
- [x] Migrate all state management from Vue stores to Crux Core
- [ ] Add more comprehensive tests for edge cases (currently only 3 basic tests)
- [x] Migrate Centrifugo capability to Command API (non-deprecated)
- [x] Migrate HTTP capability to Command API (non-deprecated)
- [x] Split monolithic lib.rs into domain-based modules
- [x] Suppress deprecated warnings with module-level `#![allow(deprecated)]`
- [ ] Remove deprecated capabilities once crux_core provides alternative Effect generation mechanism
- [ ] Introduce shared_types crate for types shared between backend API and Crux Core
