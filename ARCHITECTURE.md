# Architecture

# 1. High-Level Architecture

```
 ┌─────────────┐        HTTPS/JSON        ┌──────────────────┐        SQL         ┌──────────────┐
 │  Frontend    │ ───────────────────────▶ │   Axum API        │ ─────────────────▶ │  PostgreSQL  │
 │  (React)     │ ◀─────────────────────── │    (Rust)         │ ◀───────────────── │              │
 └─────────────┘                          └──────────────────┘                    └──────────────┘
                                                    │
                                                    ▼
                                          ┌────────────────────┐
                                          │ File storage (local)│
                                          └────────────────────┘
```

# 2. Request Flow
1. Client sends HTTP request with `Authorization: Bearer <access_token>` (if authenticated route)
2. Axum router matches request to handler
3. For protected routes, the `AuthenticatedUser` extractor (via `FromRequestParts`) runs first: decodes/validates JWT, checks `token_type == "access"`.
4. Handler performs an explicit role check (Trainee/Trainer/Admin) relevant to the endpoint
5. Handler calls SQLx directly (no repository layer) for the DB operation
6. Response returned as JSON with standard REST status codes

# 3. Code Organization (folder structure)

```
root/
├── README.md
├── ARCHITECTURE.md
├── PRD.md
├── PHASES.md
├── DESIGN.md
├── MEMORY.md
├── docker-compose.yml
├── .gitignore
│
├── backend/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── Dockerfile
│   ├── .env.example
│   ├── .sqlx/                          # SQLx offline query cache
│   ├── migrations/
│   ├── docs/
│   │   └── diagrams/
│   ├── src/
│   │   ├── main.rs                     # route table, server startup, migrations
│   │   ├── middleware.rs               # AuthenticatedUser JWT extractor, role guards
│   │   ├── models.rs                   # request/response structs, Claims
│   │   ├── storage.rs                  # file upload/URL handling (trainer library)
│   │   └── handlers/
│   │       ├── mod.rs
│   │       ├── auth.rs                 # register, login, refresh, logout, profile
│   │       ├── trainee.rs              # enrollment, assessments, feedback, duels
│   │       ├── trainer.rs              # questionnaires, library uploads, monitoring
│   │       ├── admin.rs                # approvals, role mgmt, dashboards, announcements
│   │       └── competency.rs           # tag-based trainer-subject matching
│   └── tests/
│       ├── auth_test.rs
│       ├── trainee_test.rs
│       ├── trainer_test.rs
│       ├── admin_test.rs
│       └── competency_test.rs
│
├── frontend/
│   ├── package.json
│   ├── package-lock.json
│   ├── vite.config.ts                  # (or CRA/Next config, per final framework choice)
│   ├── .env.example
│   ├── index.html
│   ├── public/
│   │   └── assets/
│   └── src/
│       ├── main.tsx
│       ├── App.tsx
│       ├── api/
│       │   ├── client.ts               # base fetch/axios wrapper, auth header injection
│       │   ├── auth.ts
│       │   ├── courses.ts
│       │   ├── assessments.ts
│       │   ├── admin.ts
│       │   └── competency.ts
│       ├── auth/
│       │   ├── AuthContext.tsx
│       │   ├── ProtectedRoute.tsx      # role-aware route guard
│       │   ├── Login.tsx
│       │   └── Register.tsx
│       ├── dashboards/
│       │   ├── trainee/
│       │   ├── trainer/
│       │   └── admin/
│       ├── components/                 # shared UI primitives (buttons, cards, tables, modals)
│       └── styles/
│
└── docs/
```

# 4. Authentication & Authorization

- **Auth:** JWT access token (15 min) + refresh token (7 day, rotated on use), both hashed with SHA-256 before storage in a `sessions` table. Passwords hashed with bcrypt.
- **Authorization:** simple role-check, not a full RBAC join-table chain. `users.role` is a fixed enum (`trainee` | `trainer` | `admin`). Each handler explicitly checks the caller's role against what the endpoint requires — no dynamic permission system, since this project has no multi-tenancy or configurable roles.
- **Approval gate:** new non-admin registrations are created with `approved = false`; login is blocked (or restricted) until an Admin approves them.

# 5. Frontend Architecture
- Single-page React app (or confirmed alternative), role-aware routing: after login, redirect to role-specific dashboard shell
- Three distinct dashboard shells (Trainee/Trainer/Admin), not one generic UI with conditional rendering everywhere — keeps each role's UI focused and demo-friendly
- API client layer talks to the Axum backend via the shared API contract (see `DESIGN.md` for endpoint list)

# 6. Deployment
- **Local dev:** Docker Compose (Postgres + Axum service)
- **Target deployment:** Railway for backend, Netlify for frontend static build — matching existing team deployment pattern
- **Environment variables:** `DATABASE_URL`, `JWT_SECRET`, storage config (local path or bucket credentials)
