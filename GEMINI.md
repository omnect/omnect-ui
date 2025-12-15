# GEMINI.md

This file provides guidance to Gemini when working with code in this repository.

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

### complete build and deployment to rpi4

# build all components and deploy to target
./build-and-deploy-image.sh --deploy --host 192.168.0.100 --user omnect --password omnect

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
│   │       ├── update/           # Domain-based event handlers
│   │       │   ├── mod.rs        # Main dispatcher
│   │       │   ├── auth.rs       # Authentication handlers
│   │       │   ├── device.rs     # Device action handlers
│   │       │   ├── websocket.rs  # WebSocket/Centrifugo handlers
│   │       │   └── ui.rs         # UI action handlers
│   │       └── capabilities/     # Custom capabilities (Centrifugo)
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
└── GEMINI.md                     # This file
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
7. append "Signed-off-by: Jan Zachmann <50990105+JanZachmann@users.noreply.github.com>" to commit messages
8. Always get approval for description part of commits and pull requests
9. use the GitHub API directly to update the PR description
10. target branch of PR is upstream/main
11. don't format! like this format!("{}", var) instead always write format!("{var}")
12. ods is the abrivation for omnect-device-service

## Testing Guidelines

- **Navigation Rule**: When testing, always navigate within the application by **clicking on UI elements (buttons, links, menu items)**. **NEVER use direct URL navigation** (e.g., by typing into the URL bar or using `navigate_page(url)` / `new_page(url)`) for internal application routes (e.g., `/network`, `/update`). Direct URL navigation can cause full page reloads, which may reset WASM state and authentication status, leading to inconsistent test results.

- **Build and Deploy**: To build and deploy the application to a target device (e.g., Raspberry Pi 4), use the following command:
  ```bash
  ./build-and-deploy-image.sh --deploy --host 192.168.0.100 --user omnect --password omnect
  ```

- **Post-Deployment Testing**: After a build and deployment, **always await explicit user confirmation** before starting any tests. The user will prepare the login screen as the starting point.

### Gemini Context (Session Findings)

### Critical Workarounds & Fixes

1.  **Crux HTTP Relative URLs:**
    *   **Issue:** `crux_http` (v0.15) panics when provided with relative URLs (e.g. `/token/login`).
    *   **Fix:** URLs in `src/app/src/macros.rs` and manual handlers MUST be prefixed with a dummy domain: `http://omnect-device`.
    *   **Handling:** The UI shell (`src/ui/src/composables/useCore.ts`) detects this prefix and strips it before making the actual `fetch` call.

2.  **Serialization Alignment (`auth_token`):**
    *   **Issue:** `shared_types` generator creates TypeScript types expecting `auth_token` in `Model`, but Rust struct had `#[serde(skip_serializing)]`. This caused "off-by-one" deserialization errors, leading to null `success_message`.
    *   **Fix:** `#[serde(skip_serializing)]` was removed from `auth_token` in `src/app/src/model.rs`. Do NOT restore it unless type generation config is updated to exclude it.

3.  **Login Authentication Method:**
    *   **Issue:** Backend middleware (`AuthMw`) expects Basic Auth header for login, but frontend was sending JSON body.
    *   **Fix:** `Event::Login` handler in `src/app/src/update/auth.rs` manually constructs `Authorization: Basic ...` header. Does NOT use `body_json`.

4.  **VS Code Debugging:**
    *   **Issue:** Standard "Debug executable" launch config does NOT rebuild WASM/Frontend.
    *   **Fix:** `scripts/dev-setup.sh` has been updated to include `wasm-pack build` and `pnpm run build`. Ensure this script runs before debugging.

5.  **Navigation Rule:**
    *   **Rule:** Do NOT use `navigate_page(url)` or `new_page(url)` to jump to internal application routes (e.g. `/network`, `/update`). ALWAYS start at the root/login and navigate using `click()` on UI elements (menu, buttons).
    *   **Reason:** Direct URL navigation causes full page reloads which clear the WASM state and authentication status, leading to false negatives in tests.

6.  **Locating Dynamic UI Elements (e.g., Logout Button):**
    *   **Challenge:** UI elements within overlays or dropdown menus (like the user menu containing "Logout") are often not immediately present in the DOM snapshot. Simple `take_snapshot()` might miss them if the menu is not active/open. `wait_for(text='...')` might time out if the element isn't present in the DOM for its entire duration.
    *   **Strategy:**
        1.  **Click the Trigger Element:** First, identify and click the UI element that *triggers* the display of the target element (e.g., the user avatar or a menu icon that opens a dropdown). In this session, the user menu was opened by clicking the avatar button (identified by its content or data attributes).
        2.  **Take a Fresh Snapshot:** Immediately after clicking the trigger, take a new snapshot. This is crucial as it refreshes the DOM model and captures the newly rendered overlay/dropdown content.
        3.  **Locate by Text/Attributes:** Within this *fresh* snapshot, identify the target element (e.g., the "LOGOUT" button) by its visible text or any unique attributes (`data-cy`, `id`, `class`). Use this information to retrieve its `uid`.
    *   **Example (Logout Button):** In our tests, the logout button (`uid=23_41` in a recent snapshot) appeared after clicking the user avatar (which was `uid=19_2` in a previous snapshot).

### Cleanups
*   Deprecated `setup-password` binary and its usage in `scripts/dev-setup.sh` have been removed.
