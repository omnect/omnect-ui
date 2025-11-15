# Shared Types Crate Analysis: Backend and Crux Core

## Executive Summary

**Recommendation: Do NOT share types between backend and Crux Core.** The types serve fundamentally different purposes and have incompatible structures. Instead, consider a shared API types crate for HTTP request/response DTOs if needed.

## Current State

### Backend Types (`src/backend/src/omnect_device_service_client.rs`)

These types are designed to **parse responses from external services** (omnect-device-service):

```rust
pub struct SystemInfo {
    pub fleet_id: Option<String>,
    pub omnect_device_service_version: String,
}

pub struct NetworkStatus {
    pub network_interfaces: Vec<NetworkInterface>,  // Has file: PathBuf
}

pub struct VersionInfo {
    pub required: String,
    pub current: String,
    pub mismatch: bool,
}
```

### Core Types (`src/app/src/types.rs`)

These types represent **UI state for display**:

```rust
pub struct SystemInfo {
    pub os: OsInfo,  // name, version
    pub azure_sdk_version: String,
    pub omnect_device_service_version: String,
    pub boot_time: Option<String>,
}

pub struct NetworkStatus {
    pub network_status: Vec<DeviceNetwork>,  // Has mac, detailed IP info
}

pub struct VersionInfo {
    pub version: String,
    pub git_sha: String,
}
```

## Detailed Comparison

| Type | Backend Purpose | Core Purpose | Compatible? |
|------|----------------|--------------|-------------|
| `SystemInfo` | Parse fleet_id, service version | Display OS info, boot time, SDK versions | **No** |
| `NetworkStatus` | Parse file paths, basic network info | Display MAC, detailed IPv4 config | **No** |
| `VersionInfo` | Version compatibility checking | Display version and git SHA | **No** |
| `HealthcheckInfo` | Compose from external data | Display to user | **No** (different VersionInfo) |

## Pros of Sharing Types

1. **Single Source of Truth** - Prevents type drift between components
2. **DRY Principle** - Eliminates code duplication
3. **Type Safety** - Compile-time guarantees across boundaries
4. **Automatic Propagation** - Changes update both sides simultaneously
5. **API Consistency** - Backend and frontend speak same language

## Cons of Sharing Types

1. **Incompatible Structures** - Current types are already fundamentally different
2. **Different Responsibilities**
   - Backend: Parse external service responses
   - Core: Represent UI state for display
3. **Coupling Risk** - Changes to external API force Core changes
4. **Serde Configuration Conflicts**
   - Backend may need `#[serde(rename = "NetworkStatus")]`
   - Core needs clean TypeGen output
5. **Feature Gate Complexity** - Would need `#[cfg]` to exclude WASM-specific features
6. **Build Dependency Issues**
   - Backend: Needs `Deserialize` only
   - Core: Needs both `Serialize` and `Deserialize` for TypeGen
7. **Violates Separation of Concerns** - Mixes external API representation with internal state

## Architecture Implications

### Current (Correct) Architecture

```
External Service → Backend Types → Transform → API Response
                                      ↓
                                  HTTP/JSON
                                      ↓
                              Core Types (via Shell effects)
                                      ↓
                                  ViewModel
```

The backend transforms external data into a standardized API format, which the Core then consumes and transforms into its internal state.

### If Types Were Shared (Problematic)

```
External Service → Shared Types ← Core State
                        ↓
                   Tight Coupling
```

This would force Core types to match external service structure, breaking the Core/Shell separation principle.

## Alternative Recommendations

### Option 1: Keep Types Separate (Recommended)

- **Backend**: Types match external service structure
- **Core**: Types optimized for UI state representation
- **Transformation**: Backend transforms data before sending to frontend

### Option 2: Shared API DTO Crate

Create a separate crate for HTTP request/response DTOs:

```
src/api-types/
  Cargo.toml
  src/
    auth.rs      # LoginRequest, AuthTokenResponse
    device.rs    # RebootRequest, FactoryResetRequest
```

These would be simple data transfer objects, not domain models.

### Option 3: Shared Domain Types (Not Recommended)

Only share truly universal types that have identical structure and semantics:
- Simple enums (FactoryResetStatus)
- Value objects (IpAddress)

But this provides minimal benefit for significant complexity.

## Files to Review

| File | Contains | Purpose |
|------|----------|---------|
| `src/backend/src/omnect_device_service_client.rs:45-100` | Backend domain types | External service parsing |
| `src/app/src/types.rs:1-127` | Core domain types | UI state representation |
| `src/backend/src/api.rs:22-39` | API request DTOs | HTTP endpoints |

## Conclusion

The current separation is architecturally sound. The backend and Core types serve different purposes:

- **Backend types** model **external** service responses
- **Core types** model **internal** UI state

Forcing these to share a common base would:
1. Require significant refactoring of both codebases
2. Create tight coupling that violates Core/Shell separation
3. Mix external API concerns with internal state concerns
4. Complicate builds with feature gates and conditional compilation

**Keep types separate.** The transformation layer between backend responses and Core state is a feature, not a bug.
