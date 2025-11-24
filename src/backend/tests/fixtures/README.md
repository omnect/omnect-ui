# Test Fixtures

This directory contains test fixture files used across integration and unit tests.

## Structure

- `config/` - Configuration files for testing
- `tokens/` - JWT tokens for authentication testing
- `certs/` - Certificate and key files for TLS testing

## Usage

Load fixtures in tests using the utility function:

```rust
use crate::common::utils::load_fixture;

let token = load_fixture("tokens/valid_jwt.txt");
```

## Important Notes

- All certificates and keys in this directory are **for testing only**
- JWT tokens are sample tokens with fake signatures
- Configuration files use test values and should not be used in production
