#!/usr/bin/env bash
# Fails if `domain` (directly or transitively) depends on a runtime or I/O crate.
# Dev-dependencies are ignored: tests may use tokio to run async code.
set -euo pipefail

forbidden='^(axum|hyper|mio|reqwest|sqlx|tokio|tower) '

if cargo tree --package domain --edges normal --prefix none --format '{p}' | grep -E "$forbidden"; then
    echo "error: domain must stay pure, but depends on the crates listed above" >&2
    exit 1
fi

echo "domain has no runtime or I/O dependencies"
