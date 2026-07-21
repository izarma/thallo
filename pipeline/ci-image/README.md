# Custom CI runner image

This directory contains the build assets for the single prebuilt image used by
`oracle-runner` for every CI and release job.

## Inventory: what the image contains

The image is built from `debian:trixie-slim` and includes everything the
workflows currently install at runtime:

### Base system (always installed)

- Core tools: `ca-certificates`, `curl`, `git`, `build-essential`, `pkg-config`
- Node.js LTS runtime (required by Forgejo/Gitea Actions JavaScript actions such as
  `actions/checkout` and `rust-cache`)
- Native Linux Bevy dependencies:
  `libasound2-dev`, `libudev-dev`, `libwayland-dev`, `libxkbcommon-dev`
- Windows cross-compilation: `gcc-mingw-w64-x86-64`
- Release/packaging helpers: `zip`, `unzip`, `libssl-dev`

### Cross-compilation support (ARM64 images only)

Installed only when the image is built on an ARM64 host for Linux x86_64 release
binaries:

- Linux x86_64 cross toolchain:
  `gcc-x86-64-linux-gnu`, `libc6-dev-amd64-cross`
- ARM64 Linux x86_64 runtime + Bevy dev libraries:
  `libc6:amd64`, `libasound2-dev:amd64`, `libudev-dev:amd64`,
  `libwayland-dev:amd64`, `libxkbcommon-dev:amd64`

### Rust toolchain

- Pinned nightly toolchain: `nightly-2026-04-16`
- Components:
  `rustfmt`, `clippy`, `rustc-dev`, `llvm-tools`, `rustc-codegen-cranelift-preview`
  - `rustc-dev` and `llvm-tools` are kept because `bevy_lint` loads rustc's
    internal libraries at runtime. They are ~1 GB combined.
- Targets:
  `wasm32-unknown-unknown`, `x86_64-pc-windows-gnu`, `x86_64-unknown-linux-gnu`

### Cargo tooling

- `cargo-binstall` (fast binary installs)
- `bevy_lint` (tag `main`) — built from source via `cargo install`
  (no reliable prebuilt binary is available for cargo-binstall)
- `bevy_cli` (tag `cli-v0.1.0-alpha.2`) — built from source via `cargo install`
  (no reliable prebuilt binary is available for cargo-binstall)
- `wasm-bindgen-cli` and `wasm-opt` (pre-fetched for web releases)

Cargo caches (`registry/cache`, `registry/src`, `git/db`, `git/checkouts`) are
**kept** in the image because they speed up CI builds.

## Design decisions

- **One image, not split per job.**
  All jobs share the same toolchain and system dependencies. The image is large,
  but it is pulled once per runner and reused, eliminating repeated installs.
- **Built on oracle-runner itself.**
  The runner already has Podman configured (`docker_host` points to the podman
  socket). Building on the same architecture avoids cross-building complexity.
  The image is large mainly because of the nightly Rust toolchain and
  `rustc-dev`/`llvm-tools` required by `bevy_lint`; image size is traded for
  faster CI builds by keeping cargo caches warm.
- **Hosted on git.izaforge.com package registry (with tailnet fallback).**
  Uses your own Forgejo instance. Authentication is done with a Personal Access
  Token passed as `FORGEJO_PACKAGE_TOKEN`. The registry URL can be overridden
  via the `REGISTRY` env var, e.g. to push through a tailnet endpoint that
  bypasses Cloudflare upload limits.
- **Explicit version tags.**
  Tags follow `<image-version>-<rust-toolchain>` (e.g.
  `0.1.0-nightly-2026-04-16`). Workflows pin the exact tag so cache keys and
  reproducibility stay intact.
- **Optimized for build time, not image size.**
  `rustc-dev`/`llvm-tools` are kept because `bevy_lint` needs them. Cargo
  caches are kept warm in the image to speed up CI builds. `wasm-bindgen-cli`
  and `wasm-opt` use prebuilt binaries; `bevy_cli` and `bevy_lint` are built
  from their Git tags because cargo-binstall cannot reliably resolve their
  pinned versions from this repository.

## Build and push

From the project root on oracle-runner: default registry is `git.izaforge.com`.
```bash
export REGISTRY=izaforge:3003
export FORGEJO_PACKAGE_TOKEN=<your PAT>
bash ci-image/build.sh
```

To build a specific combination:

```bash
bash ci-image/build.sh 0.1.0 nightly-2026-04-16 main cli-v0.1.0-alpha.2
```

## Runner configuration

If your registry uses plain HTTP, make sure Podman on the runner allows it.
Add to `~/.config/containers/registries.conf`

```toml
[[registry]]
location = "izaforge:3003"
insecure = true
```

After restarting the runner, every job that uses `runs-on: oracle-runner` will
execute inside the container instead of on the host.

## Updating the image

1. Edit `ci-image/Dockerfile` or bump the toolchain/version arguments.
2. Run `bash ci-image/build.sh <new-version>`.
3. Update the label in `runner-reference/runner-config.yml`.
4. Restart the runner.

Keep old tags around until all in-flight jobs have finished.

## Key files
- `Dockerfile` — image definition.
- `build.sh` — build and push helper for oracle-runner.
