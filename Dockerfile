FROM lukemathwalker/cargo-chef:0.1.77-rust-alpine AS cargo-chef-base
FROM node:22-alpine AS node-base
FROM alpine:3.23 AS base

# ==================
# ===== libs =======
# ==================
FROM base AS libs
    RUN apk add --no-cache curl tar xz openssl

    RUN curl -L -o /yt-dlp "https://github.com/yt-dlp/yt-dlp/releases/2026.03.17/download/yt-dlp" \
        && chmod +x /yt-dlp

    RUN curl -L -o /tmp/ffmpeg.tar.xz "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz" \
        && tar -xf /tmp/ffmpeg.tar.xz -C /tmp --strip-components=1 \
        && mv /tmp/ffmpeg /ffmpeg \
        && chmod +x /ffmpeg \
        && rm /tmp/ffmpeg.tar.xz

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

    # copy external tools from the libs stage (cached independently from the Rust build)
    COPY --from=libs /yt-dlp /usr/local/bin/yt-dlp
    COPY --from=libs /ffmpeg /usr/local/bin/ffmpeg

    USER soundome

    EXPOSE 8000

    CMD ["./soundome-server"]
