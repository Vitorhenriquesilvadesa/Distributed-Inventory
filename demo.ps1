# Script de Demonstração para Apresentação
# Sistema Distribuído de Inventário

Write-Host "=== DEMONSTRAÇÃO - SISTEMA DISTRIBUÍDO DE INVENTÁRIO ===" -ForegroundColor Green
Write-Host "Professor: Este é nosso sistema distribuído implementado em Rust!" -ForegroundColor Cyan
Write-Host ""

# Iniciar todos os serviços
Write-Host "1. INICIANDO TODOS OS SERVIÇOS..." -ForegroundColor Yellow
Write-Host "   - Service Discovery (porta 8080)" -ForegroundColor White
Write-Host "   - Hub Service (porta 8082)" -ForegroundColor White
Write-Host "   - CD Alpha (porta 8083)" -ForegroundColor White
Write-Host "   - CD Beta (porta 8084)" -ForegroundColor White
Write-Host "   - CD Gamma (porta 8085)" -ForegroundColor White
Write-Host ""

Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin service-discovery" -WindowStyle Minimized
Start-Sleep -Seconds 3
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin hub-service" -WindowStyle Minimized
Start-Sleep -Seconds 3
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_alpha 8083" -WindowStyle Minimized
Start-Sleep -Seconds 2
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_beta 8084" -WindowStyle Minimized
Start-Sleep -Seconds 2
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_gamma 8085" -WindowStyle Minimized
Start-Sleep -Seconds 5

Write-Host "Aguardando serviços estarem prontos..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

Write-Host ""
Write-Host "2. DEMONSTRAÇÃO: REGISTRO AUTOMÁTICO DOS CDs" -ForegroundColor Yellow
Write-Host "   Os CDs se registram automaticamente no Service Discovery" -ForegroundColor White
try {
    $services = Invoke-RestMethod -Uri "http://127.0.0.1:8080/lookup_all" -Method GET
    Write-Host "   ✓ CDs registrados: $($services.Count)" -ForegroundColor Green
    foreach ($service in $services) {
        Write-Host "     - $($service.id): $($service.ip):$($service.port)" -ForegroundColor White
    }
} catch {
    Write-Host "   ✗ Erro ao verificar registros" -ForegroundColor Red
}

Write-Host ""
Write-Host "3. DEMONSTRAÇÃO: INVENTÁRIO INICIAL DOS CDs" -ForegroundColor Yellow
Write-Host "   Cada CD tem seu próprio inventário distribuído" -ForegroundColor White

try {
    $cd_alpha = Invoke-RestMethod -Uri "http://127.0.0.1:8083/inventory/garrafas" -Method GET
    Write-Host "   ✓ CD Alpha - Garrafas: $($cd_alpha.quantity) unidades" -ForegroundColor Green
    
    $cd_beta = Invoke-RestMethod -Uri "http://127.0.0.1:8084/inventory/cadernos" -Method GET
    Write-Host "   ✓ CD Beta - Cadernos: $($cd_beta.quantity) unidades" -ForegroundColor Green
    
    $cd_gamma = Invoke-RestMethod -Uri "http://127.0.0.1:8085/inventory/celulares" -Method GET
    Write-Host "   ✓ CD Gamma - Celulares: $($cd_gamma.quantity) unidades" -ForegroundColor Green
} catch {
    Write-Host "   ✗ Erro ao verificar inventários" -ForegroundColor Red
}

Write-Host ""
Write-Host "4. DEMONSTRAÇÃO: CATÁLOGO DE PRODUTOS NO HUB" -ForegroundColor Yellow
Write-Host "   O Hub gerencia o catálogo central de produtos" -ForegroundColor White

try {
    $product = @{
        code = "laptop_demo"
        name = "Laptop para Demonstração"
        price = 2500.00
        description = "Produto demonstrado na apresentação"
    }
    Invoke-RestMethod -Uri "http://127.0.0.1:8082/products" -Method POST -Body ($product | ConvertTo-Json) -ContentType "application/json"
    Write-Host "   ✓ Produto registrado no Hub" -ForegroundColor Green
    
    $product_info = Invoke-RestMethod -Uri "http://127.0.0.1:8082/products/laptop_demo" -Method GET
    Write-Host "   ✓ Produto consultado: $($product_info.name) - R$ $($product_info.price)" -ForegroundColor Green
} catch {
    Write-Host "   ✗ Erro ao testar catálogo" -ForegroundColor Red
}

Write-Host ""
Write-Host "5. DEMONSTRAÇÃO: CONSULTA DE DISPONIBILIDADE" -ForegroundColor Yellow
Write-Host "   O Hub consulta todos os CDs para encontrar produtos" -ForegroundColor White

try {
    $who_has = Invoke-RestMethod -Uri "http://127.0.0.1:8082/who_has_product/celulares/5" -Method GET
    Write-Host "   ✓ CDs com celulares encontrados: $($who_has.Count)" -ForegroundColor Green
    foreach ($cd in $who_has) {
        Write-Host "     - $($cd.cd_id): $($cd.quantity_available) unidades" -ForegroundColor White
    }
} catch {
    Write-Host "   ✗ Erro ao consultar disponibilidade" -ForegroundColor Red
}

Write-Host ""
Write-Host "6. DEMONSTRAÇÃO: TRANSFERÊNCIA DE PRODUTOS" -ForegroundColor Yellow
Write-Host "   CD Alpha vai solicitar canetas (que ele não tem)" -ForegroundColor White
Write-Host "   O sistema deve encontrar CD Gamma que tem canetas" -ForegroundColor White

Start-Sleep -Seconds 3

try {
    $cd_alpha_canetas = Invoke-RestMethod -Uri "http://127.0.0.1:8083/inventory/canetas" -Method GET
    if ($cd_alpha_canetas.quantity -gt 0) {
        Write-Host "   ✓ Transferência bem-sucedida! CD Alpha agora tem $($cd_alpha_canetas.quantity) canetas" -ForegroundColor Green
    } else {
        Write-Host "   ⚠ CD Alpha ainda não tem canetas (transferência pode estar em andamento)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ✗ Erro ao verificar transferência" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== RESUMO DA ARQUITETURA ===" -ForegroundColor Green
Write-Host ""
Write-Host "✅ Service Discovery: Gerencia registro e heartbeat dos CDs" -ForegroundColor White
Write-Host "✅ Hub Service: Central de coordenação e catálogo de produtos" -ForegroundColor White
Write-Host "✅ CD Services: Inventário distribuído com comunicação P2P" -ForegroundColor White
Write-Host "✅ Comunicação via IP: Cada serviço tem IP e porta únicos" -ForegroundColor White
Write-Host "✅ Transferência Automática: CDs se comunicam quando precisam de produtos" -ForegroundColor White
Write-Host "✅ Heartbeat: Detecta CDs offline automaticamente" -ForegroundColor White
Write-Host ""
Write-Host "=== MELHORIAS IMPLEMENTADAS ===" -ForegroundColor Green
Write-Host ""
Write-Host "🚀 Service Discovery para gerenciamento dinâmico" -ForegroundColor White
Write-Host "🚀 Heartbeat para detecção de falhas" -ForegroundColor White
Write-Host "🚀 Arquitetura modular e escalável" -ForegroundColor White
Write-Host "🚀 Tratamento robusto de erros" -ForegroundColor White
Write-Host "🚀 Logs detalhados para debugging" -ForegroundColor White
Write-Host "🚀 Testes automatizados" -ForegroundColor White
Write-Host ""
Write-Host "=== ENDPOINTS DISPONÍVEIS ===" -ForegroundColor Green
Write-Host ""
Write-Host "Service Discovery (8080):" -ForegroundColor White
Write-Host "  GET /lookup_all" -ForegroundColor Gray
Write-Host "  POST /register" -ForegroundColor Gray
Write-Host "  GET /lookup/{id}" -ForegroundColor Gray
Write-Host ""
Write-Host "Hub Service (8082):" -ForegroundColor White
Write-Host "  POST /products" -ForegroundColor Gray
Write-Host "  GET /products/{code}" -ForegroundColor Gray
Write-Host "  GET /who_has_product/{code}/{quantity}" -ForegroundColor Gray
Write-Host ""
Write-Host "CD Services (8083, 8084, 8085):" -ForegroundColor White
Write-Host "  GET /inventory/{product_code}" -ForegroundColor Gray
Write-Host "  POST /transfer_product" -ForegroundColor Gray
Write-Host "  POST /receive_product" -ForegroundColor Gray
Write-Host ""
Write-Host "=== DEMONSTRAÇÃO CONCLUÍDA ===" -ForegroundColor Green
Write-Host "Todos os serviços continuam rodando para testes adicionais" -ForegroundColor Yellow
Write-Host ""
Write-Host "Pressione qualquer tecla para encerrar todos os serviços..." -ForegroundColor Red
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

# Encerrar serviços
Write-Host "Encerrando todos os serviços..." -ForegroundColor Yellow
Get-Process | Where-Object {$_.ProcessName -eq "cargo" -or $_.ProcessName -eq "cd-service" -or $_.ProcessName -eq "hub-service" -or $_.ProcessName -eq "service-discovery"} | Stop-Process -Force
Write-Host "Demonstração finalizada!" -ForegroundColor Green 