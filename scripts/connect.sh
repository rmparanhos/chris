#!/usr/bin/env bash
# Liga o CHRIS ao Copilot CLI: escreve o hook na pasta do seu projeto.
# Uso:  ./connect.sh [/caminho/do/projeto]   (sem argumento = pasta atual)
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
chris="$root/target/release/chris"

if [ ! -x "$chris" ]; then
  echo "O CHRIS ainda nao foi compilado. Rode ./setup.sh primeiro." >&2
  exit 1
fi

proj="${1:-$(pwd)}"
if [ ! -d "$proj" ]; then
  echo "Pasta nao encontrada: $proj" >&2
  exit 1
fi

( cd "$proj" && "$chris" install --agent copilot && "$chris" install --agent claude )
echo ""
echo "Pronto! Deixe o companion rodando (./run.sh) e use o Copilot ou o Claude Code nesse projeto."
