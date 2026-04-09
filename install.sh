#!/usr/bin/env bash
set -euo pipefail

# shellquest installer
# Usage: curl -fsSL https://raw.githubusercontent.com/USER/shellquest/main/install.sh | bash

REPO="https://github.com/USER/shellquest.git"
INSTALL_DIR="${SHELLQUEST_DIR:-$HOME/.shellquest-src}"
BINARY_NAME="sq"

# ── Colors ──
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
DIM='\033[2m'
RESET='\033[0m'

info()  { echo -e "${CYAN}${BOLD}>>>${RESET} $1"; }
ok()    { echo -e "${GREEN}${BOLD}  ✓${RESET} $1"; }
warn()  { echo -e "${YELLOW}${BOLD}  !${RESET} $1"; }
fail()  { echo -e "${RED}${BOLD}  ✗${RESET} $1"; exit 1; }

echo ""
echo -e "${MAGENTA}${BOLD}  ⚔️  shellquest (sq) — The Passive Terminal RPG ⚔️${RESET}"
echo -e "${DIM}  Your shell is the dungeon.${RESET}"
echo ""

# ── Check dependencies ──
info "Checking dependencies..."

if ! command -v cargo &>/dev/null; then
    fail "Rust/Cargo not found. Install from https://rustup.rs"
fi
ok "cargo $(cargo --version | awk '{print $2}')"

if ! command -v git &>/dev/null; then
    fail "git not found. Install git first."
fi
ok "git $(git --version | awk '{print $3}')"

# ── Clone or update ──
if [ -d "$INSTALL_DIR" ]; then
    info "Updating existing installation..."
    cd "$INSTALL_DIR"
    git pull --ff-only origin main 2>/dev/null || git pull --ff-only origin master 2>/dev/null || true
    ok "Source updated"
else
    info "Cloning shellquest..."
    git clone --depth 1 "$REPO" "$INSTALL_DIR" 2>/dev/null
    cd "$INSTALL_DIR"
    ok "Source cloned to $INSTALL_DIR"
fi

# ── Build & install ──
info "Building release binary..."
cargo install --path . --force 2>&1 | tail -3
ok "$BINARY_NAME installed to $(which $BINARY_NAME 2>/dev/null || echo '~/.cargo/bin/sq')"

# ── Detect shell ──
CURRENT_SHELL="$(basename "$SHELL" 2>/dev/null || echo "unknown")"
info "Detected shell: ${BOLD}$CURRENT_SHELL${RESET}"

# ── Install shell hook ──
install_hook() {
    local shell_type="$1"
    local rc_file="$2"

    if [ ! -f "$rc_file" ]; then
        touch "$rc_file"
    fi

    if grep -q "__sq_hook" "$rc_file" 2>/dev/null; then
        ok "Shell hook already installed in $rc_file"
        return
    fi

    echo "" >> "$rc_file"
    $BINARY_NAME hook --shell "$shell_type" >> "$rc_file"
    ok "Shell hook added to $rc_file"
}

case "$CURRENT_SHELL" in
    zsh)
        # Prefer .zshrc_local if it exists, otherwise .zshrc
        if [ -f "$HOME/.zshrc_local" ]; then
            install_hook "zsh" "$HOME/.zshrc_local"
        else
            install_hook "zsh" "$HOME/.zshrc"
        fi
        ;;
    bash)
        install_hook "bash" "$HOME/.bashrc"
        ;;
    fish)
        install_hook "fish" "$HOME/.config/fish/config.fish"
        ;;
    *)
        warn "Unknown shell '$CURRENT_SHELL'. Add the hook manually:"
        echo -e "    ${DIM}$BINARY_NAME hook --shell zsh >> ~/.zshrc${RESET}"
        ;;
esac

# ── Done ──
echo ""
echo -e "${GREEN}${BOLD}  Installation complete!${RESET}"
echo ""
echo -e "  ${BOLD}Next steps:${RESET}"
echo -e "    ${CYAN}1.${RESET} Reload your shell:  ${DIM}source ~/.zshrc${RESET}  (or restart terminal)"
echo -e "    ${CYAN}2.${RESET} Create a character:  ${BOLD}sq init${RESET}"
echo -e "    ${CYAN}3.${RESET} Just use your terminal — events happen automatically!"
echo ""
echo -e "  ${BOLD}Commands:${RESET}"
echo -e "    ${DIM}sq status${RESET}      View your character"
echo -e "    ${DIM}sq inventory${RESET}   Check your gear"
echo -e "    ${DIM}sq journal${RESET}     Adventure log"
echo -e "    ${DIM}sq prestige${RESET}    Ascend at level 150"
echo ""
