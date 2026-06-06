# API Overview

All application endpoints live under `/api/v1`.

Authentication uses a bearer token returned by `POST /api/v1/auth/login` or `POST /api/v1/auth/register`.

## Main Resources

- `auth`: local registration, login, logout, current user
- `libraries`: folder paths that can be scanned
- `books`: uploaded or scanned PDF/EPUB entries
- `metadata`: Open Library search
- `jobs`: upload and scan job history

OpenAPI is generated at runtime:

```text
http://localhost:6060/api/openapi.json
http://localhost:6060/api/docs
```

