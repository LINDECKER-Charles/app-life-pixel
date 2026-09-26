# syntax=docker/dockerfile:1
# The `admin` image: life-pixel-admin-server and the built console (architecture.md, "Services
# and images"). Built from the root of the repository:
#   docker build -f docker/admin.Dockerfile -t ghcr.io/lindecker-charles/life-pixel/admin:local .
# Every base image is pinned by digest; Dependabot bumps them.

# rust — the release admin server. The whole workspace is copied: Cargo resolves it with
# --locked, even for one package.
FROM rust:1.98-bookworm@sha256:93ce27a88655056a51dbdd8f5f2d7ddc071c7b0070fb288a37b5a285fc83971e AS rust
ARG CARGO_BUILD_JOBS=4
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}
WORKDIR /life-pixel
COPY Cargo.toml Cargo.lock ./
COPY .cargo/config.toml .cargo/
COPY crates crates
COPY xtask xtask
COPY tauri/Cargo.toml tauri/build.rs tauri/
COPY tauri/src tauri/src
COPY player-js player-js
COPY i18n i18n
RUN --mount=type=cache,id=life-pixel-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=life-pixel-cargo-target,target=/life-pixel/target,sharing=locked \
    cargo build --release --locked -p life-pixel-admin-server \
    && mkdir -p /out \
    && cp target/release/life-pixel-admin-server /out/

# web — the console.
FROM node:24.21-bookworm-slim@sha256:0e0ff40c39bc087845bfb27465a0df4ea419520094bc35842ff83dd8cbe6f9b6 AS web
WORKDIR /life-pixel
# The workspace depends on the loader through file:../player-js.
COPY player-js player-js
COPY frontend/package.json frontend/package-lock.json frontend/
RUN npm ci --prefix frontend --no-audit --no-fund
COPY frontend frontend
RUN npm run build:admin --prefix frontend

# runtime — no shell, no package manager, the nonroot user.
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f AS runtime
LABEL org.opencontainers.image.source="https://github.com/LINDECKER-Charles/app-life-pixel" \
      org.opencontainers.image.description="Life Pixel's admin server and console" \
      org.opencontainers.image.licenses="AGPL-3.0-only"
COPY --from=rust /out/life-pixel-admin-server /usr/local/bin/life-pixel-admin-server
COPY --from=web /life-pixel/frontend/dist/admin/browser /srv/admin
COPY i18n /srv/i18n
ENV LPA_HTTP_ADDR=0.0.0.0:8080 \
    LPA_METRICS_ADDR=0.0.0.0:9090 \
    LPA_APP_DIR=/srv/admin \
    LPA_I18N_DIR=/srv/i18n
# nonroot, by number: a runtime checks it is not root without reading the image's /etc/passwd.
USER 65532:65532
WORKDIR /srv
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=5s --start-period=30s --retries=5 \
    CMD ["/usr/local/bin/life-pixel-admin-server", "healthcheck"]
ENTRYPOINT ["/usr/local/bin/life-pixel-admin-server"]
CMD ["serve"]
