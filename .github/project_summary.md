# Project Summary

This is the repository for the Brand Abuse Detection, Defense, Enforcement, and Remediation (BADDER) tool. BADDER uses vendor provided services in combination with its own heuristics to detect domains abusing our brand and reputation in attempts to defraud our customers. When one is detected, it submits it to a takedown service and tracks the result.

## Technology Stack

- **Language**: Rust
- **Build System**: Cargo
- **Vendor Integrations**: DomainTools, PhishLabs, URLScan
- **Architecture**: Modular API-driven design with configuration-based vendor management

## Key Project Structure

- `src/` - Main application source code
    - `api/` - Vendor API integrations and endpoint handling
        - `domaintools/` - DomainTools API client (reputation, risk scoring, monitoring)
        - `phishlabs/` - PhishLabs takedown service integration
        - `urlscan/` - URLScan API integration for domain scanning
        - `endpoint/` - Generic endpoint infrastructure for API calls and scheduling
    - `domain/` - Core domain models and error handling
    - `persistence.rs` - Data storage and retrieval layer
    - `security_tests.rs` - Security-focused test suite
    - `integration_tests.rs` - Integration testing
    - `test_support/` - Test fixtures and mock server infrastructure

- `config/` - Configuration management
    - `config.schema.json` - Schema for configuration validation
    - `config.json` - Runtime configuration (populated from example)

- `docs/design/` - Aurora model design documentation and operator runbooks

## Key Modules

- **API State Management** (`api/apistate.rs`) - Manages client credentials and API state
- **Domain Models** (`domain/`) - Domain entities (domain seen, status, errors)
- **Authentication/Credentials** (`api/authentication.rs`, `api/credentials.rs`) - Secure credential handling
- **Endpoint Infrastructure** (`api/endpoint/`) - Common patterns for vendor API calls with scheduling support
- **Logging** (`logging.rs`) - Structured logging throughout the application
- **Configuration** (`configuration.rs`) - Dynamic configuration loading and validation

## Development Notes

- The project uses comprehensive testing including unit tests, integration tests, and security-focused tests
- Mock server infrastructure is available in `test_support/` for testing API integrations
- Configuration is externalized and schema-validated for flexibility across environments
- The Aurora compact model is used for architecture documentation under `docs/design/aurora/`
