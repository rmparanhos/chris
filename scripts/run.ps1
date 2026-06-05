# Inicia o companion (blob + bandeja). Deixe esta janela aberta enquanto usa.
Set-Location (Split-Path $PSScriptRoot -Parent)
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $env:Path += ";$env:USERPROFILE\.cargo\bin"
}
cargo run --release -p companiond
