#!/usr/bin/env bash
set -euo pipefail

# shellquest publish script
# Bumps version, commits, pushes, creates GitHub release, publishes to crates.io,
# builds and pushes Docker image.
#
# Usage:
#   ./publish.sh patch    # 1.1.0 -> 1.1.1
#   ./publish.sh minor    # 1.1.0 -> 1.2.0
#   ./publish.sh major    # 1.1.0 -> 2.0.0
#   ./publish.sh 1.3.0    # set exact version

DOCKER_REPO="${DOCKER_REPO:-duysqubix/shellquest}"

# ── Colors ──
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
RESET='\033[0m'

info() { echo -e "${CYAN}${BOLD}>>>${RESET} $1"; }
ok() { echo -e "${GREEN}${BOLD}  ✓${RESET} $1"; }
warn() { echo -e "${YELLOW}${BOLD}  !${RESET} $1"; }
fail() {
  echo -e "${RED}${BOLD}  ✗${RESET} $1"
  exit 1
}

# ── Parse args ──
BUMP="${1:-}"
if [ -z "$BUMP" ]; then
  echo "Usage: ./publish.sh <patch|minor|major|X.Y.Z>"
  exit 1
fi

# ── Get current version ──
CURRENT=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
IFS='.' read -r MAJOR MINOR PATCH <<<"$CURRENT"

case "$BUMP" in
patch) PATCH=$((PATCH + 1)) ;;
minor)
  MINOR=$((MINOR + 1))
  PATCH=0
  ;;
major)
  MAJOR=$((MAJOR + 1))
  MINOR=0
  PATCH=0
  ;;
*.*.*) IFS='.' read -r MAJOR MINOR PATCH <<<"$BUMP" ;;
*) fail "Invalid bump: $BUMP. Use patch, minor, major, or X.Y.Z" ;;
esac

NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"

echo ""
echo -e "${BOLD}  ⚔️  shellquest publish${RESET}"
echo -e "  ${DIM}${CURRENT} -> ${RESET}${GREEN}${BOLD}${NEW_VERSION}${RESET}"
echo ""

# ── Preflight checks ──
info "Preflight checks..."
command -v cargo &>/dev/null || fail "cargo not found"
command -v git &>/dev/null || fail "git not found"
command -v gh &>/dev/null || fail "gh not found"
command -v docker &>/dev/null || fail "docker not found"

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
  fail "Uncommitted changes. Commit or stash first."
fi
ok "Working tree clean"

# ── Bump version ──
info "Bumping version to ${NEW_VERSION}..."
sed -i "s/^version = \"${CURRENT}\"/version = \"${NEW_VERSION}\"/" Cargo.toml
cargo build --release 2>&1 | tail -1
ok "Version bumped and built"

# ── Commit & push ──
info "Committing and pushing..."
git add -A
git commit -m "v${NEW_VERSION}" --quiet
git push origin master --quiet
ok "Pushed to GitHub"

# ── GitHub release ──
info "Creating GitHub release..."
gh release create "v${NEW_VERSION}" \
  --title "v${NEW_VERSION}" \
  --generate-notes \
  --latest \
  2>&1 | tail -1
ok "GitHub release v${NEW_VERSION} created"

# ── Cargo publish ──
info "Publishing to crates.io..."
cargo publish 2>&1 | tail -1
ok "Published shellquest v${NEW_VERSION} to crates.io"

# ── Done ──
echo ""
echo -e "${GREEN}${BOLD}  ✓ shellquest v${NEW_VERSION} published everywhere!${RESET}"
echo ""
echo -e "  ${DIM}crates.io:${RESET}  cargo install shellquest"
echo -e "  ${DIM}github:${RESET}    https://github.com/duysqubix/shellquest/releases/tag/v${NEW_VERSION}"
echo -e "  ${DIM}docker:${RESET}    docker pull ${DOCKER_REPO}:${NEW_VERSION}"
echo ""
