# syntax=docker/dockerfile:1
# The `app` image: life-pixel-server and the built app (architecture.md, "Services and images").
# Built from the root of the repository:
#   docker build -f docker/app.Dockerfile -t ghcr.io/lindecker-charles/life-pixel/app:local .
# WASM_BINDGEN_VERSION is the wasm-bindgen of Cargo.lock; _build.yml passes it, and the build
# reads Cargo.lock when it is empty. Every base image is pinned by digest; Dependabot bumps them.

# rust — the editor's engine and the release server.
FROM rust:1.98-bookworm@sha256:93ce27a88655056a51dbdd8f5f2d7ddc071c7b0070fb288a37b5a285fc83971e AS rust
ARG WASM_BINDGEN_VERSION=""
ARG CARGO_BUILD_JOBS=4
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
WORKDIR /life-pixel
COPY Cargo.lock ./
RUN rustup target add wasm32-unknown-unknown \
    && version="${WASM_BINDGEN_VERSION:-$(awk '$0 == "name = \"wasm-bindgen\"" { getline; gsub(/"/, "", $3); print $3; exit }' Cargo.lock)}" \
    && cargo install wasm-bindgen-cli --version "${version:?no wasm-bindgen in Cargo.lock}" --locked \
    && rm -rf /usr/local/cargo/registry
COPY Cargo.toml ./
COPY .cargo/config.toml .cargo/
COPY crates crates
COPY xtask xtask
COPY tauri/Cargo.toml tauri/build.rs tauri/
COPY tauri/src tauri/src
COPY player-js player-js
COPY i18n i18n
RUN --mount=type=cache,id=life-pixel-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=life-pixel-cargo-target,target=/life-pixel/target,sharing=locked \
    cargo xtask build-editor \
    && cargo build --release --locked -p life-pixel-server \
    && mkdir -p /out/documents \
    && cp target/release/life-pixel-server /out/

# web — the app, on the engine the rust stage built.
FROM node:24.21-bookworm-slim@sha256:0e0ff40c39bc087845bfb27465a0df4ea419520094bc35842ff83dd8cbe6f9b6 AS web
WORKDIR /life-pixel
# The app depends on the loader through file:../player-js.
COPY player-js player-js
COPY frontend/package.json frontend/package-lock.json frontend/
RUN npm ci --prefix frontend --no-audit --no-fund
COPY i18n i18n
COPY frontend frontend
COPY --from=rust /life-pixel/frontend/projects/app/src/app/engine/wasm/generated frontend/projects/app/src/app/engine/wasm/generated
COPY --from=rust /life-pixel/frontend/projects/app/public/engine frontend/projects/app/public/engine
ENV LP_ENGINE_PREBUILT=1
RUN npm run build:app --prefix frontend

# runtime — no shell, no package manager, the nonroot user.
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f AS runtime
LABEL org.opencontainers.image.source="https://github.com/LINDECKER-Charles/app-life-pixel" \
      org.opencontainers.image.description="Life Pixel's server and app" \
      org.opencontainers.image.licenses="AGPL-3.0-only"
COPY --from=rust /out/life-pixel-server /usr/local/bin/life-pixel-server
COPY --from=web /life-pixel/frontend/dist/app/browser /srv/app
COPY i18n /srv/i18n
# The folder of LP_STORAGE_URL=file:///var/lib/life-pixel when self-hosting: a volume mounted
# there starts owned by nonroot.
COPY --from=rust --chown=65532:65532 /out/documents /var/lib/life-pixel
ENV LP_HTTP_ADDR=0.0.0.0:8080 \
    LP_METRICS_ADDR=0.0.0.0:9090 \
    LP_ADMIN_API_ADDR=0.0.0.0:9091 \
    LP_APP_DIR=/srv/app \
    LP_I18N_DIR=/srv/i18n
# nonroot, by number: a runtime checks it is not root without reading the image's /etc/passwd.
USER 65532:65532
WORKDIR /srv
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=5s --start-period=30s --retries=5 \
    CMD ["/usr/local/bin/life-pixel-server", "healthcheck"]
ENTRYPOINT ["/usr/local/bin/life-pixel-server"]
CMD ["serve"]
