# Liga o CHRIS ao Copilot CLI: escreve o hook na pasta do seu projeto.
$chris = Join-Path $PSScriptRoot "target\release\chris.exe"
if (-not (Test-Path $chris)) {
    Write-Host "O CHRIS ainda nao foi compilado. Rode o setup.bat primeiro." -ForegroundColor Red
    exit 1
}

Write-Host ""
$proj = Read-Host "Cole o caminho da pasta do seu projeto (onde voce usa o Copilot) e tecle Enter"
if ([string]::IsNullOrWhiteSpace($proj)) { $proj = (Get-Location).Path }
if (-not (Test-Path $proj)) {
    Write-Host "Pasta nao encontrada: $proj" -ForegroundColor Red
    exit 1
}

Push-Location $proj
& $chris install --agent copilot
& $chris install --agent claude
Pop-Location

Write-Host ""
Write-Host "Pronto! Deixe o companion rodando (run.bat) e use o Copilot ou o Claude Code nesse projeto." -ForegroundColor Green
