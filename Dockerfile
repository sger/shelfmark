FROM public.ecr.aws/docker/library/node:23-alpine AS frontend
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend ./
RUN npm run build

FROM public.ecr.aws/docker/library/rust:1-bookworm AS backend
ENV CARGO_BUILD_JOBS=1
WORKDIR /app
COPY Cargo.toml ./
COPY backend ./backend
RUN cargo build --release -p shelfmark-backend

FROM public.ecr.aws/docker/library/debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend /app/target/release/shelfmark-backend /app/shelfmark-backend
COPY --from=frontend /app/frontend/dist /app/static
ENV BIND_ADDR=0.0.0.0:6060
EXPOSE 6060
CMD ["/app/shelfmark-backend"]
