# Development

## Requirements

- Current stable Rust toolchain with Cargo
- Node.js 23 or newer
- Docker Compose or Podman
- PostgreSQL when running outside Compose

## Local Backend

```bash
cp .env.example .env
cargo run -p shelfmark-backend
```

The backend listens on `http://localhost:6060`.

Useful endpoints:

- `GET /health`
- `GET /api/docs`
- `GET /api/openapi.json`

## Local Frontend

```bash
cd frontend
npm install
npm run dev
```

The frontend listens on `http://localhost:5173` and proxies `/api` to the backend.

## Docker Compose

```bash
docker compose up --build
```

Equivalent legacy path:

```bash
docker compose -f deploy/compose/docker-compose.yml up --build
```

## Podman Compose

On macOS, start the Podman VM before using compose:

```bash
podman machine init
podman machine start
```

If the machine already exists, only run:

```bash
podman machine start
```

Then start the project:

```bash
podman compose up --build
```

Equivalent legacy path:

```bash
podman compose -f deploy/compose/docker-compose.yml up --build
```

The root `compose.yaml` is the preferred entrypoint. `deploy/compose/docker-compose.yml` is a standalone compatibility copy for tools that expect the previous path.
Images use the public ECR mirror for Docker Official Images because it works with both Docker and Podman and avoids Docker Hub-specific pull failures.

Mounted folders:

- `storage/data` stores imported files
- `storage/books` is available as `/books` for library folder scanning
- `storage/bookdrop` is reserved for future automatic imports

## Database Queries (sqlx compile-time checking)

Backend SQL is written with the `sqlx::query!` / `sqlx::query_as!` macros, which
verify every query against the real database schema **at compile time**. To make
this work without requiring a live database for every build, the checked queries
are cached in the committed `.sqlx/` directory and used in "offline" mode.

What this means in practice:

- A normal `cargo build` / `cargo check` uses the `.sqlx/` cache — no database needed.
- The Docker build sets `SQLX_OFFLINE=true` and copies `.sqlx/`, so it builds without a database.
- **Whenever you add or change a query macro, you must regenerate the cache and commit it**, or offline builds (including CI and Docker) will fail with "no cached data for this query".

Regenerate the cache against a running, migrated database:

```bash
# 1. Start Postgres (and apply migrations)
docker compose up -d postgres

# 2. Regenerate .sqlx/ (requires the sqlx CLI:
#    cargo install sqlx-cli --no-default-features --features rustls,postgres)
export DATABASE_URL=postgres://library:library@localhost:5432/library
cargo sqlx prepare --workspace

# 3. Commit the updated .sqlx/ directory
```

The dynamic smart-shelf filter in `services/shelves.rs` is built at runtime with
`QueryBuilder` and is intentionally **not** macro-checked.

## First Account

The first registered account becomes `admin`. Later accounts become regular `user` accounts.

## MVP Limits

The MVP intentionally supports only PDF and EPUB. OPDS, Kobo sync, annotations, OIDC, email sharing, audiobooks, comics, and advanced metadata providers are deferred.
