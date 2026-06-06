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

## First Account

The first registered account becomes `admin`. Later accounts become regular `user` accounts.

## MVP Limits

The MVP intentionally supports only PDF and EPUB. OPDS, Kobo sync, annotations, OIDC, email sharing, audiobooks, comics, and advanced metadata providers are deferred.
