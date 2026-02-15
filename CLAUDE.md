# CLAUDE.md

> This file provides guidance to AI coding assistants (GitHub Copilot, Claude Code, etc.) when working with code in this repository.

## Project Overview

Foodie Backend is a modular monolithic Rust application for kitchen stock management. It implements **Domain-Driven Design (DDD)**, **Hexagonal Architecture (Ports & Adapters)**, and follows an **API First** approach using Poem OpenAPI.

## Language and Style

- **All code, comments, and documentation must be written in English**, regardless of the language used in the prompt or communication.
- Even if the user communicates in Spanish, always generate code, comments, and documentation in English.

---

## Core Architecture

The codebase is structured into three main layers following hexagonal architecture:

### 1. Business Layer (`business/`)

Contains all domain logic, entities, and use cases. **No infrastructure or presentation dependencies allowed.**

**Structure per entity:**
```
business/
  src/
    domain/
      <entity>/
        model.rs              # Domain models and aggregates
        value_objects.rs      # Value objects and type-safe wrappers
        errors.rs             # Domain-specific errors with code-style identifiers
        repository.rs         # Repository trait definitions (ports)
        use_cases/            # Use case trait definitions (contracts)
          create.rs
          update.rs
          delete.rs
          get_by_id.rs
          get_all.rs
          ...
    application/
      <entity>/               # Use case implementations (one file per use case)
        create.rs
        update.rs
        delete.rs
        get_by_id.rs
        get_all.rs
        ...
    tests/
      mocks/                  # Mock implementations for unit testing
```

### 2. Infrastructure Layer (`infrastructure/`)

Adapters for external systems. **No business logic permitted.**

```
infrastructure/
  persistence/
    src/
      <entity>/
        entity.rs            # Persistent entity representation
        repository.rs        # Repository implementation
      migrations/            # SQLx database migrations
  logger/                    # Tracing-based logging adapter
```

### 3. Presentation Layer (`presentation/`)

Entry points and adapters for external requests. **No business logic permitted.**

```
presentation/
  rest-api/
    src/
      api/
        <entity>/
          routes.rs          # Poem OpenAPI endpoint definitions (code-first)
          dto.rs             # Manual DTO implementations matching OpenAPI
          responses.rs       # Response helper utilities
          error_mapper.rs    # Domain error to HTTP status mapping
        tags.rs              # OpenAPI documentation tags
        error.rs             # Shared ErrorResponse struct
      config/                # App, CORS, database, server configuration
      setup/                 # Dependency injection and server setup
      main.rs                # Server bootstrap
```

---

## API First Principle

The project uses **Poem OpenAPI** with a **code-first approach**:

1. **Define endpoints** in `presentation/rest-api/src/api/<entity>/routes.rs` using `#[poem_openapi]` macros
2. **Implement DTOs manually** in `presentation/rest-api/src/api/<entity>/dto.rs` to match the OpenAPI contract
3. **OpenAPI spec is generated automatically** from Rust code and served at `/openapi.json`
4. **Swagger UI** available at `http://localhost:8080/docs`

**CRITICAL:** Never implement or change an endpoint without first updating the Rust code that defines the OpenAPI contract in `routes.rs` and `dto.rs`.

---

## Rust Coding Guidelines

Follow the **Microsoft Pragmatic Rust Guidelines** (see `rust-guidelines.txt` if available).

**Key areas:**
- Error handling patterns using `Result<T, E>`
- API design for scalability
- Performance-conscious implementations
- Idiomatic Rust patterns

**Naming conventions:**
- **snake_case** for files, modules, and functions
- **PascalCase** for structs, enums, and traits
- **SCREAMING_SNAKE_CASE** for constants and static variables

---

## Best Practices

### Separation of Concerns
- Keep domain, infrastructure, and presentation logic strictly separated
- **No business logic in infrastructure or presentation**
- **No direct database access in business or presentation**; always use repositories/adapters

### Validation
- Validate all external input in the presentation layer before passing to business logic
- Use strong typing and custom types for validation where possible

### Error Handling
- Use domain-specific errors in business layer
- Map domain errors to appropriate responses in presentation layer
- Always use `Result<T, E>` and idiomatic Rust error handling
- **Never use `unwrap()` or `expect()` in production code**
- Always handle errors explicitly

### Testing
- Write unit tests for business logic
- Write integration tests for adapters
- Use traits for interfaces and mocks for testing
- All tests in `business/src/application/<entity>/` in a dedicated `mod` with `#[cfg(test)]`

### Code Quality
- Use `rustfmt` for formatting and `clippy` for linting
- Follow Rust idioms for naming and module organization
- Use environment variables for configuration; never hardcode secrets

---

## Testing Standards - Test-Driven Development (TDD)

This project follows **strict TDD methodology** where tests validate **business requirements**, not just code correctness.

### TDD Core Principles

**CRITICAL:** Tests are written FIRST, before any implementation code.

**The TDD Cycle (Red-Green-Refactor):**
1. **RED** - Write a failing test that describes a business requirement
2. **GREEN** - Write minimal code to make the test pass
3. **REFACTOR** - Improve code quality without changing behavior

### Test Naming Convention

**Pattern:** `should_<BUSINESS_EXPECTATION>_when_<BUSINESS_SCENARIO>`

**Focus on business behavior, not technical implementation.**

### Test Structure

**AAA (Arrange, Act, Assert) Pattern - With Business Context**

```rust
#[tokio::test]
async fn should_reject_item_when_quantity_is_negative() {
    // Arrange - Set up business scenario
    let mut mock_repo = MockItemRepository::new();
    mock_repo.expect_create().returning(|_| Ok(()));

    let use_case = CreateItemUseCaseImpl {
        repository: Arc::new(mock_repo),
        logger: Arc::new(MockLogger::new()),
    };

    // Act - Execute business operation
    let result = use_case.execute(CreateItemParams {
        name: "Tomatoes".to_string(),
        quantity: -5,
    }).await;

    // Assert - Verify business rule enforcement
    assert!(result.is_err());
    match result.unwrap_err() {
        ItemError::ValidationError(code) => {
            assert_eq!(code, "quantity_negative");
        },
        _ => panic!("Expected ValidationError"),
    }
}
```

### Test Categories (Prioritized)

#### 1. **Business Rule Tests** (CRITICAL Priority)
Validate core business logic and domain invariants.

**When to write:** ALWAYS - These are the foundation of the system.

```rust
#[test]
fn should_reject_item_when_stock_below_minimum() {
    // Business rule: Items cannot go below minimum stock threshold
}
```

#### 2. **Edge Case Tests** (HIGH Priority)
Test boundary conditions and exceptional business scenarios.

**When to write:** For every business rule with boundaries (dates, quantities, status transitions).

```rust
#[tokio::test]
async fn should_reject_stock_update_when_quantity_is_zero() {
    // Edge case: Zero quantity update should be rejected
}
```

#### 3. **Regression Tests** (HIGH Priority)
Prevent bugs from reappearing after fixes.

**When to write:** After every bug fix.

```rust
#[tokio::test]
async fn should_handle_item_names_case_insensitively() {
    // Regression test for bug #42
    // Business rule: Item names are case-insensitive for uniqueness
}
```

#### 4. **Integration Tests** (MEDIUM Priority)
Test interactions between layers and external systems.

**When to write:** For critical external integrations (database, APIs).

```rust
#[tokio::test]
async fn should_persist_stock_movement_in_database() {
    // Business requirement: All stock movements must be auditable
}
```

### Writing Effective Business-Focused Tests

#### DO:

1. **Write tests BEFORE implementation (TDD)**
   ```rust
   // Step 1: Write failing test
   #[test]
   fn should_reject_negative_quantities() { /* ... */ }

   // Step 2: Implement minimal code to pass
   // Step 3: Refactor
   ```

2. **Test business behavior, not implementation**
   ```rust
   // Good: Tests business outcome
   assert!(item.is_below_minimum_stock());

   // Bad: Tests internal state
   assert_eq!(item.status, Status::LowStock);
   ```

3. **Use realistic business scenarios**
   ```rust
   // Good: Realistic kitchen data
   let item = Item::new(ItemProps {
       name: "Extra Virgin Olive Oil".to_string(),
       unit: "liters".to_string(),
       quantity: 5.0,
   });

   // Bad: Unrealistic test data
   let item = Item::new(ItemProps {
       name: "test".to_string(),
       unit: "x".to_string(),
       quantity: 0.0,
   });
   ```

4. **Assert on business outcomes**
   ```rust
   // Good: Asserts business rule enforcement
   assert!(result.is_err());
   assert_eq!(result.error_code(), "quantity_negative");

   // Bad: Asserts technical detail
   assert!(mock_repo.save_was_called());
   ```

5. **Cover happy path AND edge cases**

#### DON'T:

1. **Don't test implementation details** (private methods, framework code)
2. **Don't write tests just for coverage**
3. **Don't use unrealistic test data**
4. **Don't mock everything** - Use real domain objects when possible

### Important Testing Guidelines

- **Always assert domain errors using code-style identifiers** (e.g., `name_empty`, `not_found`), never human-readable messages
- **If a domain model enforces required fields in its constructor**, tests for missing data should expect a thrown error
- **Avoid relying on human-readable error messages** in assertions - use error codes
- **Each test should validate ONE business rule** - Keep tests focused and atomic
- **Tests should be independent** - No execution order dependencies
- **Use descriptive variable names** that reflect business entities (e.g., `low_stock_item`, `expired_ingredient`)
- **Use realistic business data in tests**

---

## Error Handling and Internationalization (i18n)

- All user-facing messages (errors, logs, API responses) must use **code-style identifiers** (e.g., `not_found`, `invalid_input`, `unauthorized`)
- These codes are used by the frontend for internationalization and translation
- **Do not use human-readable messages directly** in code for user-facing responses
- Domain errors are defined in `business/src/domain/<entity>/errors.rs`

---

## User Story Implementation Workflow (TDD)

**CRITICAL:** This workflow follows **Test-Driven Development (TDD)**. Tests are written FIRST, before any implementation code.

When implementing a user story, follow these steps strictly:

### 1. Read and Analyze (Business Requirements First)
- Carefully review the user story and acceptance criteria
- **Identify business rules** - What are the core business requirements?
- **Identify edge cases** - What are the boundary conditions?
- **List test scenarios** - Write down all test cases BEFORE coding
- Identify all required fields, behaviors, and validation rules

### 2. Locate Relevant Information
- Domain models: `business/src/domain/<entity>`
- Use case contracts: `business/src/domain/<entity>/use_cases`
- Use case implementations: `business/src/application/<entity>`
- Domain errors: `business/src/domain/<entity>/errors.rs`
- Domain logger: `business/src/domain/logger.rs`
- Infrastructure logger: `infrastructure/logger`
- OpenAPI spec: `presentation/rest-api/src/api/<entity>/routes.rs`
- DTOs: `presentation/rest-api/src/api/<entity>/dto.rs`
- Repositories: `infrastructure/persistence/src/<entity>`
- Migrations: `infrastructure/persistence/src/migrations`
- Tests: inside each use case in `business/src/application/<entity>` in a dedicated `mod` with `#[cfg(test)]`

### 3. Write Failing Tests FIRST (RED Phase)
**CRITICAL:** Do NOT write any implementation code yet!

- Write tests in `business/src/application/<entity>/` in a dedicated `mod` with `#[cfg(test)]`
- Write ONE test for EACH business rule identified in Step 1
- Write tests for ALL edge cases
- Use realistic business scenarios and data
- Follow naming convention: `should_<BUSINESS_EXPECTATION>_when_<BUSINESS_SCENARIO>`

**Run tests:** `cargo test`
**Expected:** All tests fail (feature not implemented)

### 4. Update OpenAPI Spec (API First - If needed)
- Update `presentation/rest-api/src/api/<entity>/routes.rs` and `dto.rs` to reflect any new or changed endpoints
- Verify the OpenAPI spec is correct
- Never implement or change an endpoint without first updating the OpenAPI contract
- **Note:** This step may come later if implementing internal business logic first

### 5. Implement Domain Models (GREEN Phase - Minimal Code)
- Update or create domain models in `business/src/domain/<entity>/model.rs`
- Add value objects in `business/src/domain/<entity>/value_objects.rs`
- Define domain errors in `business/src/domain/<entity>/errors.rs`
- **Write ONLY enough code to make tests pass** - No more, no less
- Keep business logic in the business layer

**Run tests after each change:** `cargo test`
**Goal:** Tests turn green one by one

### 6. Implement Application Layer (GREEN Phase - Use Cases)
- Define or update use case interfaces in `business/src/domain/<entity>/use_cases`
- Implement use cases in `business/src/application/<entity>`, **always injecting the logger**
- Write minimal code to satisfy test requirements
- Ensure all business logic stays in the business layer

**Run tests:** `cargo test`
**Expected:** All tests pass

### 7. Refactor (REFACTOR Phase)
**CRITICAL:** Tests must keep passing during refactoring!

- Improve code quality without changing behavior
- Extract methods, improve naming, reduce duplication
- Move validation logic to domain layer if needed
- Optimize performance if necessary
- **Run tests after EVERY refactoring change**

### 8. Infrastructure Layer (Implementation)
- Update or create repository adapters in `infrastructure/persistence/src/<entity>`
- Create database migrations if needed using `make sqlx/add-migration NAME=your_migration_name`
- Write integration tests for repository implementations
- Ensure no business logic leaks into infrastructure

**Run tests:** `make test/infrastructure`

### 9. Presentation Layer (API Implementation)
- Implement or update controllers and routes to match the OpenAPI contract
- Validate all input in the presentation layer before passing to business logic
- Map domain errors to appropriate API responses using code-style identifiers
- Write API integration tests

**Run tests:** `make test/rest`

### 10. Documentation & Logging
- Document new features or changes in code
- Add inline comments explaining business rules
- Update README or relevant sub-README if needed
- Use the configured logger for all error/info logging
- Document any new error codes in domain errors file

### 11. Final Check
- Verify ALL acceptance criteria are met
- Verify ALL tests pass: `make test`
- Check code formatting: `make format`
- Run linter: `make lint`
- Update SQLx cache if queries modified: `make sqlx/prepare`
- Verify business rules are validated by tests
- Check that tests are business-focused, not implementation-focused

### TDD Workflow Summary

```
1. READ user story -> Identify business rules & test scenarios
2. LOCATE relevant code -> Understand existing structure
3. RED -> Write failing tests for ALL business rules
4. Run tests -> Verify all tests fail
5. GREEN -> Write minimal code to pass tests
6. Run tests -> Verify tests pass
7. REFACTOR -> Improve code quality
8. Run tests -> Verify tests still pass
9. INFRASTRUCTURE -> Implement persistence layer
10. PRESENTATION -> Implement API layer
11. DOCS -> Document everything
12. FINAL CHECK -> Verify quality standards
```

### Important TDD Reminders

- **NEVER write implementation code before writing tests**
- **Each test should validate ONE business rule**
- **Tests should describe business behavior, not technical implementation**
- **Run tests after EVERY code change**
- **Keep tests independent and isolated**
- **Use realistic business data in tests**
- **Refactor fearlessly - tests protect you**

---

## Common Development Commands

### Environment Setup
```bash
make set-up              # Install rustfmt, clippy, sqlx-cli
cp .env.example .env     # Configure environment variables
```

### Docker & Database Operations
```bash
make docker-compose/up   # Start all services (PostgreSQL)
make docker-compose/down # Stop all services
make db-up               # Start PostgreSQL database only (standalone)
make db-down             # Stop and remove PostgreSQL container
make reset-db-full       # Reset database and reapply migrations
```

### SQLx Operations
```bash
make sqlx/online                        # Switch SQLx to ONLINE mode (check against live DB)
make sqlx/offline                       # Switch SQLx to OFFLINE mode (use cached metadata)
make sqlx/migrate                       # Run database migrations
make sqlx/add-migration NAME=your_name  # Create new migration
make sqlx/prepare                       # Generate SQLx offline metadata cache
make sqlx/check                         # Verify SQLx cache is up to date
```

**Database connection:**
- Database URL: `postgres://devuser:password@localhost:5432/foodiedb`

### Running Applications
```bash
make run/rest-api        # Start REST API server (http://localhost:8080)
```

### Testing and Quality
```bash
make test                    # Run all tests
make test/domain             # Run domain layer tests only
make test/application        # Run application layer tests only
make test/infrastructure     # Run infrastructure layer tests only
make test/rest               # Run REST API tests only
make test-coverage           # Generate test coverage report
```

### Code Quality
```bash
make format                  # Check code formatting
make format/fix              # Auto-fix formatting issues
make lint                    # Run clippy linter
make check                   # Check code compilation
make clean                   # Clean build artifacts
```

### Scaffolding (Code Generation)
```bash
make generate/domain NAME=entity_name         # Generate domain module structure
make generate/application NAME=entity_name    # Generate application service structure
make generate/repository NAME=entity_name     # Generate repository implementation structure
```

---

## Key Technologies

- **Rust** (edition 2024)
- **Poem** - Web framework and OpenAPI
- **SQLx** - Raw SQL database library (compile-time checked queries)
- **PostgreSQL** (14) - Database
- **dotenvy** - Environment configuration
- **tracing** - Structured logging
- **mockall** - Testing mocks
- **Docker** & **Docker Compose** - Containerization

---

## Summary Checklist

When working on this project, ensure you:

- [ ] Write all code, comments, and documentation in English
- [ ] Follow hexagonal architecture: business, infrastructure, presentation
- [ ] Never mix business logic with infrastructure or presentation
- [ ] Update OpenAPI spec before implementing endpoints
- [ ] Use code-style identifiers for all user-facing messages
- [ ] Write tests following the `should_X_when_Y` pattern with AAA structure
- [ ] Never use `unwrap()` or `expect()` in production code
- [ ] Always inject logger into use case implementations
- [ ] Create database migrations for schema changes
- [ ] Run `make sqlx/prepare` after modifying database queries
- [ ] Run `make test`, `make format`, and `make lint` before committing
