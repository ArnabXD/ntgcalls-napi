#!/usr/bin/env bash
set -e

VERSION="${1:?Usage: scripts/release.sh <version>  (e.g. 0.3.2)}"

# Strip leading 'v' if provided
VERSION="${VERSION#v}"
TAG="v${VERSION}"

# Bump version in package.json without creating a git tag
npm version "$VERSION" --no-git-tag-version

# Sync version in Cargo.toml
sed -i "0,/^version = .*/s/^version = .*/version = \"${VERSION}\"/" Cargo.toml

# Regenerate CHANGELOG.md
npx git-cliff -o CHANGELOG.md

git add package.json Cargo.toml CHANGELOG.md
git commit -m "chore: release ${VERSION}"
git tag "$TAG"

echo "Tagged ${TAG}. Run: git push && git push origin ${TAG}"
