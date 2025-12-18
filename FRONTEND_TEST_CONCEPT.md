# Testing Concept: Omnect-UI (Strategy: Core-First)

## Strategy

Leverage the Crux architecture's testability by design. The Core contains all business logic as pure, deterministic functions - making it the highest-ROI test target. The Shell is intentionally thin (renders ViewModel, executes effects) and needs minimal testing.

**Approach:** Test the Core exhaustively (cheap, fast, deterministic), keep E2E minimal for regression safety.

## Implementation Plan

### Phase 1: Core State Transitions (Unit Tests)

*Goal: Secure business logic and state machines with fast, deterministic tests.*

#### PR 1.1: Authentication Tests
- [ ] Test login flow (loading state, success, failure)
- [ ] Test logout and session cleanup
- [ ] Test token management state
- [ ] Test password change flow

#### PR 1.2: Device Tests
- [ ] Test system info updates
- [ ] Test online status transitions
- [ ] Test factory reset state machine
- [ ] Test reboot/reload flows

#### PR 1.3: Network Tests
- [ ] Test network configuration updates
- [ ] Test IP change detection and rollback state
- [ ] Test DHCP/static switching logic
- [ ] Test reconnection state machine

#### PR 1.4: Update Tests
- [ ] Test firmware manifest parsing
- [ ] Test update validation state
- [ ] Test update progress tracking

#### PR 1.5: WebSocket Tests
- [ ] Test Centrifugo connection state
- [ ] Test subscription management
- [ ] Test message routing to model updates

### Phase 2: Core Effect Emissions

*Goal: Verify the Core emits correct effects (HTTP, WebSocket, Render) for each event.*

#### PR 2.1: HTTP Effect Tests
- [ ] Test login emits correct POST request
- [ ] Test authenticated requests include bearer token
- [ ] Test network config changes emit correct payloads
- [ ] Test error responses trigger appropriate state changes

#### PR 2.2: Centrifugo Effect Tests
- [ ] Test connection effects on authentication
- [ ] Test subscription effects for channels
- [ ] Test disconnect effects on logout

### Phase 3: E2E Regression Tests (Selective)

*Goal: Guard critical user journeys against regression. Keep minimal.*

#### PR 3.1: E2E Infrastructure
- [ ] Set up Playwright with minimal config
- [ ] Create test fixtures for mock backend responses
- [ ] Document local test execution

#### PR 3.2: Critical Path Tests
- [ ] Test: Login → View device info → Logout
- [ ] Test: Authentication redirect (unauthenticated access)
- [ ] Test: Network settings change with rollback UI

## Test Patterns

### State Transition Test
```rust
#[test]
fn test_login_sets_loading() {
    let app = AppTester::<App>::default();
    let mut model = Model::default();

    app.update(Event::Login { password: "test".into() }, &mut model);

    assert!(model.is_loading);
    assert!(model.error_message.is_none());
}
```

### Effect Emission Test
```rust
#[test]
fn test_login_emits_http_request() {
    let app = AppTester::<App>::default();
    let mut model = Model::default();

    let effects = app.update(Event::Login { password: "test".into() }, &mut model);

    // Verify HTTP effect with correct endpoint and method
    let http_effects: Vec<_> = effects
        .filter_map(|e| match e {
            Effect::Http(req) => Some(req),
            _ => None,
        })
        .collect();

    assert_eq!(http_effects.len(), 1);
    // Assert on URL, method, headers, body as needed
}
```

### Response Handling Test
```rust
#[test]
fn test_login_success_sets_authenticated() {
    let app = AppTester::<App>::default();
    let mut model = Model::default();

    // Simulate successful login response
    app.update(
        Event::LoginResponse(Ok(LoginResult { token: "abc".into() })),
        &mut model,
    );

    assert!(model.is_authenticated);
    assert!(!model.is_loading);
    assert_eq!(model.token, Some("abc".into()));
}

#[test]
fn test_login_failure_sets_error() {
    let app = AppTester::<App>::default();
    let mut model = Model::default();

    app.update(
        Event::LoginResponse(Err("Invalid credentials".into())),
        &mut model,
    );

    assert!(!model.is_authenticated);
    assert!(!model.is_loading);
    assert_eq!(model.error_message, Some("Invalid credentials".into()));
}
```

## Tools

| Scope | Tool | Purpose |
|:------|:-----|:--------|
| **Core Logic** | `cargo test` + `crux_core::testing` | State transitions, effect emissions |
| **E2E** | Playwright | Critical user journey regression |

## ROI Summary

| Phase | Speed | Stability | Coverage | Priority |
|:------|:------|:----------|:---------|:---------|
| Core State Tests | Fast (ms) | Deterministic | High | **High** |
| Core Effect Tests | Fast (ms) | Deterministic | High | **High** |
| E2E Tests | Slow (s) | Flaky-prone | Low | Low |
