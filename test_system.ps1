# Script de teste para o sistema distribuído de inventário
# Este script inicia todos os serviços e faz testes básicos

Write-Host "=== SISTEMA DISTRIBUÍDO DE INVENTÁRIO - TESTE COMPLETO ===" -ForegroundColor Green
Write-Host ""

# Função para aguardar um serviço estar disponível
function Wait-ForService {
    param($Url, $ServiceName)
    Write-Host "Aguardando $ServiceName estar disponível em $Url..." -ForegroundColor Yellow
    $maxAttempts = 30
    $attempt = 0
    
    while ($attempt -lt $maxAttempts) {
        try {
            $response = Invoke-WebRequest -Uri $Url -Method GET -TimeoutSec 5 -ErrorAction SilentlyContinue
            if ($response.StatusCode -eq 200) {
                Write-Host "$ServiceName está pronto!" -ForegroundColor Green
                return $true
            }
        }
        catch {
            # Ignora erros de conexão
        }
        
        $attempt++
        Start-Sleep -Seconds 2
    }
    
    Write-Host "ERRO: $ServiceName não ficou disponível em $maxAttempts tentativas" -ForegroundColor Red
    return $false
}

# Função para fazer uma requisição HTTP
function Invoke-TestRequest {
    param($Url, $Method = "GET", $Body = $null, $Description)
    
    Write-Host "Testando: $Description" -ForegroundColor Cyan
    try {
        if ($Body) {
            $response = Invoke-RestMethod -Uri $Url -Method $Method -Body ($Body | ConvertTo-Json -Depth 10) -ContentType "application/json"
        } else {
            $response = Invoke-RestMethod -Uri $Url -Method $Method
        }
        Write-Host "✓ Sucesso: $Description" -ForegroundColor Green
        return $response
    }
    catch {
        Write-Host "✗ Erro: $Description - $($_.Exception.Message)" -ForegroundColor Red
        return $null
    }
}

# Iniciar Service Discovery
Write-Host "1. Iniciando Service Discovery..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin service-discovery" -WindowStyle Minimized
Start-Sleep -Seconds 3

# Aguardar Service Discovery estar pronto
if (-not (Wait-ForService "http://127.0.0.1:8080/lookup_all" "Service Discovery")) {
    Write-Host "Falha ao iniciar Service Discovery. Abortando teste." -ForegroundColor Red
    exit 1
}

# Iniciar Hub Service
Write-Host "2. Iniciando Hub Service..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin hub-service" -WindowStyle Minimized
Start-Sleep -Seconds 3

# Aguardar Hub Service estar pronto
if (-not (Wait-ForService "http://127.0.0.1:8082/products" "Hub Service")) {
    Write-Host "Falha ao iniciar Hub Service. Abortando teste." -ForegroundColor Red
    exit 1
}

# Iniciar CDs
Write-Host "3. Iniciando Centros de Distribuição..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_alpha 8083" -WindowStyle Minimized
Start-Sleep -Seconds 2
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_beta 8084" -WindowStyle Minimized
Start-Sleep -Seconds 2
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_gamma 8085" -WindowStyle Minimized
Start-Sleep -Seconds 5

# Aguardar CDs estarem prontos
Write-Host "Aguardando CDs estarem prontos..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

Write-Host ""
Write-Host "=== INICIANDO TESTES DE FUNCIONALIDADE ===" -ForegroundColor Green
Write-Host ""

# Teste 1: Verificar se os CDs se registraram no Service Discovery
Write-Host "Teste 1: Verificando registro dos CDs no Service Discovery" -ForegroundColor Cyan
$services = Invoke-TestRequest "http://127.0.0.1:8080/lookup_all" "GET" $null "Listar todos os serviços registrados"
if ($services) {
    Write-Host "Serviços registrados: $($services.Count)" -ForegroundColor Green
    foreach ($service in $services) {
        Write-Host "  - $($service.id): $($service.ip):$($service.port)" -ForegroundColor White
    }
}

# Teste 2: Verificar inventário inicial dos CDs
Write-Host ""
Write-Host "Teste 2: Verificando inventário inicial dos CDs" -ForegroundColor Cyan

$cd_alpha_inventory = Invoke-TestRequest "http://127.0.0.1:8083/inventory/garrafas" "GET" $null "Inventário de garrafas no CD Alpha"
if ($cd_alpha_inventory) {
    Write-Host "CD Alpha - Garrafas: $($cd_alpha_inventory.quantity) unidades" -ForegroundColor Green
}

$cd_beta_inventory = Invoke-TestRequest "http://127.0.0.1:8084/inventory/cadernos" "GET" $null "Inventário de cadernos no CD Beta"
if ($cd_beta_inventory) {
    Write-Host "CD Beta - Cadernos: $($cd_beta_inventory.quantity) unidades" -ForegroundColor Green
}

$cd_gamma_inventory = Invoke-TestRequest "http://127.0.0.1:8085/inventory/celulares" "GET" $null "Inventário de celulares no CD Gamma"
if ($cd_gamma_inventory) {
    Write-Host "CD Gamma - Celulares: $($cd_gamma_inventory.quantity) unidades" -ForegroundColor Green
}

# Teste 3: Registrar produtos no Hub
Write-Host ""
Write-Host "Teste 3: Registrando produtos no Hub" -ForegroundColor Cyan

$product1 = @{
    code = "laptop"
    name = "Laptop Dell Inspiron"
    price = 3500.00
    description = "Laptop para trabalho e estudos"
}

$product2 = @{
    code = "mouse"
    name = "Mouse Wireless"
    price = 89.90
    description = "Mouse sem fio ergonômico"
}

Invoke-TestRequest "http://127.0.0.1:8082/products" "POST" $product1 "Registrar laptop no Hub"
Invoke-TestRequest "http://127.0.0.1:8082/products" "POST" $product2 "Registrar mouse no Hub"

# Teste 4: Consultar produtos no Hub
Write-Host ""
Write-Host "Teste 4: Consultando produtos no Hub" -ForegroundColor Cyan

$laptop_info = Invoke-TestRequest "http://127.0.0.1:8082/products/laptop" "GET" $null "Consultar informações do laptop no Hub"
if ($laptop_info) {
    Write-Host "Laptop registrado: $($laptop_info.name) - R$ $($laptop_info.price)" -ForegroundColor Green
}

# Teste 5: Testar transferência de produtos entre CDs
Write-Host ""
Write-Host "Teste 5: Testando transferência de produtos entre CDs" -ForegroundColor Cyan

# CD Alpha vai pedir canetas (que ele não tem, mas CD Gamma tem)
Write-Host "CD Alpha solicitando canetas (que ele não tem)..." -ForegroundColor Yellow
Start-Sleep -Seconds 3

# Verificar se a transferência aconteceu
$cd_alpha_canetas = Invoke-TestRequest "http://127.0.0.1:8083/inventory/canetas" "GET" $null "Verificar se CD Alpha recebeu canetas"
if ($cd_alpha_canetas) {
    Write-Host "CD Alpha agora tem $($cd_alpha_canetas.quantity) canetas" -ForegroundColor Green
} else {
    Write-Host "CD Alpha ainda não tem canetas" -ForegroundColor Yellow
}

# Teste 6: Consultar quem tem um produto específico
Write-Host ""
Write-Host "Teste 6: Consultando quem tem celulares" -ForegroundColor Cyan

$who_has_celulares = Invoke-TestRequest "http://127.0.0.1:8082/who_has_product/celulares/5" "GET" $null "Consultar quem tem pelo menos 5 celulares"
if ($who_has_celulares) {
    Write-Host "CDs com celulares:" -ForegroundColor Green
    foreach ($cd in $who_has_celulares) {
        Write-Host "  - $($cd.cd_id): $($cd.quantity_available) unidades" -ForegroundColor White
    }
}

Write-Host ""
Write-Host "=== TESTE CONCLUÍDO ===" -ForegroundColor Green
Write-Host "Todos os serviços estão rodando:" -ForegroundColor White
Write-Host "  - Service Discovery: http://127.0.0.1:8080" -ForegroundColor White
Write-Host "  - Hub Service: http://127.0.0.1:8082" -ForegroundColor White
Write-Host "  - CD Alpha: http://127.0.0.1:8083" -ForegroundColor White
Write-Host "  - CD Beta: http://127.0.0.1:8084" -ForegroundColor White
Write-Host "  - CD Gamma: http://127.0.0.1:8085" -ForegroundColor White
Write-Host ""
Write-Host "Para parar todos os serviços, feche as janelas do PowerShell ou pressione Ctrl+C" -ForegroundColor Yellow
Write-Host ""

# Manter o script rodando para que os serviços continuem ativos
Write-Host "Pressione qualquer tecla para encerrar todos os serviços..." -ForegroundColor Red
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

# Encerrar todos os processos do cargo
Write-Host "Encerrando todos os serviços..." -ForegroundColor Yellow
Get-Process | Where-Object {$_.ProcessName -eq "cargo" -or $_.ProcessName -eq "cd-service" -or $_.ProcessName -eq "hub-service" -or $_.ProcessName -eq "service-discovery"} | Stop-Process -Force
Write-Host "Serviços encerrados!" -ForegroundColor Green 