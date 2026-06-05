#!/usr/bin/env bash
# Inicia o companion (blob + bandeja). Deixe este terminal aberto.
set -euo pipefail
cd "$(dirname "$0")/.."   # raiz do projeto
[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
exec cargo run --release -p companiond
