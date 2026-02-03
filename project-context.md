# Project Context

This repository implements a web frontend and backend to provide omnect specific features in a local environment, where the device might not be connected to the azure cloud.

## Rules

- Before committing any changes that affect the project structure, the "Project Structure" section in this file must be updated.

## 1. Architecture & Tech Stack

The project uses a Cargo workspace structure and implements the Crux framework's Core/Shell architecture for the frontend.

### Crux Architecture

The application follows the Crux pattern where:

- **Core** (`src/app/`): Contains all business logic, state management, and type definitions. Compiled to WASM for the web shell.
- **Shell** (`src/ui/`): Vue 3 UI that renders the Core's view model and processes effects (HTTP, WebSocket).
- **Shared Types** (`src/shared_types/`): Generates TypeScript bindings from Rust types using TypeGen.

#### Core/Shell Communication

1. Shell sends Events to Core (serialized via bincode)
2. Core processes events, updates Model, returns Commands
3. Shell processes Commands (render UI, make HTTP requests, manage WebSocket)
4. Shell reads ViewModel from Core for rendering

### Backend (`src/backend/`)
- **Role:** Web Service / API providing access to omnect device features
- **Frameworks:** Rust, Actix-web
- Contains frontend as static ressource

### Frontend Core (`src/app/`)
- **Role:** Business Logic, State Management (Platform-agnostic)
- **Frameworks:** Rust, Crux (compiled to WASM)

### Frontend Shell (`src/ui/`)
- **Role:** User Interface (Single Page Application)
- **Frameworks:** Vue 3, TypeScript, Vite

### Shared Types (`src/shared_types/`)
- **Role:** Type definitions shared between Backend/Core and Frontend
- **Frameworks:** TypeGen (generates TypeScript bindings)

## Build Commands

### Build Frontend (Crux Core WASM, Frontend (Vue 3), TypeScript Types)

```bash
# build all frontend artefacts
# This script performs:
# 1. Builds WASM module with wasm-pack
# 2. Generates TypeScript types from Rust (cargo build -p shared_types)
# 3. Removes .js files to force Vite to use .ts sources
# 4. Installs dependencies and builds UI with bun
scripts/build-frontend.sh
```

### Build and Deploy omnect-ui Docker Image

```bash
# Build and Deploy ARM64 image
./scripts/build-and-deploy-image.sh --arch arm64 --deploy --host <device-ip> --password <ssh-pwd>
```

### Build and run Image on Host

```bash
# Build and Deploy ARM64 image
./scripts/build-and-run-image.sh
```

## Test Commands

### Unit Tests (Rust)

```bash
# Run tests
cargo test --features mock
```

### End-to-End (e2e tests)

```bash
# Run all e2e tests
./scripts/run-e2e-tests.sh

# Run a single e2e test
 ./scripts/run-e2e-tests.sh -g 'my-test'

# Run all e2e tests located in a file
 ./scripts/run-e2e-tests.sh my-tests.spec.ts
```

## Project Structure

```text
omnect-ui/
├── Cargo.toml                    # Workspace root
├── Dockerfile                    # Multi-stage Docker build
├── scripts/                      # Build and test scripts
│   ├── build-frontend.sh         # Build WASM + Types + UI
│   ├── build-and-deploy-image.sh # Docker build and deploy
│   └── run-e2e-tests.sh          # Playwright test runner
├── src/
│   ├── app/                      # Crux Core (business logic)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # App struct, Effect enum, re-exports
│   │       ├── model.rs          # Model struct (application state)
│   │       ├── events.rs         # Event enum
│   │       ├── wasm.rs           # WASM FFI bindings
│   │       ├── macros.rs         # URL and log macros
│   │       ├── http_helpers.rs   # HTTP request utilities
│   │       ├── commands/         # Custom side-effect commands
│   │       │   ├── mod.rs
│   │       │   └── centrifugo.rs # Centrifugo WebSocket commands
│   │       ├── types/            # Domain types
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs       # Authentication types
│   │       │   ├── device.rs     # Device information types
│   │       │   ├── network.rs    # Network configuration types
│   │       │   ├── factory_reset.rs
│   │       │   └── update.rs     # Update validation types
│   │       └── update/           # Domain-based event handlers
│   │           ├── mod.rs        # Main dispatcher
│   │           ├── auth.rs       # Auth event handlers
│   │           ├── ui.rs         # UI state handlers
│   │           ├── websocket.rs  # WebSocket state handlers
│   │           └── device/       # Device domain handlers
│   ├── backend/                  # Rust backend (Actix-web)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs           # Application entry point
│   │   │   ├── api.rs            # API route handlers
│   │   │   ├── middleware.rs     # Auth middleware
│   │   │   ├── config.rs         # Configuration loading
│   │   │   ├── http_client.rs    # Internal HTTP client
│   │   │   ├── keycloak_client.rs
│   │   │   ├── omnect_device_service_client.rs
│   │   │   └── services/         # Business logic services
│   │   │       ├── mod.rs
│   │   │       ├── auth/         # Auth logic
│   │   │       ├── certificate.rs
│   │   │       ├── firmware.rs
│   │   │       └── network.rs
│   │   └── tests/                # Integration tests
│   ├── shared_types/             # TypeGen for TypeScript bindings
│   │   ├── Cargo.toml
│   │   ├── build.rs              # TypeGen build script
│   │   └── generated/            # Generated TypeScript types
│   └── ui/                       # Vue 3 Shell
│       ├── package.json
│       ├── playwright.config.ts  # E2E test configuration
│       ├── src/
│       │   ├── App.vue           # Root component
│       │   ├── main.ts           # UI entry point
│       │   ├── components/       # UI components
│       │   ├── composables/      # Logic & WASM bridge
│       │   │   ├── useCore.ts    # Main bridge + effect handlers
│       │   │   └── useCentrifugo.ts
│       │   ├── pages/            # Route components
│       │   │   ├── DeviceOverview.vue
│       │   │   ├── DeviceUpdate.vue
│       │   │   ├── Network.vue
│       │   │   └── Login.vue
│       │   ├── plugins/          # Router, Vuetify
│       │   └── types/            # UI-specific types
│       └── tests/                # Playwright E2E tests
└── project-context.md            # This file
```

### Key Files

**Frontend (Shell):**
- `src/ui/src/composables/useCore.ts` - Core WASM bridge + effect handlers
- `src/ui/src/composables/useCentrifugo.ts` - WebSocket client integration
- `src/ui/src/pages/DeviceOverview.vue` - Main device dashboard page

**Core:**
- `src/app/src/lib.rs` - App struct, Effect enum, and re-exports
- `src/app/src/model.rs` - Model struct (application state)
- `src/app/src/events.rs` - Event enum definitions
- `src/app/src/types/` - Domain types organized by domain
- `src/app/src/update/` - Domain-based event handlers
- `src/app/src/commands/centrifugo.rs` - Custom Centrifugo commands

**Backend:**
- `src/backend/src/main.rs` - Application entry point
- `src/backend/src/api.rs` - API route handlers
- `src/backend/src/services/` - Business logic services

**Scripts:**
- `scripts/build-frontend.sh` - Build complete frontend (WASM + TypeScript types + UI)
- `scripts/build-and-deploy-image.sh` - Docker build and deploy script
