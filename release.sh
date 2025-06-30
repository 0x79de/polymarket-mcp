#!/bin/bash
set -e

# Release script for polymarket-mcp
# Usage: ./release.sh <version>
# Example: ./release.sh 0.1.0

if [ $# -eq 0 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0"
    exit 1
fi

VERSION="$1"
TAG="v$VERSION"

# Validate version format (basic check)
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be in format X.Y.Z (e.g., 0.1.0)"
    exit 1
fi

echo "Creating release $TAG..."

# Check if we're on main/master branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "master" ] && [ "$CURRENT_BRANCH" != "main" ]; then
    echo "Warning: You're not on main/master branch. Current branch: $CURRENT_BRANCH"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check if working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Working directory is not clean. Please commit or stash your changes."
    git status --short
    exit 1
fi

# Update version in Cargo.toml
echo "Updating version in Cargo.toml..."
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Update Cargo.lock
echo "Updating Cargo.lock..."
cargo check

# Commit version change
echo "Committing version change..."
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to $VERSION"

# Create and push tag
echo "Creating tag $TAG..."
git tag -a "$TAG" -m "Release $TAG"

echo "Pushing changes and tag..."
git push origin "$CURRENT_BRANCH"
git push origin "$TAG"

echo "✅ Release $TAG created successfully!"
echo "📦 GitHub Actions will now build and publish the release automatically."
echo "🔗 Check the progress at: https://github.com/0x79de/polymarket-mcp/actions"
echo "📋 View releases at: https://github.com/0x79de/polymarket-mcp/releases"