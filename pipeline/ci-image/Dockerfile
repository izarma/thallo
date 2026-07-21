# syntax=docker/dockerfile:1
#
# Custom Forgejo runner image for bevy_cicd_test.
#
# Build context: project root (so the Dockerfile can be kept under ci-image/).
# Build on:       oracle-runner (Debian/Ubuntu host with Podman/Docker).
# Registry:       izaforge:3003 (tailnet) or git.izaforge.com package registry.
#
# Tag convention: <registry>/izarma/bevy-cicd-runner:<image-version>-<rust-toolchain>
# Example:        izaforge:3003/izarma/bevy-cicd-runner:0.1.0-nightly-2026-01-22

FROM debian:trixie-slim

ARG RUST_TOOLCHAIN=nightly-2026-04-16
ARG BEVY_LINT_TAG=main
ARG BEVY_CLI_TAG=cli-v0.1.0-alpha.2
ARG CARGO_BINSTALL_VERSION=1.21.0
ARG NODE_VERSION=20.18.3

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    DEBIAN_FRONTEND=noninteractive \
    CARGO_NET_RETRY=10 \
    CARGO_NET_GIT_FETCH_WITH_CLI=true

# rust-cache defaults to caching ~/.cargo. Symlink it to the actual CARGO_HOME
# so the cache action hits the warm registry/git caches baked into the image.
RUN ln -s /usr/local/cargo /root/.cargo

# 1. System dependencies
# -----------------------------------------------------------------------------
# These replace the per-job `apt-get install` steps from ci.yaml/release.yaml.
#
# On an ARM64 host that builds Linux x86_64 release binaries, we need the amd64
# versions of Bevy's system libraries. Register the amd64 architecture first
# and install the cross toolchain only when required.
RUN if [ "$(dpkg --print-architecture)" = "arm64" ]; then \
        dpkg --add-architecture amd64; \
    fi \
    && apt-get update \
    && apt-get upgrade -y \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        git \
        build-essential \
        pkg-config \
        # Native Linux Bevy dependencies
        libasound2-dev \
        libudev-dev \
        libwayland-dev \
        libxkbcommon-dev \
        # Windows cross-compilation
        gcc-mingw-w64-x86-64 \
        # Web / release packaging
        libssl-dev \
        zip \
        unzip \
    && if [ "$(dpkg --print-architecture)" = "arm64" ]; then \
        apt-get install -y --no-install-recommends \
            gcc-x86-64-linux-gnu \
            libc6-dev-amd64-cross \
            libc6:amd64 \
            libasound2-dev:amd64 \
            libudev-dev:amd64 \
            libwayland-dev:amd64 \
            libxkbcommon-dev:amd64; \
    fi \
    && rm -rf /var/lib/apt/lists/*

# 1.5 Node.js runtime
# -----------------------------------------------------------------------------
# Forgejo/Gitea Actions use `act` to run workflows. JavaScript actions such as
# `actions/checkout` and `rust-cache` execute `node` inside the job container,
# so a Node LTS runtime must be available in this image.
RUN NODE_ARCH=$(dpkg --print-architecture) \
    && case "${NODE_ARCH}" in \
        amd64) NODE_ARCH=x64 ;; \
        arm64) NODE_ARCH=arm64 ;; \
        *) echo "Unsupported architecture: ${NODE_ARCH}"; exit 1 ;; \
    esac \
    && curl -fsSL --proto '=https' --tlsv1.2 \
        "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-${NODE_ARCH}.tar.xz" \
    | tar -xJf - -C /usr/local --strip-components=1 \
    && node --version \
    && npm --version

# 2. Rust toolchain
# -----------------------------------------------------------------------------
# Pin the exact nightly used by the workflows so jobs share a common cache key.
#
# NOTE: `rustc-dev` and `llvm-tools` are large (~1 GB together) but are required
# at runtime by `bevy_lint`, which loads rustc's internal dynamic libraries.
# Do not remove them unless you drop the Bevy linter from CI.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --no-modify-path --default-toolchain "${RUST_TOOLCHAIN}" \
    && rustup component add --toolchain "${RUST_TOOLCHAIN}" \
        rustfmt \
        clippy \
        rustc-dev \
        llvm-tools \
        rustc-codegen-cranelift-preview \
    && rustup target add --toolchain "${RUST_TOOLCHAIN}" \
        wasm32-unknown-unknown \
        x86_64-pc-windows-gnu \
        x86_64-unknown-linux-gnu \
    && rustup default "${RUST_TOOLCHAIN}"

# 3. Cargo helpers
# -----------------------------------------------------------------------------
# cargo-binstall is used by release.yaml to pull wasm-bindgen-cli/wasm-opt quickly.
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
    "https://github.com/cargo-bins/cargo-binstall/releases/download/v${CARGO_BINSTALL_VERSION}/cargo-binstall-$(uname -m)-unknown-linux-gnu.tgz" \
    | tar -xz -C /usr/local/cargo/bin

# 4. Bevy ecosystem tools
# -----------------------------------------------------------------------------
# `bevy_lint` is built from the `main` branch; `bevy_cli` is pinned to a Git
# tag. cargo-binstall does not reliably resolve the pinned versions from this
# repository, so `cargo install --git` is used instead. `wasm-bindgen-cli` and
# `wasm-opt` are pulled from crates.io prebuilt binaries. Cargo caches are kept
# in the image because they speed up CI builds.
RUN cargo install --git https://github.com/TheBevyFlock/bevy_cli \
        --branch "${BEVY_LINT_TAG}" --locked bevy_lint \
    && cargo install --git https://github.com/TheBevyFlock/bevy_cli \
        --tag "${BEVY_CLI_TAG}" --locked bevy_cli \
    && cargo-binstall --locked --no-confirm wasm-bindgen-cli wasm-opt

# 5. Verification
# -----------------------------------------------------------------------------
# Fail the build early if any of the expected tools are missing or broken.
# Note: cargo-binstall uses --version for install-time version requirements,
# not for printing its own version, so we just verify the binary is on PATH.
RUN cargo --version \
    && rustc --version \
    && rustfmt --version \
    && cargo clippy --version \
    && bevy --version \
    && bevy_lint --version \
    && command -v cargo-binstall \
    && node --version

WORKDIR /workspace
