# Test Coverage Analysis - omnect-ui Workspace

## Executive Summary

**Total Tests: 27** covering approximately 1-2% of the codebase

- **8 integration tests** (validate_portal_token, http_client)
- **19 unit tests** (middleware auth, token management, password hashing)
- **0 frontend tests** (Vue/TypeScript completely untested)
- **0 Crux Core tests** (no src/app/ directory exists)

---

## Current Test Statistics

| Category | Count | Status |
|----------|-------|--------|
| **Integration Tests** | 8 | Very Low Coverage |
| **Unit Tests** | 19 | Low Coverage |
| **Total Tests** | 27 | ~1% of codebase |
| **Service Methods** | 18+ public methods | ~20% tested |
| **API Routes** | 17+ routes | ~6% tested (1/17) |
| **Files with Tests** | 5 | ~20% of backend modules |
| **Frontend Tests** | 0 | 0% coverage |

---

## Implementation Strategy & Key Recommendations

### Approach: Multiple Focused PRs (NOT One Large PR)

**Total: 14 PRs organized in 4 phases**

**Why Multiple PRs?**
- Easier to review and approve
- Can be merged incrementally
- Reduces merge conflicts
- Each PR is independently valuable
- Easier to rollback if issues arise

### Phase Overview

| Phase | Duration | PRs | Coverage Goal | Priority |
|-------|----------|-----|---------------|----------|
| **Phase 1: Security & Stability** | Weeks 1-2 | 4 | 1% → 13% | 🔴 CRITICAL |
| **Phase 2: Core Device Operations** | Weeks 3-4 | 4 | 13% → 50% | 🔴 CRITICAL |
| **Phase 3: API Coverage** | Weeks 5-6 | 3 | 50% → 72% | 🟠 HIGH |
| **Phase 4: Frontend & E2E** | Weeks 7-8 | 3 | 72% → 85-90% | 🟡 MEDIUM |

### Critical Path (Must Do First)

1. **PR #1: `test/infrastructure-setup`** - Enables all other PRs (8-12 hours)
2. **PR #2: `test/auth-and-keycloak`** - Security critical (16-20 hours)
3. **PR #5: `test/device-service-client`** - Enables device operations (20-24 hours)
4. **PR #6: `test/network-configuration`** - Can brick devices! (20-28 hours)
5. **PR #7: `test/firmware-updates`** - Can brick devices! (16-20 hours)

### Parallelization Opportunities

After PR #1 is merged, these can be worked on in parallel:

- **Group A (Security):** PRs #2, #3, #4 - All only depend on PR #1
- **Group B (Device Ops):** PRs #6, #7, #8 - Depend on PR #1 and PR #5
- **Group C (API):** PRs #9, #10, #11 - Minimal dependencies
- **Group D (Frontend):** PRs #12, #13, #14 - Completely independent

### Timeline Estimates

- **Conservative (sequential, single developer):** 10 weeks
- **Optimistic (with parallelization):** 7 weeks
- **Minimum viable (Phases 1-2 only):** 4-5 weeks, 50% coverage

### PR List Summary

#### Phase 1: Security & Stability (🔴 CRITICAL)
1. `test/infrastructure-setup` - Test fixtures, mocks, CI/CD
2. `test/auth-and-keycloak` - Authorization & Keycloak SSO
3. `test/token-and-login` - Token/login endpoints
4. `test/password-management` - Password endpoints

#### Phase 2: Core Device Operations (🔴 CRITICAL)
5. `test/device-service-client` - Device service client (enables #6, #7, #9)
6. `test/network-configuration` - Network config (can brick devices!)
7. `test/firmware-updates` - Firmware updates (can brick devices!)
8. `test/certificate-service` - Certificate service

#### Phase 3: API Coverage (🟠 HIGH)
9. `test/api-endpoints` - Remaining API routes
10. `test/middleware-integration` - Middleware integration tests
11. `test/configuration` - Config loading tests

#### Phase 4: Frontend & E2E (🟡 MEDIUM)
12. `test/frontend-setup` - Frontend test infrastructure
13. `test/frontend-components` - Component tests
14. `test/e2e` - E2E tests (optional)

### Key Recommendations

1. ✅ **Start with PR #1 immediately** - Infrastructure enables everything else
2. ✅ **Prioritize Phase 1 completely before Phase 2** - Security is critical
3. ✅ **Consider parallel work on Group A** (PRs #2, #3, #4) after PR #1
4. ✅ **Do NOT skip PR #6 and #7** - Network and firmware can brick devices
5. ✅ **Get Phase 4 approved by stakeholders** - Frontend testing may not be immediate priority
6. ✅ **Consider stopping at Phase 3** (72% coverage) if resources are constrained

### Success Metrics

- **After Phase 1:** 13% coverage (security covered)
- **After Phase 2:** 50% coverage (critical operations covered)
- **After Phase 3:** 72% coverage (API coverage complete)
- **After Phase 4:** 85-90% coverage (full stack covered)

---

## Existing Test Coverage

### Integration Tests (`src/backend/tests/`)

#### 1. `validate_portal_token.rs` - 5 tests
Tests API endpoint `/validate` with SSO token validation:
- ✅ Fleet admin with correct tenant
- ✅ Fleet admin with invalid tenant (expects failure)
- ✅ Fleet operator with valid fleet
- ✅ Fleet operator with invalid fleet (expects failure)
- ✅ Fleet observer without proper role (expects failure)

#### 2. `http_client.rs` - 3 tests
Tests Unix socket HTTP client with mock server:
- ✅ GET request success
- ✅ POST request with JSON payload
- ✅ Multiple sequential requests

### Unit Tests in Source Files

#### Middleware (`src/backend/src/middleware.rs`) - 11 tests
- ✅ Token validation (valid, expired, invalid subject, invalid token)
- ✅ Basic auth credentials (correct/incorrect password)
- ✅ Session token handling
- ✅ JWT token creation and verification
- ✅ Expired token rejection
- ✅ Wrong secret rejection

#### Service Tests
| File | Tests | What's Tested |
|------|-------|---------------|
| `services/auth/token.rs` | 3 | Token creation, verification, wrong secret |
| `services/auth/password.rs` | 2 | Hash password, store/check password |
| `services/firmware.rs` | 1 | Clear data folder |
| `http_client.rs` | 2 | Unix socket path validation |

---

## 🔴 CRITICAL Must-Have Test Cases

### 1. Network Configuration Service (419 LOC - UNTESTED)
**Location:** `src/backend/src/services/network.rs`
**Why Critical:** Can brick devices by making them unreachable

**Must-have tests:**
```rust
#[cfg(test)]
mod tests {
    // Core functionality
    - test_set_network_config_with_valid_configuration()
    - test_set_network_config_creates_backup()
    - test_process_pending_rollback_restores_network_on_timeout()
    - test_cancel_rollback_prevents_automatic_rollback()

    // Edge cases
    - test_rollback_timer_mechanism_90_seconds()
    - test_systemd_networkd_file_generation()
    - test_concurrent_network_changes_rejected()
    - test_invalid_network_config_rejected()
    - test_rollback_with_corrupted_backup()
}
```

### 2. Firmware Update Flow (128 LOC - 1 TEST ONLY)
**Location:** `src/backend/src/services/firmware.rs`
**Why Critical:** Failed updates can brick devices

**Must-have tests:**
```rust
#[cfg(test)]
mod tests {
    // Core functionality
    - test_handle_uploaded_firmware_saves_file_with_correct_permissions()
    - test_load_update_calls_device_service_correctly()
    - test_run_update_executes_firmware_update()

    // Error handling
    - test_error_handling_when_device_service_unavailable()
    - test_cleanup_on_failure_scenarios()
    - test_invalid_firmware_file_rejected()
    - test_insufficient_disk_space_handled()
}
```

### 3. Device Service Client (353 LOC - UNTESTED)
**Location:** `src/backend/src/omnect_device_service_client.rs`
**Why Critical:** ALL device operations depend on this

**Must-have tests:**
```rust
#[cfg(test)]
mod tests {
    // Core functionality
    - test_get_fleet_id_returns_correct_value()
    - test_get_version_version_compatibility_check()
    - test_factory_reset_sends_correct_command()
    - test_reboot_sends_correct_command()
    - test_reload_network_triggers_network_reload()
    - test_load_update_loads_firmware_correctly()
    - test_run_update_executes_update()
    - test_publish_endpoint_registers_with_centrifugo()

    // Error handling
    - test_error_handling_for_unix_socket_failures()
    - test_timeout_handling()
    - test_invalid_json_response_handling()
    - test_device_service_not_running()
}
```

### 4. Authorization Service (77 LOC - UNTESTED)
**Location:** `src/backend/src/services/auth/authorization.rs`
**Why Critical:** Security vulnerability - potential auth bypass

**Must-have tests:**
```rust
#[cfg(test)]
mod tests {
    // Role-based authorization
    - test_validate_token_with_fleet_administrator_role()
    - test_validate_token_with_fleet_operator_role()
    - test_validate_token_with_fleet_observer_role()

    // Security checks
    - test_reject_invalid_tenant_ids()
    - test_reject_invalid_fleet_ids()
    - test_token_verification_with_keycloak_provider()
    - test_handle_expired_tokens()
    - test_reject_tokens_without_required_claims()
    - test_reject_tokens_with_insufficient_permissions()
}
```

### 5. Keycloak Provider (76 LOC - UNTESTED)
**Location:** `src/backend/src/keycloak_client.rs`
**Why Critical:** SSO authentication completely untested

**Must-have tests:**
```rust
#[cfg(test)]
mod tests {
    // Token verification
    - test_verify_token_with_valid_token()
    - test_verify_token_rejects_expired_token()
    - test_verify_token_rejects_invalid_signature()
    - test_verify_token_rejects_wrong_issuer()

    // Key management
    - test_fetch_public_key_retrieves_correct_key()
    - test_public_key_caching()

    // Claims parsing
    - test_parse_token_claims_extracts_correct_data()
    - test_frontend_config_generation()
}
```

### 6. Certificate Service (98 LOC - UNTESTED)
**Location:** `src/backend/src/services/certificate.rs`
**Why Critical:** HTTPS won't work without certificates

**Must-have tests:**
```rust
#[cfg(test)]
mod tests {
    // Core functionality
    - test_create_module_certificate_generates_valid_cert()
    - test_create_module_certificate_creates_private_key()
    - test_iot_edge_workload_api_communication()

    // Error handling
    - test_error_handling_when_workload_api_unavailable()
    - test_certificate_file_permissions_0o600()
    - test_invalid_common_name_rejected()
    - test_certificate_parsing_errors_handled()
}
```

### 7. API Routes Integration Tests
**Location:** `src/backend/src/api.rs`
**Why Critical:** Only 1/17+ routes tested

**Must-have endpoint tests:**
```rust
// CRITICAL - Authentication & Authorization
❌ POST /api/token (login/refresh)
❌ POST /api/password/set
❌ POST /api/password/update
❌ GET /api/password/require-set

// CRITICAL - Device Operations
❌ POST /api/factory-reset
❌ POST /api/reboot
❌ POST /api/reload-network

// CRITICAL - Firmware Management
❌ POST /api/update/file
❌ POST /api/update/load
❌ POST /api/update/run

// CRITICAL - Network Configuration
❌ POST /api/network

// HIGH PRIORITY - Health & Status
❌ GET /api/version
❌ GET /healthcheck

// MEDIUM PRIORITY
❌ GET / (index)
❌ GET /config.js
❌ POST /api/logout

✅ POST /validate (EXISTS - only tested route)
```

**Integration test template:**
```rust
#[actix_web::test]
async fn test_endpoint_happy_path() {
    // Setup mock services
    // Create test app with route
    // Send request with valid auth
    // Assert response status and body
}

#[actix_web::test]
async fn test_endpoint_unauthorized() {
    // Test without auth token
    // Assert 401 Unauthorized
}

#[actix_web::test]
async fn test_endpoint_invalid_input() {
    // Test with invalid payload
    // Assert 400 Bad Request
}

#[actix_web::test]
async fn test_endpoint_service_error() {
    // Mock service to return error
    // Assert 500 Internal Server Error
}
```

---

## 🟠 HIGH PRIORITY Test Cases

### 8. Password Management Endpoints
**Why Important:** Users can't authenticate without this

```rust
// Integration tests for password flow
- test_set_password_initial_password_setup()
- test_update_password_password_change()
- test_require_set_password_status_check()
- test_reject_weak_passwords()
- test_hash_passwords_correctly_before_storage()
- test_rate_limiting_on_password_endpoints()
- test_old_password_verification_on_update()
```

### 9. Middleware Integration
**Location:** `src/backend/src/middleware.rs`
**Why Important:** Routes may be accessible without proper auth

```rust
// Unit tests exist (11 tests), but add integration tests:
- test_protected_routes_reject_unauthenticated_requests()
- test_protected_routes_accept_valid_jwt_tokens()
- test_protected_routes_accept_valid_session_tokens()
- test_protected_routes_accept_valid_basic_auth()
- test_unauthorized_response_format()
- test_cors_headers_on_auth_failure()
- test_auth_with_multiple_concurrent_requests()
```

### 10. Configuration Loading
**Location:** `src/backend/src/config.rs`
**Why Important:** App won't start with invalid config

```rust
#[cfg(test)]
mod tests {
    - test_load_valid_configuration_file()
    - test_handle_missing_configuration()
    - test_validate_tls_certificate_paths()
    - test_validate_required_fields()
    - test_environment_variable_overrides()
    - test_invalid_toml_syntax_rejected()
    - test_default_values_applied()
}
```

---

## 🟡 MEDIUM PRIORITY Test Cases

### 11. Frontend (Vue/TypeScript) - Currently 0 tests
**Location:** `src/ui/src/`

#### Composables Tests (Vitest)
```typescript
// src/ui/src/composables/useCore.ts
describe('useCore', () => {
  - test('initializes Core and loads WASM module')
  - test('dispatches events to Core')
  - test('processes HTTP effects')
  - test('processes WebSocket effects')
  - test('updates viewModel on Core changes')
  - test('handles Core initialization errors')
})

// src/ui/src/composables/useCentrifugo.ts
describe('useCentrifugo', () => {
  - test('establishes WebSocket connection')
  - test('subscribes to channels')
  - test('handles incoming messages')
  - test('reconnects on connection loss')
  - test('cleans up on unmount')
})
```

#### Component Tests (Vue Test Utils)
```typescript
// src/ui/src/components/DeviceInfoCore.vue
describe('DeviceInfoCore', () => {
  - test('renders device info correctly')
  - test('displays online status')
  - test('handles user interactions')
  - test('dispatches correct events on button click')
  - test('displays error states')
  - test('shows loading state during initialization')
})
```

### 12. HTTP Client Unit Tests
**Location:** `src/backend/src/http_client.rs`
**Current:** 2 tests exist for validation only

```rust
#[cfg(test)]
mod tests {
    // Existing: path validation tests

    // Add actual HTTP communication tests:
    - test_get_request_with_query_parameters()
    - test_post_request_with_large_json_payload()
    - test_put_request()
    - test_delete_request()
    - test_timeout_handling()
    - test_connection_refused_handling()
    - test_malformed_response_handling()
}
```

---

## Missing Test Infrastructure

### Currently Not Found:
- ❌ No test fixtures or test utilities
- ❌ No database/service mocking helpers
- ❌ No integration test harness
- ❌ No frontend test framework
- ❌ No CI/CD test configuration (GitHub Actions)
- ❌ No test coverage reporting tools
- ❌ No E2E test framework
- ❌ No performance/load testing

### Build Features:
- `mock` feature exists (`Cargo.toml` line 69) but appears unused in test code
- Uses `mockall_double` for mocking in integration tests
- No consistent mocking strategy across codebase

---

## Recommended Testing Strategy

### Phase 1: Security & Stability (Week 1-2)
**Goal:** Prevent security vulnerabilities and auth bypasses

1. Authorization service tests (#4)
2. Keycloak provider tests (#5)
3. Token/login endpoint tests (#7 - /api/token)
4. Password management tests (#8)

**Deliverable:** All authentication/authorization paths tested

### Phase 2: Core Device Operations (Week 3-4)
**Goal:** Prevent device bricking and operational failures

1. Device service client tests (#3)
2. Network configuration tests (#1)
3. Firmware update tests (#2)
4. Certificate service tests (#6)

**Deliverable:** All critical device operations tested

### Phase 3: API Coverage (Week 5-6)
**Goal:** Comprehensive API testing

1. Remaining API route integration tests (#7)
2. Middleware integration tests (#9)
3. Configuration loading tests (#10)

**Deliverable:** All API endpoints tested with happy path + error cases

### Phase 4: Frontend & E2E (Week 7-8)
**Goal:** End-to-end user flow validation

1. Frontend unit tests (#11)
2. E2E tests for critical flows
3. Performance/load testing

**Deliverable:** Full stack test coverage

---

## Test Infrastructure Setup

### 1. Mocking Helpers
Create reusable mocks in `src/backend/tests/common/mocks.rs`:
```rust
pub mod mocks {
    pub fn mock_device_service_client() -> MockDeviceServiceClient { ... }
    pub fn mock_keycloak_provider() -> MockKeycloakProvider { ... }
    pub fn mock_authorization_service() -> MockAuthorizationService { ... }
}
```

### 2. Test Fixtures
Create fixtures in `src/backend/tests/fixtures/`:
```
fixtures/
├── config/
│   ├── valid_config.toml
│   └── invalid_config.toml
├── tokens/
│   ├── valid_jwt.txt
│   ├── expired_jwt.txt
│   └── invalid_jwt.txt
└── certs/
    ├── test_cert.pem
    └── test_key.pem
```

### 3. Integration Test Utilities
Create test utilities in `src/backend/tests/common/utils.rs`:
```rust
pub async fn create_test_app() -> App { ... }
pub async fn authenticate_test_user() -> String { ... }
pub fn create_mock_request(path: &str, method: Method) -> TestRequest { ... }
```

### 4. CI/CD Integration
Create `.github/workflows/test.yml`:
```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: cargo test --features mock
      - run: cargo test -p omnect-ui-core
      - run: cargo clippy --all-targets
```

### 5. Coverage Reporting
Add to `Cargo.toml`:
```toml
[dev-dependencies]
tarpaulin = "0.27"
```

Run coverage:
```bash
cargo tarpaulin --out Html --output-dir coverage/ --features mock
```

### 6. Frontend Test Setup
Add to `src/ui/package.json`:
```json
{
  "devDependencies": {
    "vitest": "^1.0.0",
    "@vue/test-utils": "^2.4.0",
    "@vitest/ui": "^1.0.0"
  },
  "scripts": {
    "test": "vitest",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest --coverage"
  }
}
```

---

## Files Currently With Tests

### Backend Tests:
- ✅ `src/backend/tests/validate_portal_token.rs` (5 tests)
- ✅ `src/backend/tests/http_client.rs` (3 tests)
- ✅ `src/backend/src/middleware.rs` (11 tests)
- ✅ `src/backend/src/services/auth/token.rs` (3 tests)
- ✅ `src/backend/src/services/auth/password.rs` (2 tests)
- ✅ `src/backend/src/services/firmware.rs` (1 test)
- ✅ `src/backend/src/http_client.rs` (2 tests)

### Files WITHOUT Tests (Critical):
- ❌ `src/backend/src/services/certificate.rs` (98 LOC)
- ❌ `src/backend/src/services/network.rs` (419 LOC)
- ❌ `src/backend/src/services/auth/authorization.rs` (77 LOC)
- ❌ `src/backend/src/omnect_device_service_client.rs` (353 LOC)
- ❌ `src/backend/src/keycloak_client.rs` (76 LOC)
- ❌ `src/backend/src/config.rs` (308 LOC)
- ❌ `src/backend/src/api.rs` (tested indirectly via 1 integration test only)
- ❌ `src/ui/**/*.ts` (entire frontend - 0 tests)
- ❌ `src/ui/**/*.vue` (entire frontend - 0 tests)

---

## Risk Assessment

| Component | Risk Level | Impact if Broken | Current Tests |
|-----------|------------|------------------|---------------|
| Network Config | 🔴 CRITICAL | Device unreachable | 0 |
| Firmware Update | 🔴 CRITICAL | Device bricked | 1 |
| Device Service Client | 🔴 CRITICAL | All operations fail | 0 |
| Authorization | 🔴 CRITICAL | Security breach | 0 |
| Keycloak SSO | 🔴 CRITICAL | Can't login | 0 |
| Certificate Service | 🟠 HIGH | HTTPS broken | 0 |
| Password Management | 🟠 HIGH | Can't authenticate | 0 |
| API Routes | 🟠 HIGH | Unknown failures | 1/17 |
| Middleware | 🟡 MEDIUM | Auth bypass risk | 11 (unit only) |
| Frontend | 🟡 MEDIUM | UX broken | 0 |

---

## Summary & Recommendations

### Current State:
- **27 total tests** covering ~1-2% of codebase
- **Critical gaps** in network config, firmware updates, device service client
- **Security risk** with untested authorization and authentication
- **Zero frontend tests** - entire UI untested

### Immediate Actions Needed:
1. **This Week:** Add authorization and authentication tests (#4, #5)
2. **Next Week:** Add device service client tests (#3)
3. **Within Month:** Add network config and firmware tests (#1, #2)

### Long-term Goals:
- Achieve 80%+ code coverage on critical paths
- Implement CI/CD automated testing
- Add E2E test suite
- Establish test-first development culture

### Estimated Effort:
- **Phase 1 (Security):** 40-60 hours
- **Phase 2 (Core Operations):** 60-80 hours
- **Phase 3 (API Coverage):** 40-50 hours
- **Phase 4 (Frontend):** 50-70 hours
- **Total:** 190-260 hours (6-8 weeks with 1 developer)
