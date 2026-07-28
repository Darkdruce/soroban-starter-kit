#!/usr/bin/env bash
# Installs a git pre-commit hook that runs format-check and a quick Clippy
# pass, so formatting/lint issues are caught locally instead of in CI.
set -euo pipefail

HOOK_DIR="$(git rev-parse --git-path hooks)"
HOOK_FILE="$HOOK_DIR/pre-commit"

mkdir -p "$HOOK_DIR"

cat > "$HOOK_FILE" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

./scripts/format-check.sh
cargo clippy --workspace --all-targets -- -D warnings
EOF

chmod +x "$HOOK_FILE"

echo "Installed pre-commit hook at $HOOK_FILE"
