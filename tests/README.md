# Fitchfork Testing Suite

This directory contains all end-to-end (E2E) and load testing scripts for the Fitchfork platform. It uses **Cypress** for UI and API tests and **k6** for performance testing.

---

## Directory Structure

```
tests/
├── cypress/                 # Cypress E2E tests
│   ├── e2e/                 # Test specs organized by user role
│   ├── fixtures/            # Static data and file uploads
│   ├── logs/                # Optional log output
│   ├── screenshots/         # Failure screenshots
│   └── support/             # Custom commands and utils
│       ├── commands/        # API and UI command modules
│       └── utils/           # Shared utilities like token management
├── k6/                      # Load test setup using k6
│   ├── scenarios/           # Phase-based scripts for full flows
│   ├── shared/              # Auth, config, and HTTP helpers
│   └── test_files/          # ZIP files used for submission tests
├── package.json             # Test scripts and dependencies
├── jsconfig.json            # IntelliSense support
├── cypress.config.js        # Cypress configuration
├── webpack.config.js        # Custom Webpack setup
└── README.md
```

---

## NPM Scripts

You can run all test workflows via defined NPM scripts:

- `npm run e2e`
  Starts the backend and frontend, waits for both to be ready, and runs Cypress headless.

- `npm run e2e:open`
  Opens the Cypress GUI runner.

- `npm run e2e:headless`
  Runs all Cypress tests headlessly.

- `npm run k6:submission`
  Executes the full k6 load testing scenario for submissions.

---

## Prerequisites

- [Node.js](https://nodejs.org/) and `npm`
- `cargo` and Rust environment for backend
- All backend and frontend dependencies installed (`cd ../backend && cargo build`, `cd ../frontend && npm install`)
- k6 installed globally (for running load tests):

  ```
  brew install k6  # macOS
  choco install k6 # Windows
  ```
