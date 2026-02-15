# Foodie - Infrastructure Makefile
#
# Main targets:
# - db-up: Start PostgreSQL database in Docker
# - db-down: Stop and remove PostgreSQL container
# - docker-compose/up: Start all containers
# - docker-compose/down: Stop all containers
# - help: Show available targets
# Variables
WORKSPACE_ROOT := $(shell pwd)
CARGO := cargo
DOCKER := docker
DOCKER_COMPOSE := docker compose

# SQLx offline/online mode
SQLX_ONLINE_ENV := SQLX_OFFLINE=false
SQLX_OFFLINE_ENV := SQLX_OFFLINE=true

# Colors for output
GREEN := \033[1;32m
YELLOW := \033[1;33m
BLUE := \033[1;34m
CYAN := \033[1;36m
MAGENTA := \033[1;35m
RED := \033[1;31m
NC := \033[0m # No Color

# Dev Setup
set-up:
	@echo "${CYAN}Setting up development environment...${NC}"
	rustup component add rustfmt clippy
	cargo install sqlx-cli

# Database
db-up:
	@echo "${BLUE}Starting PostgreSQL database in Docker...${NC}"
	$(DOCKER) run --name foodie-postgres -e POSTGRES_PASSWORD=postgres \
		-e POSTGRES_USER=postgres -e POSTGRES_DB=foodie \
		-p 5435:5432 -d postgres:14

db-down:
	@echo "${BLUE}Stopping and removing PostgreSQL container...${NC}"
	$(DOCKER) stop foodie-postgres || true
	$(DOCKER) rm foodie-postgres || true
	$(DOCKER) volume rm foodie-data || true

sqlx/online:
	@echo "${YELLOW}Changing SQLx to Online mode...${NC}"
	@echo 'SQLX_OFFLINE = "false"' > .cargo/config.toml
	@echo "${GREEN}SQLx is now in ONLINE mode. Queries will be checked against a real database.${NC}"

sqlx/offline:
	@echo "${YELLOW}Changing SQLx to Offline mode...${NC}"
	@echo '[env]\nSQLX_OFFLINE = "true"' > .cargo/config.toml
	@echo "${GREEN}SQLx is now in OFFLINE mode. Queries will be checked against the saved metadata.${NC}"

sqlx/prepare: docker-compose/up
	@echo "${GREEN}Preparing SQLx cache...${NC}"
	cd $(WORKSPACE_ROOT) && $(SQLX_ONLINE_ENV) $(CARGO) sqlx prepare --workspace --database-url postgres://devuser:password@localhost:5435/foodiedb

sqlx/check: docker-compose/up
	@echo "${GREEN}Checking SQLx cache...${NC}"
	cd $(WORKSPACE_ROOT) && $(SQLX_ONLINE_ENV) $(CARGO) sqlx prepare --workspace --check --database-url postgres://devuser:password@localhost:5435/foodiedb

sqlx/migrate: docker-compose/up
	@echo "${GREEN}Running database migrations...${NC}"
	cd $(WORKSPACE_ROOT) && $(SQLX_ONLINE_ENV) $(CARGO) sqlx migrate run --source infrastructure/persistence/src/migrations --database-url postgres://devuser:password@localhost:5435/foodiedb

sqlx/add-migration:
	@echo "${GREEN}Adding new SQLx migration...${NC}"
	@if [ -z "$$NAME" ]; then \
		echo "Error: Please provide a migration name using 'make sqlx/add-migration NAME=your_migration_name'"; \
		exit 1; \
	fi
	sqlx migrate add $$NAME --source infrastructure/persistence/src/migrations

reset-db-full:
	@echo "${RED}╔════════════════════════════════════════════════╗${NC}"
	@echo "${RED}║     ⚠️  DESTRUCTIVE OPERATION WARNING ⚠️      ║${NC}"
	@echo "${RED}╚════════════════════════════════════════════════╝${NC}"
	@echo ""
	@echo "${YELLOW}This will:${NC}"
	@echo "${RED}  • Stop and remove the database container${NC}"
	@echo "${RED}  • Delete ALL data${NC}"
	@echo "${RED}  • Recreate database from scratch${NC}"
	@echo ""
	@echo "${YELLOW}Type 'DESTROY' to confirm (case-sensitive):${NC}"
	@read -p "➤ " CONFIRM; \
	if [ "$$CONFIRM" != "DESTROY" ]; then \
		echo "${GREEN}❌ Operation cancelled${NC}"; \
		exit 1; \
	fi
	@echo ""
	@echo "${CYAN}Proceeding with database reset...${NC}"
	@$(MAKE) db-down
	@$(MAKE) docker-compose/up
	@$(MAKE) sqlx/migrate
	@echo ""
	@echo "${GREEN}Database has been reset and migrations applied.${NC}"

# Docker Compose

docker-compose/up:
	@echo "${MAGENTA}Starting containers with docker-compose...${NC}"
	$(DOCKER_COMPOSE) up -d

docker-compose/down:
	@echo "${MAGENTA}Stopping containers with docker-compose...${NC}"
	$(DOCKER_COMPOSE) down

run/rest-api:
	@echo "${CYAN}Running REST API server...${NC}"
	cargo run --manifest-path=presentation/rest-api/Cargo.toml

# Test
test:
	@echo "${YELLOW}Running tests...${NC}"
	cargo test --all --workspace

test/domain:
	@echo "${YELLOW}Running domain business/src/domain tests...${NC}"
	cargo test -p business domain

test/application:
	@echo "${YELLOW}Running application tests...${NC}"
	cargo test -p business application

test/infrastructure:
	@echo "${YELLOW}Running infrastructure tests...${NC}"
	cargo test -p logger
	cargo test -p persistence

test-coverage:
	@echo "${GREEN}Generating test coverage report...${NC}"
	cargo install cargo-tarpaulin --locked || true
	cargo tarpaulin --out Xml --output-dir ./coverage

test/rest:
	@echo "${YELLOW}Running REST API tests...${NC}"
	cargo test -p rest-api

# Docs
docs:
	@echo "${CYAN}Generating documentation...${NC}"
	cargo doc --no-deps --workspace
	@echo "${CYAN}Opening documentation in browser...${NC}"
	xdg-open target/doc/business/index.html || open target/doc/business/index.html

# Benchmark
benchmark:
	@echo "${MAGENTA}Running benchmarks...${NC}"
	cargo bench --all --workspace

# Clean, Linting, Formatting
format:
	@echo "${CYAN}Checking code formatting...${NC}"
	cargo fmt --all -- --check

format/fix:
	@echo "${CYAN}Fixing code format...${NC}"
	cargo fmt --all

lint:
	@echo "${CYAN}Linting code...${NC}"
	cargo clippy --all --workspace -- -D warnings

clean:
	@echo "${RED}Cleaning project...${NC}"
	cargo clean

check:
	@echo "${CYAN}Checking code...${NC}"
	cargo check --all --workspace

# Scaffolding
generate/domain:
 # create a new domain module inside of business/src/domain
	@echo "${GREEN}Creating new domain module...${NC}"
	@if [ -z "$$NAME" ]; then \
		echo "Error: Please provide a module name using 'make generate/domain NAME=your_module_name'"; \
		exit 1; \
	fi
	mkdir -p business/src/domain/$$NAME
	touch business/src/domain/$$NAME/errors.rs
	touch business/src/domain/$$NAME/model.rs
	touch business/src/domain/$$NAME/repository.rs
	touch business/src/domain/$$NAME/value_objects.rs
	mkdir -p business/src/domain/$$NAME/use_cases
	touch business/src/domain/$$NAME/use_cases/create.rs
	touch business/src/domain/$$NAME/use_cases/delete.rs
	touch business/src/domain/$$NAME/use_cases/update.rs
	touch business/src/domain/$$NAME/use_cases/get_by_id.rs
	touch business/src/domain/$$NAME/use_cases/get_all.rs
	@echo "${GREEN}Domain module '$$NAME' created successfully.${NC}"
	@echo "${YELLOW}Remember to register the new module in domain/src/lib.rs and update application and infrastructure modules to use it.${NC}"

generate/application:
 # create a new application service inside of business/src/application
	@echo "${GREEN}Creating new application service...${NC}"
	@if [ -z "$$NAME" ]; then \
		echo "Error: Please provide a service name using 'make generate/application NAME=your_service_name'"; \
		exit 1; \
	fi
	mkdir -p business/src/application/$$NAME
	touch business/src/application/$$NAME/create.rs
	touch business/src/application/$$NAME/delete.rs
	touch business/src/application/$$NAME/update.rs
	touch business/src/application/$$NAME/get_by_id.rs
	touch business/src/application/$$NAME/get_all.rs
	@echo "${GREEN}Application service '$$NAME' created successfully.${NC}"
	@echo "${YELLOW}Remember to register the new service in application/src/lib.rs and update the domain module to use it.${NC}"

generate/repository:
 # create a new repository implementation inside of infrastructure/persistence/src/repositories
	@echo "${GREEN}Creating new repository implementation...${NC}"
	@if [ -z "$$NAME" ]; then \
		echo "Error: Please provide a repository name using 'make generate/repository NAME=your_repository_name'"; \
		exit 1; \
	fi
	mkdir -p infrastructure/persistence/src/$$NAME
	touch infrastructure/persistence/src/$$NAME/entity.rs
	touch infrastructure/persistence/src/$$NAME/repository.rs
	@echo "${GREEN}Repository implementation '$$NAME' created successfully.${NC}"
	@echo "${YELLOW}Remember to register the new repository in persistence/src/lib.rs and update the domain module to use it.${NC}"

update-guidelines:
	@echo "${CYAN}Updating Rust guidelines...${NC}"
	./update-rust-guidelines.sh

rust-review:
	@echo "${CYAN}Reviewing Rust code against guidelines...${NC}"
	./rust-review.sh

help:
	@echo "${GREEN}Available targets:${NC}"
	@echo "${YELLOW}  help                 ${NC}${GREEN}- Show this help message ${NC}"
	@echo "${YELLOW}  set-up               ${NC}${BLUE}- Set up development environment"
	@echo "${YELLOW}  db-up                ${NC}${BLUE}- Start PostgreSQL database in Docker"
	@echo "${YELLOW}  db-down              ${NC}${BLUE}- Stop and remove PostgreSQL container"
	@echo "${YELLOW}  reset-db-full        ${NC}${RED}- Reset database (destructive)"
	@echo "${YELLOW}  docker-compose/up    ${NC}${MAGENTA}- Start all containers"
	@echo "${YELLOW}  docker-compose/down  ${NC}${MAGENTA}- Stop all containers"
	@echo "${YELLOW}  run/rest-api         ${NC}${CYAN}- Run the REST API server"
	@echo "${YELLOW}  generate/domain      ${NC}${GREEN}- Generate scaffolding for a new domain module (use NAME=your_module_name)"
	@echo "${YELLOW}  generate/application ${NC}${GREEN}- Generate scaffolding for a new application service (use NAME=your_service_name)"
	@echo "${YELLOW}  generate/repository  ${NC}${GREEN}- Generate scaffolding for a new repository implementation (use NAME=your_repository_name)"
	@echo "${YELLOW}  sqlx/online          ${NC}${GREEN}- Switch SQLx to ONLINE mode"
	@echo "${YELLOW}  sqlx/offline         ${NC}${GREEN}- Switch SQLx to OFFLINE mode"
	@echo "${YELLOW}  sqlx/prepare         ${NC}${GREEN}- Prepare SQLx cache"
	@echo "${YELLOW}  sqlx/check           ${NC}${GREEN}- Check SQLx cache"
	@echo "${YELLOW}  sqlx/migrate         ${NC}${GREEN}- Run SQLx migrations"
	@echo "${YELLOW}  sqlx/add-migration   ${NC}${GREEN}- Add a new SQLx migration (use NAME=your_migration_name)"
	@echo "${YELLOW}  test                 ${NC}${YELLOW}- Run all tests"
	@echo "${YELLOW}  test/domain          ${NC}${YELLOW}- Run domain business/src/domain tests"
	@echo "${YELLOW}  test/application     ${NC}${YELLOW}- Run application tests"
	@echo "${YELLOW}  test/infrastructure  ${NC}${YELLOW}- Run infrastructure tests"
	@echo "${YELLOW}  test/rest            ${NC}${YELLOW}- Run REST API tests"
	@echo "${YELLOW}  test-coverage        ${NC}${GREEN}- Generate test coverage report"
	@echo "${YELLOW}  docs                 ${NC}${CYAN}- Generate and open documentation"
	@echo "${YELLOW}  benchmark            ${NC}${MAGENTA}- Run benchmarks"
	@echo "${YELLOW}  format               ${NC}${CYAN}- Check code formatting"
	@echo "${YELLOW}  format/fix           ${NC}${CYAN}- Fix code formatting"
	@echo "${YELLOW}  lint                 ${NC}${CYAN}- Lint code"
	@echo "${YELLOW}  check                ${NC}${CYAN}- Check code"
	@echo "${YELLOW}  clean                ${NC}${RED}- Clean project"
	@echo "${YELLOW}  update-guidelines    ${NC}${CYAN}- Update Rust development guidelines"
	@echo "${YELLOW}  rust-review          ${NC}${CYAN}- Review Rust code against guidelines${NC}"
