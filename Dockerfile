FROM lukemathwalker/cargo-chef:0.1.77-rust-alpine AS cargo-chef-base
FROM node:22-alpine AS node-base
FROM alpine:3.23 AS base

# ==================
# ===== libs =======
# ==================
FROM base AS libs
    # TARGETARCH is set automatically by BuildKit (e.g. "amd64", "arm64").
    # Declare it here so it is available for RUN commands in this stage.
    ARG TARGETARCH

    RUN apk add --no-cache curl tar xz openssl

    # yt-dlp: use the musl standalone binary for the target architecture.
    # The plain "yt-dlp" release is a Python zipapp and requires Python, which
    # is not present in Alpine.  The musllinux builds are fully self-contained.
    RUN set -e; \
        case "$TARGETARCH" in \
          amd64) YTDLP_BIN="yt-dlp_musllinux" ;; \
          arm64) YTDLP_BIN="yt-dlp_musllinux_aarch64" ;; \
          *) echo "Unsupported architecture: $TARGETARCH" && exit 1 ;; \
        esac; \
        curl -L -f -o /yt-dlp \
             "https://github.com/yt-dlp/yt-dlp/releases/download/2026.07.04/${YTDLP_BIN}"; \
        chmod +x /yt-dlp

    # ffmpeg: static build for the target architecture.
    RUN set -e; \
        case "$TARGETARCH" in \
          amd64) FFMPEG_ARCH="amd64" ;; \
          arm64) FFMPEG_ARCH="arm64" ;; \
          *) echo "Unsupported architecture: $TARGETARCH" && exit 1 ;; \
        esac; \
        FFMPEG_URL="https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-${FFMPEG_ARCH}-static.tar.xz"; \
        echo "Downloading ffmpeg from: $FFMPEG_URL"; \
        curl --http1.1 -L -f --retry 3 --retry-delay 2 \
             -A "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36" \
             -H "Accept: application/octet-stream" \
             -o /tmp/ffmpeg.tar.xz "$FFMPEG_URL"; \
        echo "Extracting ffmpeg archive..."; \
        tar -xf /tmp/ffmpeg.tar.xz -C /tmp --strip-components=1; \
        echo "Setting up ffmpeg binary..."; \
        mv /tmp/ffmpeg /ffmpeg; \
        chmod +x /ffmpeg; \
        rm /tmp/ffmpeg.tar.xz; \
        echo "ffmpeg setup complete"  

# ==================
# == web builder ===
# ==================
FROM node-base AS web-builder
    ENV PNPM_HOME="/pnpm"
    ENV PATH="$PNPM_HOME:$PATH"
    RUN corepack enable

    WORKDIR /app

    # copy workspace manifests first for dependency caching
    COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
    COPY apps/web/package.json ./apps/web/

    RUN --mount=type=cache,id=pnpm,target=/pnpm/store \
        pnpm install --frozen-lockfile

    COPY apps/web ./apps/web

    RUN pnpm --filter @soundome/web build

# ==================
# == api deps ======
# ==================
FROM cargo-chef-base AS api-dependencies
    WORKDIR /app

    COPY . .

    # use chef to prepare the dependency tree json
    RUN cargo chef prepare --recipe-path recipe.json

# ==================
# == api builder ===
# ==================
FROM cargo-chef-base AS api-builder
    WORKDIR /app

    RUN apk add --no-cache pkgconfig openssl-dev openssl-libs-static musl-dev

    # build dependencies in a separate layer to cache them
    COPY --from=api-dependencies /app/recipe.json .
    RUN cargo chef cook --release --recipe-path recipe.json

    # copy all the source code
    COPY . .

    # build the application
    RUN cargo build --release --bin soundome-server

# ==================
# ==== runner ======
# ==================
FROM base AS runner
    WORKDIR /app

    # create a user and a group to run the app more securely and properly
    RUN addgroup --system --gid 1000 soundome \
        && adduser --system --uid 1000 soundome

    # copy the binary from the api builder stage
    COPY --from=api-builder /app/target/release/soundome-server .

    # embed Rocket config so users don't need to set ROCKET_* env vars
    COPY Rocket.toml .

    # copy the frontend assets (served by Rocket's FileServer at data/web)
    COPY --from=web-builder /app/data/web ./data/web

    # copy Diesel migrations (required for diesel migration run at startup)
    COPY packages/database/migrations ./packages/database/migrations

    # copy external tools from the libs stage (cached independently from the Rust build)
    COPY --from=libs /yt-dlp /usr/local/bin/yt-dlp
    COPY --from=libs /ffmpeg /usr/local/bin/ffmpeg

    # ensure runtime directories exist and belong to the app user.
    # note: when bind-mounted from the host, the host directory permissions take
    # precedence — make sure the host dirs are writable by uid 1000 (e.g. chown -R 1000:1000).
    RUN mkdir -p /app/data /app/library /app/temp \
        && chown -R soundome:soundome /app

    USER soundome

    EXPOSE 8000

    CMD ["./soundome-server"]
