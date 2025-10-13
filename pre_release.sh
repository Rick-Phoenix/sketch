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

cargo test --all-features -p sketch -- -q --nocapture

if [[ "$EXEC_RELEASE" == "true" ]]; then
  MINOR_VERSION=$(echo "$VERSION" | cut -d'.' -f1-2)

  if [[ ! "$MINOR_VERSION" =~ ^[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Invalid version format. Expected 'major.minor', but got '$MINOR_VERSION'." >&2
    exit 1
  fi

  echo "Generating JSON schema"

  TARGET_SCHEMA="schemas/v$MINOR_VERSION.json"

  cargo run --bin json-schema "$VERSION"

  if [[ -n $(git status --porcelain "$TARGET_SCHEMA") ]]; then
    git add "$TARGET_SCHEMA"
    git commit -m "updated json schema"
  fi

  echo "Updating changelog"
  git cliff --tag "$VERSION" -o "CHANGELOG.md"

  echo "Deploying documentation..."
  ./update_docs.sh "$VERSION"
fi

echo "Pre-release routine finished!"
