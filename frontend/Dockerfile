# Hosted Eplyx Stock Transition frontend and analysis service. Builds from the
# repository root: Railway's GitHub source, or a context staged by
# scripts/railway/stage-frontend.mjs. Everything comes from tracked sources,
# including the registered candidate program. The engine resolves its evidence
# relative to the build-time repository root, so runtime keeps /src.
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

# The registered Eplyx demo conversion program, built with the same pinned
# cargo-build-sbf the release workflow uses. Never deployed to any cluster.
FROM rust:1-bookworm AS programs
RUN cargo install cargo-build-sbf --version 4.4.0 --locked
WORKDIR /programs
COPY programs/eplyx-demo-conversion eplyx-demo-conversion
RUN cargo-build-sbf --manifest-path eplyx-demo-conversion/Cargo.toml --sbf-out-dir /out \
    && sha256sum /out/eplyx_demo_conversion.so

FROM node:22-bookworm-slim
WORKDIR /src
COPY package.json package-lock.json ./
RUN npm ci --omit=dev --ignore-scripts
COPY . .
COPY --from=engine /src/target/release/eplyx-lifecycle /src/target/release/eplyx-lifecycle
COPY --from=programs /out/eplyx_demo_conversion.so /src/artifacts/eplyx_demo_conversion.so
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
