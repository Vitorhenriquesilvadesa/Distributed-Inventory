Write-Host "=== TESTE SIMPLES - SISTEMA DISTRIBUIDO ===" -ForegroundColor Green

# Iniciar Service Discovery
Write-Host "Iniciando Service Discovery..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin service-discovery" -WindowStyle Minimized
Start-Sleep -Seconds 5

# Iniciar Hub Service
Write-Host "Iniciando Hub Service..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin hub-service" -WindowStyle Minimized
Start-Sleep -Seconds 5

# Iniciar CDs
Write-Host "Iniciando CDs..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_alpha 8083" -WindowStyle Minimized
Start-Sleep -Seconds 3
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_beta 8084" -WindowStyle Minimized
Start-Sleep -Seconds 3
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_gamma 8085" -WindowStyle Minimized
Start-Sleep -Seconds 5

Write-Host "Aguardando servicos estarem prontos..." -ForegroundColor Yellow
Start-Sleep -Seconds 15

# Testar Service Discovery
Write-Host "Testando Service Discovery..." -ForegroundColor Cyan
try {
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:8080/lookup_all" -Method GET
    Write-Host "Service Discovery OK! CDs registrados: $($response.Count)" -ForegroundColor Green
} catch {
    Write-Host "Erro no Service Discovery: $($_.Exception.Message)" -ForegroundColor Red
}

# Testar Hub
Write-Host "Testando Hub Service..." -ForegroundColor Cyan
try {
    $product = @{
        code = "teste"
        name = "Produto Teste"
        price = 10.00
        description = "Teste"
    }
    Invoke-RestMethod -Uri "http://127.0.0.1:8082/products" -Method POST -Body ($product | ConvertTo-Json) -ContentType "application/json"
    Write-Host "Hub Service OK!" -ForegroundColor Green
} catch {
    Write-Host "Erro no Hub: $($_.Exception.Message)" -ForegroundColor Red
}

# Testar CD Alpha
Write-Host "Testando CD Alpha..." -ForegroundColor Cyan
try {
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:8083/inventory/garrafas" -Method GET
    Write-Host "CD Alpha OK! Garrafas: $($response.quantity)" -ForegroundColor Green
} catch {
    Write-Host "Erro no CD Alpha: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "Teste concluido! Pressione qualquer tecla para encerrar..." -ForegroundColor Green
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

# Encerrar
Get-Process | Where-Object {$_.ProcessName -eq "cargo"} | Stop-Process -Force
Write-Host "Servicos encerrados!" -ForegroundColor Green 