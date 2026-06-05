#!/usr/bin/env bash
# =====================================================================
#  CHRIS - instalacao automatica para Linux/macOS
#  Instala Rust + dependencias de sistema e compila o projeto.
#  Uso:  ./setup.sh
# =====================================================================
set -euo pipefail
cd "$(dirname "$0")/.."   # raiz do projeto

echo ""
echo "===== Instalacao do CHRIS ====="
echo ""

# ---- 1. Rust ----
if ! command -v cargo >/dev/null 2>&1; then
  echo "-> Instalando Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

# ---- 2. Dependencias de sistema (a 'tela' do app) ----
if [[ "${OSTYPE:-}" == linux* ]]; then
  if command -v apt-get >/dev/null 2>&1; then
    echo "-> Instalando libs do sistema (vai pedir sua senha)..."
    sudo apt-get update
    sudo apt-get install -y \
      libwebkit2gtk-4.1-dev build-essential curl wget file \
      libssl-dev libayatana-appindicator3-dev librsvg2-dev
  else
    echo "!! Sua distro nao usa apt. Instale o equivalente ao 'webkit2gtk-4.1'"
    echo "   e as ferramentas de build antes de continuar."
  fi
elif [[ "${OSTYPE:-}" == darwin* ]]; then
  echo "-> Garantindo as Command Line Tools do Xcode..."
  xcode-select --install 2>/dev/null || true
fi

# ---- 3. Compila ----
echo ""
echo "-> Compilando o CHRIS (a primeira vez demora alguns minutos)..."
cargo build --release -p chris-cli
cargo build --release -p companiond

echo ""
echo "===== Pronto! ====="
echo "1) Para iniciar o companion:   ./run.sh"
echo "2) Para ligar no seu projeto:  ./connect.sh /caminho/do/projeto"
echo ""
