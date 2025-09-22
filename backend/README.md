# FitchFork Backend

This is the backend monorepo for the Markr project, built in Rust using a modular multi-binary workspace layout. It currently includes (non‑exhaustive):

- `api`: the main HTTP API server (Axum)
- `common`: shared code and utilities
- `db`: data layer models and helpers
- `code_manager`: containerized code execution service
- `code_runner`: runner/validation helpers
- `marker`: grading/feedback logic
- `migration`: database migrations
- `seeder`: seed data
- `util`: cross‑cutting utilities
- `ai`, `moss_parser`: analysis/parsing helpers

---

## Project Structure

```
backend/
├── api/               # Axum-based HTTP API
├── common/            # Shared modules (config, logging, etc.)
├── code_manager/      # Code execution microservice
├── code_runner/       # Runner helpers
├── db/                # Data models
├── marker/            # Grading/feedback
├── migration/         # Migrations
├── moss_parser/       # Parsers
├── seeder/            # Seed data
├── util/              # Shared utilities
├── logs/              # Application logs (ignored by git)
├── .env               # Environment file (ignored by git)
├── .env.example       # Template to copy for local env
├── Cargo.toml         # Workspace manifest
├── Makefile.toml      # Task shortcuts (if used)
```

---

## Prerequisites

- Rust (stable toolchain)
- SQLite (bundled by `rusqlite`/`sea-orm` backends as needed; no separate install typically required)

---

## Setup Instructions

### 0) Navigate to the backend folder

```bash
cd backend
```

Make sure you are in the `backend/` folder when running the following commands.

### 1) Create a `.env` file

Copy the example:

```bash
cp .env.example .env
```

### 2) Adjust Environment Variables

Edit `.env` to match your local development setup. **Current fields** (example):

```env
# ┌──────────────────────────────┐
# │    Application Environment   │
# └──────────────────────────────┘

# Sets the app environment: affects logging, error output, etc.
# Valid values: development, production, test
APP_ENV=development

# Name of the project (used in startup banner and log entries)
PROJECT_NAME=fitch-fork

# ┌──────────────────────────────┐
# │      Logging Configuration   │
# └──────────────────────────────┘

# Set the minimum log level per crate (e.g., `api=info`, `debug`, `error`)
# Format follows `tracing_subscriber::EnvFilter` syntax
LOG_LEVEL=api=info

# Log file name (stored in the `logs/` directory, auto-created if missing)
LOG_FILE=api.log

# Whether to also print logs to terminal (true/false)
LOG_TO_STDOUT=false

# ┌──────────────────────────────┐
# │   Database & File Storage    │
# └──────────────────────────────┘

# Path to SQLite database file
# Use a path in the user's home directory or temporary directory to avoid permission issues
DATABASE_PATH=$HOME/fitchfork/db/dev.db

# One root for all app storage; subfolders will be created under here
STORAGE_ROOT=$HOME/fitchfork/storage


# ┌──────────────────────────────┐
# │   Server Network Settings    │
# └──────────────────────────────┘

# Host and port the server will bind to
HOST=127.0.0.1
PORT=3000

# Host and port for the code manager
CODE_MANAGER_HOST=127.0.0.1
CODE_MANAGER_PORT=5000

# ┌──────────────────────────────┐
# │     Container Settings       │
# └──────────────────────────────┘

# Number of containers that may run at once
MAX_NUM_CONTAINERS=10
SYSTEM_HEALTH_BROADCAST_MS=2000
# Interval in seconds for persisting system health metrics
SYSTEM_HEALTH_PERSIST_SECONDS=60

# ┌──────────────────────────────┐
# │   Authentication Settings    │
# └──────────────────────────────┘

# JWT signing secret key (keep this secure!)
JWT_SECRET=super_secret_key

# Token validity duration in minutes
JWT_DURATION_MINUTES=60

# Password reset token validity duration in minutes
RESET_TOKEN_EXPIRY_MINUTES=15

# Maximum password reset requests per hour
MAX_PASSWORD_RESET_REQUESTS_PER_HOUR=3

# Password reset mail settings
GMAIL_USERNAME=tuksfitchfork@gmail.com
GMAIL_APP_PASSWORD=your_16_character_app_password
FRONTEND_URL=https://fitchfork.co.za
EMAIL_FROM_NAME=FitchFork

# Gemini API key for AI feedback
GEMINI_API_KEY=gemini_api_key_here

# ID used for moss requests
MOSS_USER_ID=000000000
```

> **Note:** Some dotenv loaders do **not** expand `$HOME`. If paths don’t resolve at runtime, replace `$HOME` with your absolute home directory path.

Ensure the `logs/` directory exists; it is created on demand if missing.

---

## Running the API Server

From the `backend/` directory:

```bash
cargo run -p api
```

This starts the Axum server and creates the SQLite database if it doesn't exist. Logs are written to the terminal (if enabled) and to the file defined in `.env`.

### Running the Code Manager

```bash
cargo run -p code_manager
```

The service binds to `CODE_MANAGER_HOST:CODE_MANAGER_PORT` from your `.env`.

---

## Code Formatting & Linting

### Fix formatting locally (auto‑apply)

```bash
cargo fmt --all
```

### Check formatting only (CI‑friendly)

```bash
cargo fmt --all -- --check
```

### Run Clippy lints

```bash
# Install once if needed:
rustup component add clippy

# Lint the whole workspace (treat warnings as errors)
cargo clippy --all-targets --all-features -- -D warnings
```

---

## Running Tests

### Standard tests

```bash
cargo test
```

### Workspace tests (all crates)

```bash
cargo test --workspace
```

### With `nextest` (if installed, as used in CI)

```bash
cargo nextest run --workspace --release --no-capture
```

---

## Notes

- Do **not** commit your `.env` file. Use `.env.example` to share defaults.
- Keep secrets (e.g., `JWT_SECRET`, `GMAIL_APP_PASSWORD`, `GEMINI_API_KEY`) out of commits and CI logs.
- The repository contains multiple crates; run Cargo commands from `backend/` so they operate on the workspace root.
