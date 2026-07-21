#!/bin/bash
#
# Build and push the custom Forgejo runner image from oracle-runner.
#
# Usage:
#   export FORGEJO_PACKAGE_TOKEN=<your PAT>
#   ./ci-image/build.sh [image-version] [rust-toolchain] [bevy-lint-tag] [bevy-cli-tag]
#
# Defaults:
#   image-version  -> 0.1.0
#   rust-toolchain -> nightly-2026-04-16
#   bevy-lint-tag  -> main
#   bevy-cli-tag   -> cli-v0.1.0-alpha.2
#
# The image is pushed to git.izaforge.com/izarma/bevy-cicd-runner.

set -euo pipefail

IMAGE_VERSION="${1:-0.1.0}"
RUST_TOOLCHAIN="${2:-nightly-2026-04-16}"
BEVY_LINT_TAG="${3:-main}"
BEVY_CLI_TAG="${4:-cli-v0.1.0-alpha.2}"

# Override this to push to a different registry, e.g. a tailnet endpoint.
REGISTRY="${REGISTRY:-git.izaforge.com}"
IMAGE_NAME="${REGISTRY}/izarma/bevy-cicd-runner"
TAG="${IMAGE_NAME}:${IMAGE_VERSION}-${RUST_TOOLCHAIN}"

# The runner config already points at podman; podman is used here, but docker
# works too if that is what is installed on oracle-runner.
CONTAINER_ENGINE="${CONTAINER_ENGINE:-podman}"

echo "Building ${TAG} with ${CONTAINER_ENGINE}..."
echo "  Rust toolchain: ${RUST_TOOLCHAIN}"
echo "  bevy_lint tag:  ${BEVY_LINT_TAG}"
echo "  bevy_cli tag:   ${BEVY_CLI_TAG}"

# Authenticate to the Forgejo package registry.
# The PAT needs at least `package:write` scope.
if [ -z "${FORGEJO_PACKAGE_TOKEN:-}" ]; then
    echo "ERROR: FORGEJO_PACKAGE_TOKEN is not set." >&2
    exit 1
fi
echo "${FORGEJO_PACKAGE_TOKEN}" | "${CONTAINER_ENGINE}" login -u izarma --password-stdin "${REGISTRY}"

# Build from the repository root so the Dockerfile path is correct.
"${CONTAINER_ENGINE}" build \
    --build-arg RUST_TOOLCHAIN="${RUST_TOOLCHAIN}" \
    --build-arg BEVY_LINT_TAG="${BEVY_LINT_TAG}" \
    --build-arg BEVY_CLI_TAG="${BEVY_CLI_TAG}" \
    -t "${TAG}" \
    -f ci-image/Dockerfile \
    .

# Push the explicit version tag.
"${CONTAINER_ENGINE}" push "${TAG}"

# Also tag as "latest" for convenience, but workflows should pin explicit tags.
LATEST_TAG="${IMAGE_NAME}:latest"
"${CONTAINER_ENGINE}" tag "${TAG}" "${LATEST_TAG}"
"${CONTAINER_ENGINE}" push "${LATEST_TAG}"

echo ""
echo "Published:"
echo "  ${TAG}"
echo "  ${LATEST_TAG}"
echo ""
echo "Update runner-reference/runner-config.yml and workflows to use the new tag."
