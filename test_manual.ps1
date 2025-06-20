# Script de teste manual para o sistema distribuído
# Permite testar cada serviço individualmente

Write-Host "=== TESTE MANUAL - SISTEMA DISTRIBUÍDO ===" -ForegroundColor Green
Write-Host ""

Write-Host "Escolha uma opção:" -ForegroundColor Yellow
Write-Host "1. Iniciar Service Discovery (porta 8080)" -ForegroundColor White
Write-Host "2. Iniciar Hub Service (porta 8082)" -ForegroundColor White
Write-Host "3. Iniciar CD Alpha (porta 8083)" -ForegroundColor White
Write-Host "4. Iniciar CD Beta (porta 8084)" -ForegroundColor White
Write-Host "5. Iniciar CD Gamma (porta 8085)" -ForegroundColor White
Write-Host "6. Testar Service Discovery" -ForegroundColor White
Write-Host "7. Testar Hub Service" -ForegroundColor White
Write-Host "8. Testar CD Alpha" -ForegroundColor White
Write-Host "9. Testar CD Beta" -ForegroundColor White
Write-Host "10. Testar CD Gamma" -ForegroundColor White
Write-Host "11. Teste completo de transferência" -ForegroundColor White
Write-Host "0. Sair" -ForegroundColor Red
Write-Host ""

do {
    $choice = Read-Host "Digite sua escolha (0-11)"
    
    switch ($choice) {
        "1" {
            Write-Host "Iniciando Service Discovery..." -ForegroundColor Yellow
            Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin service-discovery"
        }
        "2" {
            Write-Host "Iniciando Hub Service..." -ForegroundColor Yellow
            Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin hub-service"
        }
        "3" {
            Write-Host "Iniciando CD Alpha..." -ForegroundColor Yellow
            Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_alpha 8083"
        }
        "4" {
            Write-Host "Iniciando CD Beta..." -ForegroundColor Yellow
            Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_beta 8084"
        }
        "5" {
            Write-Host "Iniciando CD Gamma..." -ForegroundColor Yellow
            Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_gamma 8085"
        }
        "6" {
            Write-Host "Testando Service Discovery..." -ForegroundColor Cyan
            try {
                $response = Invoke-RestMethod -Uri "http://127.0.0.1:8080/lookup_all" -Method GET
                Write-Host "✓ Service Discovery funcionando! Serviços registrados: $($response.Count)" -ForegroundColor Green
                foreach ($service in $response) {
                    Write-Host "  - $($service.id): $($service.ip):$($service.port)" -ForegroundColor White
                }
            }
            catch {
                Write-Host "✗ Erro ao conectar com Service Discovery: $($_.Exception.Message)" -ForegroundColor Red
            }
        }
        "7" {
            Write-Host "Testando Hub Service..." -ForegroundColor Cyan
            try {
                # Registrar um produto
                $product = @{
                    code = "teste"
                    name = "Produto Teste"
                    price = 10.00
                    description = "Produto para teste"
                }
                $response = Invoke-RestMethod -Uri "http://127.0.0.1:8082/products" -Method POST -Body ($product | ConvertTo-Json) -ContentType "application/json"
                Write-Host "✓ Produto registrado no Hub!" -ForegroundColor Green
                
                # Consultar o produto
                $product_info = Invoke-RestMethod -Uri "http://127.0.0.1:8082/products/teste" -Method GET
                Write-Host "✓ Produto consultado: $($product_info.name) - R$ $($product_info.price)" -ForegroundColor Green
            }
            catch {
                Write-Host "✗ Erro ao testar Hub Service: $($_.Exception.Message)" -ForegroundColor Red
            }
        }
        "8" {
            Write-Host "Testando CD Alpha..." -ForegroundColor Cyan
            try {
                $response = Invoke-RestMethod -Uri "http://127.0.0.1:8083/inventory/garrafas" -Method GET
                Write-Host "✓ CD Alpha funcionando! Garrafas: $($response.quantity) unidades" -ForegroundColor Green
            }
            catch {
                Write-Host "✗ Erro ao testar CD Alpha: $($_.Exception.Message)" -ForegroundColor Red
            }
        }
        "9" {
            Write-Host "Testando CD Beta..." -ForegroundColor Cyan
            try {
                $response = Invoke-RestMethod -Uri "http://127.0.0.1:8084/inventory/cadernos" -Method GET
                Write-Host "✓ CD Beta funcionando! Cadernos: $($response.quantity) unidades" -ForegroundColor Green
            }
            catch {
                Write-Host "✗ Erro ao testar CD Beta: $($_.Exception.Message)" -ForegroundColor Red
            }
        }
        "10" {
            Write-Host "Testando CD Gamma..." -ForegroundColor Cyan
            try {
                $response = Invoke-RestMethod -Uri "http://127.0.0.1:8085/inventory/celulares" -Method GET
                Write-Host "✓ CD Gamma funcionando! Celulares: $($response.quantity) unidades" -ForegroundColor Green
            }
            catch {
                Write-Host "✗ Erro ao testar CD Gamma: $($_.Exception.Message)" -ForegroundColor Red
            }
        }
        "11" {
            Write-Host "Teste completo de transferência..." -ForegroundColor Cyan
            Write-Host "Este teste simula um CD pedindo produtos que não tem para outro CD via Hub" -ForegroundColor Yellow
            
            try {
                # Verificar inventário inicial
                Write-Host "Verificando inventário inicial..." -ForegroundColor Yellow
                $cd_alpha_garrafas = Invoke-RestMethod -Uri "http://127.0.0.1:8083/inventory/garrafas" -Method GET
                Write-Host "CD Alpha tem $($cd_alpha_garrafas.quantity) garrafas" -ForegroundColor White
                
                $cd_gamma_celulares = Invoke-RestMethod -Uri "http://127.0.0.1:8085/inventory/celulares" -Method GET
                Write-Host "CD Gamma tem $($cd_gamma_celulares.quantity) celulares" -ForegroundColor White
                
                # CD Alpha pede celulares (que ele não tem)
                Write-Host "CD Alpha solicitando 5 celulares..." -ForegroundColor Yellow
                Start-Sleep -Seconds 3
                
                # Verificar se a transferência aconteceu
                $cd_alpha_celulares = Invoke-RestMethod -Uri "http://127.0.0.1:8083/inventory/celulares" -Method GET
                if ($cd_alpha_celulares.quantity -gt 0) {
                    Write-Host "✓ Transferência bem-sucedida! CD Alpha agora tem $($cd_alpha_celulares.quantity) celulares" -ForegroundColor Green
                } else {
                    Write-Host "✗ Transferência não aconteceu ou falhou" -ForegroundColor Red
                }
            }
            catch {
                Write-Host "✗ Erro no teste de transferência: $($_.Exception.Message)" -ForegroundColor Red
            }
        }
        "0" {
            Write-Host "Saindo..." -ForegroundColor Green
        }
        default {
            Write-Host "Opção inválida. Digite um número de 0 a 11." -ForegroundColor Red
        }
    }
    
    if ($choice -ne "0") {
        Write-Host ""
        Write-Host "Pressione Enter para continuar..." -ForegroundColor Yellow
        Read-Host
        Clear-Host
        Write-Host "=== TESTE MANUAL - SISTEMA DISTRIBUÍDO ===" -ForegroundColor Green
        Write-Host ""
        Write-Host "Escolha uma opção:" -ForegroundColor Yellow
        Write-Host "1. Iniciar Service Discovery (porta 8080)" -ForegroundColor White
        Write-Host "2. Iniciar Hub Service (porta 8082)" -ForegroundColor White
        Write-Host "3. Iniciar CD Alpha (porta 8083)" -ForegroundColor White
        Write-Host "4. Iniciar CD Beta (porta 8084)" -ForegroundColor White
        Write-Host "5. Iniciar CD Gamma (porta 8085)" -ForegroundColor White
        Write-Host "6. Testar Service Discovery" -ForegroundColor White
        Write-Host "7. Testar Hub Service" -ForegroundColor White
        Write-Host "8. Testar CD Alpha" -ForegroundColor White
        Write-Host "9. Testar CD Beta" -ForegroundColor White
        Write-Host "10. Testar CD Gamma" -ForegroundColor White
        Write-Host "11. Teste completo de transferência" -ForegroundColor White
        Write-Host "0. Sair" -ForegroundColor Red
        Write-Host ""
    }
} while ($choice -ne "0")

Write-Host "Teste manual finalizado!" -ForegroundColor Green 