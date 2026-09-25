# Hosted Eplyx Stock Transition frontend and analysis service. Build from a
# context staged by scripts/railway/stage-frontend.mjs. The engine resolves its
# evidence relative to the build-time repository root, so runtime keeps /src.
# Fresh analyses need SOLANA_RPC_URL (a Railway variable, never in the image)
# and are gated by EPLYX_ANALYSIS_ACCESS_CODE.
FROM rust:1-bookworm AS engine
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY interface interface
COPY engine engine
COPY cloud cloud
COPY fixtures/program-id.txt fixtures/program-id.txt
COPY frontend/dashboard frontend/dashboard
COPY frontend/cloud frontend/cloud
COPY frontend/src/mode.js frontend/src/format.js frontend/src/brand.js frontend/src/
COPY frontend/public/logo.svg frontend/public/logo.svg
ARG EPLYX_BUILD_COMMIT=""
ENV EPLYX_BUILD_COMMIT=${EPLYX_BUILD_COMMIT}
RUN cargo build --locked --release -p eplyx-lifecycle-impact --bin eplyx-lifecycle

FROM node:22-bookworm-slim
WORKDIR /src
COPY package.json package-lock.json ./
RUN npm ci --omit=dev --ignore-scripts
COPY . .
COPY --from=engine /src/target/release/eplyx-lifecycle /src/target/release/eplyx-lifecycle
RUN node frontend/build.mjs \
    && mkdir -p /data/analysis-runs \
    && chown -R node:node /data /src/frontend/public
ENV NODE_ENV=production \
    HOST=0.0.0.0 \
    PORT=8080 \
    EPLYX_ENGINE=/src/target/release/eplyx-lifecycle \
    EPLYX_RUN_DIRECTORY=/data/analysis-runs
USER node
EXPOSE 8080
CMD ["node", "frontend/serve.mjs", "--production"]
