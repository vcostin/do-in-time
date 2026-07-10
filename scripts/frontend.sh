#!/usr/bin/env sh
# Run the Vite frontend with Deno when available, otherwise npm.
set -eu
cd "$(dirname "$0")/.."
cmd="${1:-dev}"

case "$cmd" in
  dev|build|preview) ;;
  *)
    echo "usage: $0 [dev|build|preview]" >&2
    exit 1
    ;;
esac

if command -v deno >/dev/null 2>&1 && [ -f deno.json ]; then
  exec deno task "$cmd"
fi

if command -v npm >/dev/null 2>&1; then
  exec npm run "$cmd"
fi

echo "Neither deno nor npm is available to run '$cmd'." >&2
exit 1
