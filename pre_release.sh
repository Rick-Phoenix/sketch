#!/bin/bash

set -eo pipefail

EXEC_RELEASE=false
if [[ "${2:-}" == "--execute" ]]; then
	EXEC_RELEASE=true
fi
VERSION="$1"

if [[ -z "$VERSION" ]]; then
	echo "Missing new version"
	exit 1
fi

echo "Preparing to release version $VERSION"
echo "EXEC_RELEASE status: $EXEC_RELEASE"

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
		git add "schemas/latest.json"
		git commit -m "updated json schema"
	fi

	echo "Generating changelog..."
	git cliff --tag "$VERSION" -o "CHANGELOG.md"

	if [[ -n $(git status --porcelain "CHANGELOG.md") ]]; then
		echo "Committing the new changelog..."
		git add "CHANGELOG.md"
		git commit -m "chore(release): update changelog for ${VERSION}"
	fi

	echo "Deploying documentation..."
	./update_docs.sh "$VERSION"
fi

echo "Pre-release routine finished!"
