#!/bin/bash

set -eo pipefail

VERSION="$1"

if [[ -z "$VERSION" ]]; then
  echo "Missing new version"
  exit 1
fi

echo "Preparing to release version $VERSION"
echo "EXEC_RELEASE status: $EXEC_RELEASE"

echo "Running tests..."

cargo test --all-features -- -q --nocapture

if [[ "$EXEC_RELEASE" == "true" ]]; then
  echo "Deploying documentation..."
  ./update_docs.sh

  echo "Generating JSON schema"
  cargo run --bin json-schema "$VERSION"

  echo "Updating changelog"
  git cliff --tag "$VERSION" -o "CHANGELOG.md"
fi

echo "Pre-release routine finished!"
