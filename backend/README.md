# FitchFork Backend

This is the backend monorepo for the Markr project, built in Rust using a modular multi-binary workspace layout. It currently includes:

- `api`: the main HTTP API server (based on Axum)
- `common`: shared code and utilities used across backend binaries

---

## Project Structure

```
backend/
├── api/             # Axum-based HTTP API
├── common/          # Shared modules (config, logging, etc.)
├── data/            # SQLite database file (ignored by git)
├── logs/            # Application logs (ignored by git)
├── .env             # Environment file for the API binary (ignored by git)
├── .env.example     # Copy this to create your own .env
├── Cargo.toml       # Workspace manifest
```

---

## Setup Instructions

### 0. Navigate to the backend folder

Start by changing into the backend directory:

```bash
cd backend
```

Make sure you are in the `backend/` folder when running the following commands.

### 1. Create a `.env` file

The `api` binary requires a `.env` file to be present in the `backend/` directory. To create it:

```bash
cp .env.example .env
```

### 2. Adjust Environment Variables

Edit `.env` to match your local development setup. For example:

```env
PROJECT_NAME=fitch-fork
LOG_LEVEL=debug
LOG_FILE=logs/api-server.log
DATABASE_URL=data/dev.db
HOST=127.0.0.1
PORT=3000
```

Ensure the `logs/` and `data/` folders exist or are created at runtime.

---

## Running the API Server

Make sure you are in the `backend/` directory, then run:

```bash
cargo run -p api
```

This will start the Axum server and create the SQLite database if it doesn't exist. Logs will be written to both the terminal and the file defined in `.env`.

---

## Running Tests

The API uses `rtest` for endpoint testing. To run tests from the `backend/` folder:

```bash
cargo test -p api
```

---
