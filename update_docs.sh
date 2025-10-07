#!/bin/bash

set -e

VERSION="$1"

if [[ -z "$VERSION" ]]; then
  echo "Missing new version"
  exit 1
fi

MINOR_VERSION=$(echo "$VERSION" | cut -d'.' -f1-2)

if [[ ! "$MINOR_VERSION" =~ ^[0-9]+\.[0-9]+$ ]]; then
  echo "Error: Invalid version format. Expected 'major.minor', but got '$MINOR_VERSION'." >&2
  exit 1
fi

echo "Building and deploying docs for v$VERSION (minor version v$MINOR_VERSION)..."

mdbook build docs

BOOK_DIR="docs/book"
TMP_DIR=$(mktemp -d)
REPO_URL=$(git config --get remote.origin.url)
VERSIONS_DIR="versions"

echo "Cloning gh-pages branch into a temporary directory..."
git clone --branch gh-pages "$REPO_URL" "$TMP_DIR"

echo "Cleaning the root directory..."
find "$TMP_DIR" -mindepth 1 -maxdepth 1 \
  ! -name '.git' \
  ! -name "$VERSIONS_DIR" \
  -print

echo "Copying new docs..."

# Copy to the root for the latest version
cp -r "$BOOK_DIR"/* "$TMP_DIR/"

# Define the path for the minor version's docs
VERSIONED_DOCS_PATH="$TMP_DIR/$VERSIONS_DIR/$MINOR_VERSION"

# Check for and remove existing directory for this minor version
if [ -d "$VERSIONED_DOCS_PATH" ]; then
  echo "Found existing docs for v$MINOR_VERSION. Replacing them..."
  rm -rf "$VERSIONED_DOCS_PATH"
fi

# Copy to the versioned directory
mkdir -p "$VERSIONED_DOCS_PATH"
cp -r "$BOOK_DIR"/* "$VERSIONED_DOCS_PATH/"

echo "Docs copied to root and /$VERSIONS_DIR/$MINOR_VERSION/"

# Commit and push
cd "$TMP_DIR"

git add .
git commit -m "Deploy docs for v$VERSION"
git push origin gh-pages

echo "Docs for v$VERSION deployed successfully"

# Clean up
cd ..
rm -rf "$TMP_DIR"
