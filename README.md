# ![Project Logo](./frontend/public/ff_banner.png)

**Advanced FitchFork**
_Developed by Team OWCA for COS 301 Capstone Project_

---

## Project Overview

**Advanced FitchFork** is a modular, extensible system for managing programming assignments, grading scripts, code execution, AI-powered diagnostics, and progress tracking. It provides a complete academic toolkit for tutors and administrators, with an intuitive interface and modern role-based access.

This project is developed for the COS 301 Capstone Project at the University of Pretoria in collaboration with the Computer Science Department of University of Pretoria.

**Core Functional Highlights:**

- Authentication with role-based access (Admin, Tutor, Student)
- Module and assignment management
- GATLAM and RNG-based marking scripts
- AI-generated summaries
- Containerized code execution
- Plagiarism detection and statistics
- Gamification and grading system

---

## Key Features

Please refer to our detailed [Functional and Non-Functional Requirements](#functional-and-non-functional-requirements) section below.

---

## Meet the Team

| Member           | Role(s)                             | Skills                                    | GitHub / LinkedIn                                                                                             |
| ---------------- | ----------------------------------- | ----------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| Jacques Klooster | Team Lead, Fullstack Dev            | React, Rust, Tailwind, SQLite, Ant Design | [GitHub](https://github.com/jacqu3sk) [LinkedIn](https://www.linkedin.com/in/jacquesklooster/)                |
| Reece Jordaan    | DevOps, Backend Developer           | Restful APIs, Rust, Testing               | [GitHub](https://github.com/ReeceJordaan) [LinkedIn](https://www.linkedin.com/in/reecejordaan/)               |
| Luke Gouws       | Backend/Frontend                    | Restful APIs, Rust, React                 | [GitHub](https://github.com/CartographySilence) [LinkedIn](https://www.linkedin.com/in/luke-gouws-4b07b7300/) |
| Richard Kruse    | Business Analyst, Backend Developer | Rust, Docker, SQLite                      | [GitHub](https://github.com/RKruse42) [LinkedIn](https://www.linkedin.com/in/richard-kruse/)                  |
| Aidan McKenzie   | Backend/Frontend                    | Rust, Figma                               | [GitHub](https://github.com/RaiderRoss) [LinkedIn](https://www.linkedin.com/in/aidan-mckenzie-772730355/)     |

---

## Tech Stack

| Tool        | Logo                                                  | Used For                                          |
| ----------- | ----------------------------------------------------- | ------------------------------------------------- |
| TypeScript  | ![TypeScript](./frontend/public/stack/typescript.png) | Frontend logic and type safety                    |
| Rust        | ![Rust](./frontend/public/stack/rust.png)             | Backend API and core logic                        |
| TailwindCSS | ![Tailwind](./frontend/public/stack/tailwind.png)     | Styling the UI with utility-first CSS             |
| Ant Design  | ![AntD](./frontend/public/stack/antd.png)             | UI components (tables, buttons, layouts)          |
| Vite        | ![Vite](./frontend/public/stack/vite.png)             | Fast frontend bundler and dev server              |
| SQLite      | ![SQLite](./frontend/public/stack/sqlite.png)         | Lightweight database for storing application data |
| Docker      | ![Docker](./frontend/public/stack/docker.png)         | Containerized code execution and deployment       |
| Jest        | ![Jest](./frontend/public/stack/jest.png)             | Unit testing for frontend logic                   |
| Cypress     | ![Cypress](./frontend/public/stack/cypress.png)       | End-to-end testing of frontend                    |
| React       | ![React](./frontend/public/stack/atom.png)            | Frontend component-based UI framework             |

---

## Project Structure

```bash
advanced-fitch-fork/
├── backend/           # Rust backend (api, db, common, docker examples)
├── frontend/          # React frontend (UI, routing, services)
├── .github/           # GitHub workflows (CI)
├── docs/              # Requirements and documentation
├── README.md          # Project documentation
```

Detailed submodules exist for `auth`, `assignments`, `modules`, `users`, and interpreters.

---

## Getting Started

### Prerequisites

- Node.js (LTS)
- Rust (stable)
- Docker
- cargo-make

### Clone & Setup

```bash
git clone https://github.com/your-org/advanced-fitch-fork.git
cd advanced-fitch-fork
```

#### Frontend

```bash
cd frontend
npm install
npm run dev
```

#### Backend

```bash
cd backend
cp .env.example .env
docker build -t universal-runner .
cargo make fresh
cargo make api
```

> **Note:** This project uses [`cargo-make`](https://sagiegurari.github.io/cargo-make/).  
> Make sure to install it first with:
> ```bash
> cargo install cargo-make
> ```


---

## Running Tests

```bash
# Frontend unit tests
cd frontend
npm test

# Backend unit/integration tests
cd backend
cargo test
```

---

## Git Strategy

- `main`: Production-ready
- `dev`: Latest integration
- `feature/*`: In-progress features

> Pull requests require at least 1 approval.

---

## 💌 Contact

- Email: [owcaheadquarters@protonmail.com](mailto:owcaheadquarters@protonmail.com)
- Course: COS 301, University of Pretoria

---

## Functional and Non-Functional Requirements

Refer to the full specification in the [requirement_specification.pdf](./docs/requirement_specification.pdf) file.

**Highlights:**

- FR1–FR17: Role-based access, module/assignment setup, grammar input, code upload, execution sandbox, plagiarism tools, AI feedback, gamification, stats, security, support.
- NFR1–NFR7: Performance (<3s), scalability, 99.9% uptime, 3-click UX, RBAC, TLS, modular plugin architecture.

> Full requirement list available in `docs/` folder or internal documentation.

---

## Additional Resources

- **[Project Board (GitHub)](https://github.com/orgs/COS301-SE-2025/projects/199)**
  Track issues, iterations, and overall project progress.

- **[Domain Model Diagram](./docs/domain_model.png)**
  Visual design of system architecture and module relationships.

- **[User Stories](./docs/user_stories.pdf)**
  Detailed user-focused functionality and workflow requirements.

- **[Use Cases](./docs/use_cases.pdf)**
  Descriptions of system interactions from the perspective of different roles.

- **[Software Requirements Specification (SRS)](./docs/srs.pdf)**
  Full formal specification of system-level requirements and constraints.

- **[Functional and Non-Functional Requirements Summary](./docs/requirement_specification.pdf)**
  Concise document highlighting core features, performance expectations, and architectural principles.

---

## Viewing Internal Rust Documentation

To browse the backend's internal API documentation (generated with `rustdoc`), follow these steps:

```bash
cd backend
cargo doc --open
```

This will generate and open the documentation in your default web browser. It includes all public modules, functions, and type definitions used throughout the backend.

> Ensure you have Rust installed and configured properly before running the above command.

---

© 2025 Team OWCA. All rights reserved.
