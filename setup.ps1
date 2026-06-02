# =====================================================================
#  CHRIS - instalacao automatica para Windows
#  Instala Rust + C++ Build Tools + WebView2 e compila o projeto.
#  Rode com duplo-clique no setup.bat (ou: powershell -File setup.ps1)
# =====================================================================

function Have($cmd) { return [bool](Get-Command $cmd -ErrorAction SilentlyContinue) }

Write-Host ""
Write-Host "===== Instalacao do CHRIS =====" -ForegroundColor Cyan
Write-Host ""

# ---- 0. winget disponivel? ----
if (-not (Have winget)) {
    Write-Host "ERRO: 'winget' nao encontrado." -ForegroundColor Red
    Write-Host "Abra a Microsoft Store, instale/atualize o 'App Installer' e rode de novo."
    exit 1
}

# winget pode retornar codigo != 0 quando o pacote ja esta instalado;
# por isso nao abortamos o script por causa disso.
function WingetInstall($id, $label, $extra) {
    Write-Host ("-> {0}..." -f $label) -ForegroundColor Yellow
    winget install --id $id -e --accept-package-agreements --accept-source-agreements $extra 2>$null
}

# ---- 1. C++ Build Tools (necessario pro Rust MSVC e pro Tauri) ----
WingetInstall "Microsoft.VisualStudio.2022.BuildTools" "C++ Build Tools (pode demorar bastante)" `
    @("--override", "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended")

# ---- 2. WebView2 (a 'tela' do app; normalmente ja existe) ----
WingetInstall "Microsoft.EdgeWebView2Runtime" "WebView2 Runtime" @()

# ---- 3. Rust ----
if (-not (Have cargo)) {
    WingetInstall "Rustlang.Rustup" "Rust" @()
    # deixa o cargo acessivel nesta mesma sessao
    $env:Path += ";$env:USERPROFILE\.cargo\bin"
}
# garante a toolchain certa do Windows
if (Have rustup) { rustup default stable-msvc 2>$null | Out-Null }

if (-not (Have cargo)) {
    Write-Host ""
    Write-Host "Rust foi instalado, mas o 'cargo' ainda nao esta no PATH desta janela." -ForegroundColor Yellow
    Write-Host "FECHE esta janela e rode o setup.bat de novo para concluir a compilacao."
    exit 1
}

# ---- 4. Compila o CHRIS ----
Write-Host ""
Write-Host "-> Compilando o CHRIS (a primeira vez demora alguns minutos)..." -ForegroundColor Yellow
cargo build --release -p chris-cli
if ($LASTEXITCODE -ne 0) { Write-Host "Falha ao compilar a CLI." -ForegroundColor Red; exit 1 }
cargo build --release -p companiond
if ($LASTEXITCODE -ne 0) { Write-Host "Falha ao compilar o app." -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "===== Pronto! =====" -ForegroundColor Green
Write-Host "1) Para iniciar o companion:   duplo-clique em  run.bat"
Write-Host "2) Para ligar no seu projeto:  rode  connect.bat  dentro da pasta do projeto"
Write-Host ""
