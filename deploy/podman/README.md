# Podman Setup

This project uses the root `compose.yaml`. Podman can run it through `podman compose`. The compatibility file at `deploy/compose/docker-compose.yml` is also standalone.

## Start Podman

```bash
podman machine start
```

If no machine exists yet:

```bash
podman machine init
podman machine start
```

## Run The App

From the repository root:

```bash
podman compose -f deploy/compose/docker-compose.yml up --build
```

or:

```bash
podman compose up --build
```

Open:

- Frontend: `http://localhost:5173`
- Backend health: `http://localhost:6060/health`
- API docs: `http://localhost:6060/api/docs`

## Stop The App

```bash
podman compose -f deploy/compose/docker-compose.yml down
```

or:

```bash
podman compose down
```

## Storage

The Compose file mounts:

- `storage/data` to `/app/data`
- `storage/books` to `/books`
- `storage/bookdrop` to `/bookdrop`

Use `/books` as the library folder path inside the app when scanning the mounted sample folder.
