# Hosted Eplyx workspace (Milestone 18). Build from the repository root:
#   docker build -f cloud/Dockerfile --build-arg EPLYX_BUILD_COMMIT=$(git rev-parse HEAD) .
# The image holds only the eplyx-cloud server. It never captures, executes or
# replays, and it needs no Solana RPC credential.
FROM rust:1-bookworm AS build
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
RUN cargo build --locked --release -p eplyx-cloud --bin eplyx-cloud

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home eplyx
COPY --from=build /src/target/release/eplyx-cloud /usr/local/bin/eplyx-cloud
USER eplyx
ENV PORT=8080
EXPOSE 8080
CMD ["eplyx-cloud"]
