# Backend Test Concept: Omnect-UI

## Current State Assessment

### Existing Test Coverage

The backend currently has **60 unit/integration tests** organized as follows:

| Module | Test Count | Coverage |
|:-------|:-----------|:---------|
| `middleware` | 14 | Token validation, session auth, bearer auth, basic auth |
| `services::network` | 14 | Validation, INI generation, rollback response, serialization |
| `services::auth::authorization` | 12 | Role-based access control, tenant/fleet validation |
| `services::firmware` | 7 | Data folder cleanup, load/run update delegation |
| `services::auth::token` | 3 | Token creation/verification |
| `services::auth::password` | 2 | Password hashing/storage |
| `http_client` | 2 | Unix socket client validation |
| Integration (`tests/`) | 6 | HTTP client + portal token validation |

**Total: 60 tests** (52 unit tests, 5 integration tests, 3 http_client integration tests)

### Test Infrastructure

| Component | Status | Notes |
|:----------|:-------|:------|
| Mock Feature Flag | ✅ Configured | `features = ["mock"]` enables `mockall` |
| `mockall` Integration | ✅ Used | Traits annotated with `#[cfg_attr(feature = "mock", automock)]` |
| Actix Test Utilities | ✅ Used | `actix_web::test` for HTTP handler testing |
| Test Config | ✅ Implemented | `AppConfig` uses temp directories in test mode |
| Password File Locking | ✅ Implemented | `PasswordService::lock_for_test()` prevents race conditions |

### Covered Areas

**Well-Tested:**
- Authentication middleware (session, bearer, basic)
- JWT token creation/verification
- Password hashing with Argon2
- Portal token validation with role/fleet authorization
- Unix socket HTTP client

**Partially Tested:**
- Network configuration service (validation, INI generation - rollback logic not tested due to hardcoded paths)

**Not Tested:**
- API handlers (`api.rs`)
- Network rollback logic (hardcoded `/network/` and `/tmp/` paths)
- Certificate service (production path in `certificate.rs`)
- Device service client operations
- Keycloak provider (production path)
- Error handling paths in HTTP client
- Configuration loading edge cases

## Test Strategy

### Principles

1. **Service Layer Focus**: Test business logic in service modules independently of HTTP
2. **Mock External Dependencies**: Use `mockall` for device service, SSO provider, file system
3. **Integration for Critical Paths**: Test complete request/response cycles for key flows
4. **Avoid Flaky Tests**: No real network/socket tests; mock all I/O

### Test Pyramid

```
          /\
         /  \
        / E2E \        ← Covered by frontend E2E tests
       /------\
      /  API   \       ← Integration tests (handlers + middleware)
     /----------\
    /  Services  \     ← Unit tests (business logic)
   /--------------\
  /  Utilities     \   ← Unit tests (helpers, clients)
 /------------------\
```

## Implementation Plan

### Phase 1: Service Layer Unit Tests

*Goal: Test business logic in isolation without HTTP concerns.*

#### PR 1.1: Authorization Service Tests ✅
- [x] Test `validate_token_and_claims` with valid FleetAdministrator
- [x] Test `validate_token_and_claims` with valid FleetOperator + matching fleet
- [x] Test rejection of FleetOperator with non-matching fleet
- [x] Test rejection of invalid tenant
- [x] Test rejection of missing roles
- [x] Test rejection of FleetObserver role
- [x] Test FleetAdministrator with multiple tenants
- [x] Test FleetOperator with multiple fleets
- [x] Test FleetOperator without fleet_list in claims
- [x] Test missing tenant_list in claims
- [x] Test missing roles in claims
- [x] Test invalid SSO token verification

**12 tests added** in [authorization.rs:79-457](src/backend/src/services/auth/authorization.rs#L79-L457)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fleet_admin_with_valid_tenant_succeeds() {
        let mut sso_mock = MockSingleSignOnProvider::new();
        sso_mock.expect_verify_token().returning(|_| {
            Box::pin(async {
                Ok(TokenClaims {
                    roles: Some(vec!["FleetAdministrator".into()]),
                    tenant_list: Some(vec!["cp".into()]),
                    fleet_list: None,
                })
            })
        });

        let mut device_mock = MockDeviceServiceClient::new();

        let result = AuthorizationService::validate_token_and_claims(
            &sso_mock,
            &device_mock,
            "token"
        ).await;

        assert!(result.is_ok());
    }
}
```

#### PR 1.2: Network Configuration Service Tests ✅
- [x] Test validation with valid DHCP config
- [x] Test validation with valid static IP config
- [x] Test validation failure for invalid netmask (> 32)
- [x] Test validation at netmask boundaries (0, 32)
- [x] Test validation failure for empty interface name
- [x] Test INI file generation for DHCP mode
- [x] Test INI file generation for static IP with gateway/DNS
- [x] Test rollback response structure
- [x] Test camelCase serialization/deserialization
- [x] Test enable_rollback field handling

**14 tests added** in [network.rs:503-799](src/backend/src/services/network.rs#L503-L799)

**Note:** Tests for rollback file creation/process/cancellation and actual `set_network_config` integration are not included due to hardcoded filesystem paths (`/network/`, `/tmp/`). These would require refactoring to inject path dependencies or using integration tests with Docker.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn write_network_config_creates_valid_ini_for_dhcp() {
        // Test INI file generation for DHCP mode
    }

    #[test]
    fn write_network_config_creates_valid_ini_for_static() {
        // Test INI file generation for static IP with gateway/DNS
    }

    #[tokio::test]
    async fn set_network_config_validates_netmask_range() {
        // Test validation rejects netmask > 32
    }
}
```

#### PR 1.3: Firmware Service Tests ✅
- [x] Test `clear_data_folder` removes all files
- [x] Test `clear_data_folder` succeeds with empty directory
- [x] Test `clear_data_folder` preserves subdirectories
- [x] Test `load_update` forwards request to device service
- [x] Test `load_update` returns error on device service failure
- [x] Test `run_update` forwards request to device service
- [x] Test `run_update` returns error on device service failure
- [x] Refactored tests to support parallel execution using mutex lock pattern

**7 tests added (6 new)** in [firmware.rs:100-291](src/backend/src/services/firmware.rs#L100-L291)

**Parallel Test Support:** Tests now use static `DATA_FOLDER_LOCK` mutex following the `PASSWORD_FILE_LOCK` pattern, enabling safe parallel execution without race conditions.

**Note:** Tests for `handle_uploaded_firmware` with actual file uploads are not included as they require mocking `TempFile` from actix-multipart, which is complex.

#### PR 1.4: Device Service Client Tests
- [ ] Test URL building (`build_url`)
- [ ] Test version requirement parsing
- [ ] Test version mismatch detection in healthcheck
- [ ] Test `healthcheck_info` response construction

### Phase 2: API Handler Integration Tests

*Goal: Test complete HTTP request/response cycles with mocked dependencies.*

#### PR 2.1: Authentication Endpoints
- [ ] Test `POST /set_password` creates password on first call
- [ ] Test `POST /set_password` redirects if password exists
- [ ] Test `POST /update_password` with correct current password
- [ ] Test `POST /update_password` rejects incorrect current password
- [ ] Test `POST /logout` clears session
- [ ] Test `GET /require_set_password` returns correct boolean

```rust
#[tokio::test]
async fn set_password_creates_password_and_returns_token() {
    let _lock = PasswordService::lock_for_test();
    // Ensure no password file exists

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(token_manager))
            .wrap(session_middleware)
            .route("/set_password", web::post().to(Api::set_password))
    ).await;

    let req = test::TestRequest::post()
        .uri("/set_password")
        .set_json(&SetPasswordPayload { password: "test123".into() })
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(PasswordService::password_exists());
}
```

#### PR 2.2: Device Operation Endpoints
- [ ] Test `GET /healthcheck` with healthy device service
- [ ] Test `GET /healthcheck` with version mismatch
- [ ] Test `GET /healthcheck` when device service unavailable
- [ ] Test `POST /reboot` (mock device service)
- [ ] Test `POST /factory_reset` with valid payload
- [ ] Test `POST /factory_reset` clears session

#### PR 2.3: Network Endpoints
- [ ] Test `POST /network` with valid config
- [ ] Test `POST /network` returns rollback info for IP changes
- [ ] Test `POST /ack_rollback` clears rollback marker
- [ ] Test network config validation errors return 400

#### PR 2.4: Firmware Endpoints
- [ ] Test `POST /upload_firmware_file` with multipart form
- [ ] Test `GET /load_update` (mock device service response)
- [ ] Test `POST /run_update` with validation flag

### Phase 3: Error Handling & Edge Cases

*Goal: Ensure robust error handling and edge case coverage.*

#### PR 3.1: Error Path Testing
- [ ] Test HTTP client error handling (`handle_http_response`)
- [ ] Test service error propagation to HTTP 500
- [ ] Test session insert failure handling
- [ ] Test token creation failure handling

#### PR 3.2: Configuration Edge Cases
- [ ] Test config loading with missing optional env vars
- [ ] Test config loading with invalid port values
- [ ] Test path validation (e.g., missing /data directory)

### Phase 4: Documentation & Maintenance

#### PR 4.1: Test Documentation
- [ ] Add doc comments to test helper functions
- [ ] Document test fixtures and setup requirements
- [ ] Add examples to public API documentation

## Test Patterns

### Mocking Device Service Client

```rust
use mockall_double::double;

#[double]
use crate::omnect_device_service_client::DeviceServiceClient;

fn create_mock_device_service() -> MockDeviceServiceClient {
    let mut mock = MockDeviceServiceClient::new();
    mock.expect_fleet_id()
        .returning(|| Box::pin(async { Ok("test-fleet".into()) }));
    mock.expect_status()
        .returning(|| Box::pin(async { Ok(test_status()) }));
    mock
}
```

### Testing with Actix-Web

```rust
use actix_web::{test, web, App};
use actix_http::StatusCode;

#[tokio::test]
async fn handler_returns_expected_response() {
    let api = make_mock_api();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(api))
            .route("/endpoint", web::get().to(handler))
    ).await;

    let req = test::TestRequest::get()
        .uri("/endpoint")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
```

### Password File Test Isolation

```rust
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn test_requiring_password_file() {
    // Acquire lock to prevent concurrent password file access
    let _lock = PasswordService::lock_for_test();

    // Test logic here - lock automatically released on drop
}
```

### Temp Directory for File Operations

```rust
use tempfile::TempDir;

#[test]
fn test_file_operation() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let file_path = temp_dir.path().join("test.txt");

    // Use file_path for testing
    // Directory automatically cleaned up on drop
}
```

## Test Infrastructure Improvements

### Recommended Additions

1. **Test Fixtures Module**: Create `src/backend/src/test_fixtures.rs` with common test data builders
2. **Response Builders**: Helper functions to create mock HTTP responses
3. **Config Override**: Allow test-specific configuration injection

### Example Test Fixtures

```rust
// src/backend/src/test_fixtures.rs

pub fn test_status() -> Status {
    Status {
        network_status: NetworkStatus {
            network_interfaces: vec![test_network_interface()],
        },
        system_info: SystemInfo {
            fleet_id: Some("test-fleet".into()),
            omnect_device_service_version: "0.40.0".into(),
        },
        update_validation_status: UpdateValidationStatus {
            status: "idle".into(),
        },
    }
}

pub fn test_network_interface() -> NetworkInterface {
    NetworkInterface {
        online: true,
        ipv4: Ipv4Info {
            addrs: vec![Ipv4AddrInfo { addr: "192.168.1.100".into() }],
        },
        file: PathBuf::from("/network/10-eth0.network"),
        name: "eth0".into(),
    }
}

pub fn test_token_claims(role: &str, tenant: &str) -> TokenClaims {
    TokenClaims {
        roles: Some(vec![role.into()]),
        tenant_list: Some(vec![tenant.into()]),
        fleet_list: None,
    }
}
```

## Run Commands

```bash
# Run all backend tests
cargo test --features mock

# Run specific test module
cargo test --features mock middleware::tests

# Run with verbose output
cargo test --features mock -- --nocapture

# Run integration tests only
cargo test --features mock --test '*'

# Check code coverage (requires cargo-tarpaulin)
cargo tarpaulin --features mock --out Html
```

## Coverage Goals

| Module | Current | Target | Priority |
|:-------|:--------|:-------|:---------|
| `middleware` | 80% | 90% | High |
| `services::auth` | 60% | 85% | High |
| `services::network` | 0% | 75% | Medium |
| `services::firmware` | 20% | 70% | Medium |
| `services::certificate` | 0% | 50% | Low |
| `api` | 5% | 70% | High |
| `omnect_device_service_client` | 0% | 60% | Medium |
| `http_client` | 40% | 80% | Medium |
| `config` | 10% | 50% | Low |

## Dependencies

```toml
[dev-dependencies]
actix-http = "3.11"
actix-service = "2.0"
mockall_double = "0.3"
tempfile = "3.20"
```

## Summary

The backend has a solid foundation for testing with:
- Mock infrastructure via `mockall`
- Actix test utilities for HTTP testing
- Test-mode configuration with temp directories

Key gaps to address:
1. **Network service** has no tests despite complex rollback logic
2. **API handlers** are largely untested
3. **Error paths** lack systematic coverage

Priority should be:
1. Add network service tests (high business value, complex logic)
2. Add API handler integration tests (validates request/response contracts)
3. Expand error handling tests (improves reliability)
