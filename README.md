# Shelfmark

A self-hostable personal e-book library. Upload, organize, and read your PDF and
EPUB books from the browser, with per-user reading progress, smart shelves,
tagging, and metadata lookup via Open Library.

## Features

- 📚 Upload and import PDF / EPUB books, or scan a library folder
- 📖 Built-in in-browser readers for PDF and EPUB
- 📊 Per-user reading progress and status (unread / reading / finished)
- 🗂️ Smart shelves — dynamic collections driven by rules (author, language, year, tag, status, dates, progress, …)
- 🏷️ Tagging and favorites
- 🔍 External metadata search via Open Library
- 🔐 JWT authentication with Argon2 password hashing (first account becomes `admin`)
- 📜 OpenAPI docs with Swagger UI

## Tech Stack

- **Backend:** Rust, [Axum](https://github.com/tokio-rs/axum), [SQLx](https://github.com/launchbadge/sqlx) (compile-time-checked queries), PostgreSQL, JWT + Argon2, Utoipa/Swagger
- **Frontend:** React 19 + TypeScript, TanStack Router/Query/Table, Radix UI, Tailwind CSS v4, Vite
- **Infra:** PostgreSQL 17, Docker / Podman (multi-stage build)

## Quick Start (Docker)

```bash
cp .env.example .env
# Set a strong secret — the backend refuses to start without one:
#   openssl rand -hex 32   →  put it in .env as JWT_SECRET=...
docker compose up --build
```

Then open:

- Frontend (the app): http://localhost:5173
- Backend health: http://localhost:6060/health
- API docs (Swagger): http://localhost:6060/api/docs

The **first account you register becomes the admin.**

> Podman works too — substitute `podman compose`. See [deploy/podman/README.md](deploy/podman/README.md).

## Local Development

**Backend** (requires a running PostgreSQL — e.g. `docker compose up -d postgres`):

```bash
cp .env.example .env   # set JWT_SECRET and DATABASE_URL
cargo run -p shelfmark-backend   # http://localhost:6060
```

**Frontend:**

```bash
cd frontend
npm install
npm run dev   # http://localhost:5173, proxies /api to the backend
```

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for more, including the sqlx
compile-time query workflow.

## Configuration

Environment variables (see [.env.example](.env.example)):

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | yes | — | PostgreSQL connection string |
| `JWT_SECRET` | yes | — | Signing secret; must not be empty or a placeholder. Generate with `openssl rand -hex 32` |
| `STORAGE_DIR` | no | `./storage/data` | Where uploaded book files are stored |
| `ALLOWED_ORIGINS` | no | `http://localhost:5173` | CORS allowed origin(s) |
| `BIND_ADDR` | no | `0.0.0.0:6060` | Backend listen address |

Persistent data lives under `storage/` (Postgres data, uploaded books, scan
library, drop folder) — all git-ignored.

## Project Structure

```
backend/          Rust + Axum API
  src/
    routes/       HTTP handlers (auth, books, libraries, shelves, metadata, jobs)
    services/     business logic (auth, books, shelves, metadata)
    models/       data types
    db/           connection + migrations runner
  migrations/     PostgreSQL schema migrations
frontend/         React + TypeScript SPA
  src/pages/      route pages (Books, Reader, Shelves, Libraries, ...)
  src/components/ UI components incl. PDF/EPUB readers
  src/api/        API client + types
.sqlx/            committed sqlx offline query cache (do not hand-edit)
deploy/           Podman / compose deployment helpers
docs/             development and API notes
```

## Database & Queries

SQL is written with sqlx's `query!` / `query_as!` macros, validated against the
real schema at compile time. The checked queries are cached in the committed
`.sqlx/` directory so builds (and Docker/CI) don't need a live database. **After
changing any query, regenerate the cache** — see
[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md#database-queries-sqlx-compile-time-checking).

## Testing

```bash
# Backend
cargo test -p shelfmark-backend

# Frontend
cd frontend
npm run test    # Vitest unit tests
npm run e2e     # Playwright end-to-end tests
```

## MVP Scope

Shelfmark intentionally supports only PDF and EPUB for now. OPDS, Kobo sync,
annotations, OIDC, email sharing, audiobooks, comics, and advanced metadata
providers are deferred.
