# Foodie Backend

> Kitchen stock management backend built with Rust, Hexagonal Architecture, and Poem OpenAPI.

## Architecture

Foodie Backend follows **Hexagonal Architecture** (Ports & Adapters) with three layers:

- **Business** (`business/`) - Domain models, use cases, and business rules. Zero external dependencies.
- **Infrastructure** (`infrastructure/`) - Database persistence (`persistence/`) and logging (`logger/`).
- **Presentation** (`presentation/rest-api/`) - REST API with auto-generated OpenAPI/Swagger docs.

## Quick Start

```bash
# 1. Install tools
make set-up

# 2. Configure environment
cp .env.example .env

# 3. Start database
make docker-compose/up

# 4. Run migrations
make sqlx/migrate

# 5. Start the server
make run/rest-api
```

- Swagger UI: http://localhost:8080/docs
- OpenAPI JSON: http://localhost:8080/openapi.json

## Development

```bash
make test          # Run all tests
make format        # Check formatting
make lint          # Run clippy
make check         # Check compilation
make sqlx/prepare  # Update SQLx offline cache (after query changes)
```

## Project Structure

```
foodie-backend/
  business/                    # Domain logic (models, use cases, errors)
  infrastructure/
    logger/                    # Tracing-based structured logging
    persistence/               # SQLx PostgreSQL repositories + migrations
  presentation/
    rest-api/                  # Poem OpenAPI REST server
  docker-compose.yml           # PostgreSQL service
  Makefile                     # Development commands
```

## Key Technologies

- **Rust** (edition 2024) - Language
- **Poem + Poem OpenAPI** - Web framework with Swagger UI
- **SQLx** - Compile-time checked SQL queries
- **PostgreSQL 14** - Database
- **Docker Compose** - Local development infrastructure
