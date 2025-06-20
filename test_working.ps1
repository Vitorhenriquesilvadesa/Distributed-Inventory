Write-Host "=== TESTE FUNCIONAL - SISTEMA DISTRIBUIDO ===" -ForegroundColor Green
Write-Host ""

# Iniciar Service Discovery
Write-Host "1. Iniciando Service Discovery..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin service-discovery" -WindowStyle Minimized
Start-Sleep -Seconds 8

# Testar Service Discovery
Write-Host "   Testando Service Discovery..." -ForegroundColor Cyan
try {
    $services = Invoke-RestMethod -Uri "http://127.0.0.1:8080/lookup_all" -Method GET
    Write-Host "   ✓ Service Discovery OK! CDs registrados: $($services.Count)" -ForegroundColor Green
} catch {
    Write-Host "   ✗ Erro no Service Discovery: $($_.Exception.Message)" -ForegroundColor Red
}

# Iniciar Hub Service
Write-Host ""
Write-Host "2. Iniciando Hub Service..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin hub-service" -WindowStyle Minimized
Start-Sleep -Seconds 8

# Testar Hub Service
Write-Host "   Testando Hub Service..." -ForegroundColor Cyan
try {
    $product = @{
        code = "teste_hub"
        name = "Produto Teste Hub"
        price = 15.50
        description = "Produto para teste do Hub"
    }
    Invoke-RestMethod -Uri "http://127.0.0.1:8082/products" -Method POST -Body ($product | ConvertTo-Json) -ContentType "application/json"
    Write-Host "   ✓ Hub Service OK! Produto registrado" -ForegroundColor Green
} catch {
    Write-Host "   ✗ Erro no Hub: $($_.Exception.Message)" -ForegroundColor Red
}

# Iniciar CDs
Write-Host ""
Write-Host "3. Iniciando Centros de Distribuicao..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_alpha 8083" -WindowStyle Minimized
Start-Sleep -Seconds 3
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_beta 8084" -WindowStyle Minimized
Start-Sleep -Seconds 3
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_gamma 8085" -WindowStyle Minimized
Start-Sleep -Seconds 8

# Testar CDs
Write-Host "   Testando CDs..." -ForegroundColor Cyan

# CD Alpha
try {
    $cd_alpha = Invoke-RestMethod -Uri "http://127.0.0.1:8083/inventory/garrafas" -Method GET
    Write-Host "   ✓ CD Alpha OK! Garrafas: $($cd_alpha.quantity) unidades" -ForegroundColor Green
} catch {
    Write-Host "   ✗ Erro no CD Alpha: $($_.Exception.Message)" -ForegroundColor Red
}

# CD Beta
try {
    $cd_beta = Invoke-RestMethod -Uri "http://127.0.0.1:8084/inventory/cadernos" -Method GET
    Write-Host "   ✓ CD Beta OK! Cadernos: $($cd_beta.quantity) unidades" -ForegroundColor Green
} catch {
    Write-Host "   ✗ Erro no CD Beta: $($_.Exception.Message)" -ForegroundColor Red
}

# CD Gamma
try {
    $cd_gamma = Invoke-RestMethod -Uri "http://127.0.0.1:8085/inventory/celulares" -Method GET
    Write-Host "   ✓ CD Gamma OK! Celulares: $($cd_gamma.quantity) unidades" -ForegroundColor Green
} catch {
    Write-Host "   ✗ Erro no CD Gamma: $($_.Exception.Message)" -ForegroundColor Red
}

# Testar consulta de disponibilidade no Hub
Write-Host ""
Write-Host "4. Testando consulta de disponibilidade..." -ForegroundColor Yellow
try {
    $who_has = Invoke-RestMethod -Uri "http://127.0.0.1:8082/who_has_product/celulares/5" -Method GET
    Write-Host "   ✓ Consulta OK! CDs com celulares: $($who_has.Count)" -ForegroundColor Green
    foreach ($cd in $who_has) {
        Write-Host "     - $($cd.cd_id): $($cd.quantity_available) unidades" -ForegroundColor White
    }
} catch {
    Write-Host "   ✗ Erro na consulta: $($_.Exception.Message)" -ForegroundColor Red
}

# Verificar registros atualizados
Write-Host ""
Write-Host "5. Verificando registros atualizados..." -ForegroundColor Yellow
try {
    $services = Invoke-RestMethod -Uri "http://127.0.0.1:8080/lookup_all" -Method GET
    Write-Host "   ✓ CDs registrados: $($services.Count)" -ForegroundColor Green
    foreach ($service in $services) {
        Write-Host "     - $($service.id): $($service.ip):$($service.port)" -ForegroundColor White
    }
} catch {
    Write-Host "   ✗ Erro ao verificar registros: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== TESTE CONCLUIDO ===" -ForegroundColor Green
Write-Host "Todos os servicos estao rodando:" -ForegroundColor White
Write-Host "  - Service Discovery: http://127.0.0.1:8080" -ForegroundColor White
Write-Host "  - Hub Service: http://127.0.0.1:8082" -ForegroundColor White
Write-Host "  - CD Alpha: http://127.0.0.1:8083" -ForegroundColor White
Write-Host "  - CD Beta: http://127.0.0.1:8084" -ForegroundColor White
Write-Host "  - CD Gamma: http://127.0.0.1:8085" -ForegroundColor White
Write-Host ""
Write-Host "Pressione qualquer tecla para encerrar todos os servicos..." -ForegroundColor Red
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

# Encerrar servicos
Write-Host "Encerrando servicos..." -ForegroundColor Yellow
Get-Process | Where-Object {$_.ProcessName -eq "cargo"} | Stop-Process -Force
Write-Host "Servicos encerrados!" -ForegroundColor Green 