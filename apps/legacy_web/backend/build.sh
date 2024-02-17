#!/usr/bin/env bash

set -o errexit  # abort on nonzero exitstatus
set -o nounset  # abort on unbound variable
set -o pipefail  # don't hide errors within pipes

TARGET="x86_64-unknown-linux-gnu"
BIN_NAME="backend"
REPO_ROOT="$(readlink -f "$(dirname "${0}")/../../..")"
BACKEND_DIR="$(readlink -f "$(dirname "${0}")")"
RELATIVE_RELEASE_DIR="target/${TARGET}/release" 
echo "REPO_ROOT=${REPO_ROOT}"
echo "BACKEND_DIR=${BACKEND_DIR}"

cargo zigbuild --release --target "${TARGET}" --bin "${BIN_NAME}"

docker buildx build "${REPO_ROOT}/${RELATIVE_RELEASE_DIR}" \
	--platform=linux/amd64 \
	-f "${BACKEND_DIR}/Dockerfile" \
	-t "${BIN_NAME}" \
	--build-arg BIN_PATH="${BIN_NAME}"

echo "built docker image. You can run via:"
echo "\"docker run -it -p HOST_PORT:8080 ${BIN_NAME}\""
