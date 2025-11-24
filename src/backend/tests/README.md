# Test Infrastructure

This directory contains reusable test infrastructure for the omnect-ui backend.

## Structure

```
tests/
├── common/                  # Reusable test utilities
│   ├── mod.rs              # Module exports
│   ├── mocks.rs            # Mock helpers for services
│   └── utils.rs            # Test utilities (app setup, requests)
├── fixtures/               # Test fixture files
│   ├── config/            # Configuration files
│   ├── tokens/            # JWT tokens
│   ├── certs/             # Certificates and keys
│   └── README.md          # Fixture documentation
├── http_client.rs         # HTTP client integration tests
└── validate_portal_token.rs  # Portal token validation tests
```

## Using Test Infrastructure

### Mock Helpers

The `common::mocks` module provides reusable mock constructors:

```rust
use crate::common::mocks;

// Create mock device service client
let device_client = mocks::mock_device_service_client_with_fleet_id("test-fleet");

// Create mock SSO provider with claims
let claims = mocks::make_token_claims("FleetAdministrator", "test-tenant", None);
let sso_provider = mocks::mock_sso_provider_with_claims(claims);

// Create complete API instance with mocks
let api = mocks::make_api("test-fleet", claims);
```

Available mock helpers:
- `mock_device_service_client_with_fleet_id(fleet_id)` - Success case
- `mock_device_service_client_with_error()` - Error case
- `mock_sso_provider_with_claims(claims)` - Success case
- `mock_sso_provider_with_error(msg)` - Error case
- `make_token_claims(role, tenant, fleets)` - Create TokenClaims
- `make_api(fleet_id, claims)` - Create complete API instance

### Test Utilities

The `common::utils` module provides test utilities:

```rust
use crate::common::utils;

// Create test app
let app = utils::create_test_app(api);

// Create test requests
let req = utils::create_post_request("/api/endpoint", "payload");
let req = utils::create_get_request("/api/endpoint");
let req = utils::create_authenticated_request("/api/endpoint", Method::GET, "token");
let req = utils::create_basic_auth_request("/api/endpoint", Method::POST, "credentials");

// Load fixture files
let token = utils::load_fixture("tokens/valid_jwt.txt");
```

### Fixtures

Load test fixtures using the utility function:

```rust
use crate::common::utils::load_fixture;

let valid_token = load_fixture("tokens/valid_jwt.txt");
let expired_token = load_fixture("tokens/expired_jwt.txt");
let cert = load_fixture("certs/test_cert.pem");
```

See `fixtures/README.md` for more details on available fixtures.

## Running Tests

```bash
# Run all tests with mock feature
cargo test --features mock

# Run specific test file
cargo test --features mock --test validate_portal_token

# Run specific test
cargo test --features mock validate_portal_token_fleet_admin_should_succeed

# Run with output
cargo test --features mock -- --nocapture
```

## Writing New Tests

### Integration Tests

Create a new file in `tests/` directory:

```rust
// tests/my_feature.rs

mod common;  // Import common test infrastructure

use crate::common::{mocks, utils};

#[tokio::test]
async fn test_my_feature() {
    // Use mocks and utilities
    let claims = mocks::make_token_claims("FleetAdministrator", "test-tenant", None);
    let api = mocks::make_api("test-fleet", claims);

    // Create test app and request
    let app = actix_web::test::init_service(utils::create_test_app(api)).await;
    let req = utils::create_get_request("/api/my-feature").to_request();

    // Execute and assert
    let resp = actix_web::test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}
```

### Unit Tests

Add `#[cfg(test)]` module in source files:

```rust
// src/services/my_service.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_my_function() {
        // Test implementation
    }
}
```

## Test Patterns

### Authentication Testing

```rust
// Test with valid token
let req = utils::create_authenticated_request("/api/endpoint", Method::GET, "valid-token");

// Test with basic auth
let req = utils::create_basic_auth_request("/api/endpoint", Method::POST, "dXNlcjpwYXNz");

// Test unauthorized access
let req = utils::create_get_request("/api/endpoint");  // No auth header
let resp = test::call_service(&app, req.to_request()).await;
assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
```

### Mock Configuration

```rust
// Configure mock with specific behavior
let mut mock = DeviceServiceClient::default();
mock.expect_fleet_id()
    .times(1)
    .returning(|| Box::pin(async { Ok("test-fleet".to_string()) }));
```

### Error Testing

```rust
// Test error handling
let device_client = mocks::mock_device_service_client_with_error();
let sso_provider = mocks::mock_sso_provider_with_error("Invalid token");
```

## Best Practices

1. **Use reusable mocks** - Prefer `common::mocks` helpers over creating mocks inline
2. **Use test fixtures** - Load tokens, configs from fixtures instead of hardcoding
3. **Test happy path first** - Verify functionality works before testing edge cases
4. **Test error cases** - Always test failure scenarios
5. **Keep tests isolated** - Each test should be independent
6. **Use descriptive names** - Test names should describe what they test
7. **Follow naming convention** - `test_function_name_scenario_expected_result`

## CI/CD Integration

Tests run automatically in Concourse CI pipeline. Ensure:
- All tests pass: `cargo test --features mock`
- Code is formatted: `cargo fmt`
- Clippy succeeds: `cargo clippy --all-targets --features mock`

## Coverage Goals

Current: 27 tests (~1-2% coverage)

Target coverage by phase:
- Phase 1 (Security): 13% coverage
- Phase 2 (Device Ops): 50% coverage
- Phase 3 (API): 72% coverage
- Phase 4 (Frontend): 85-90% coverage

See TEST_COVERAGE_ANALYSIS.md in repository root for detailed coverage analysis.
