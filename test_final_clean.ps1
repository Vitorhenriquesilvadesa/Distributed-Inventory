Write-Host "=== FINAL TEST - DISTRIBUTED SYSTEM ===" -ForegroundColor Green
Write-Host ""

# Start Service Discovery
Write-Host "1. Starting Service Discovery..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin service-discovery" -WindowStyle Minimized
Start-Sleep -Seconds 8

# Test Service Discovery
Write-Host "   Testing Service Discovery..." -ForegroundColor Cyan
$services = Invoke-RestMethod -Uri "http://127.0.0.1:8080/lookup_all" -Method GET
Write-Host "   Service Discovery OK! Registered CDs: $($services.Count)" -ForegroundColor Green

# Start Hub Service
Write-Host ""
Write-Host "2. Starting Hub Service..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin hub-service" -WindowStyle Minimized
Start-Sleep -Seconds 8

# Test Hub Service
Write-Host "   Testing Hub Service..." -ForegroundColor Cyan
$product = @{
    code = "test_hub"
    name = "Test Product Hub"
    price = 15.50
    description = "Product for Hub test"
}
Invoke-RestMethod -Uri "http://127.0.0.1:8082/products" -Method POST -Body ($product | ConvertTo-Json) -ContentType "application/json"
Write-Host "   Hub Service OK! Product registered" -ForegroundColor Green

# Start CDs
Write-Host ""
Write-Host "3. Starting Distribution Centers..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_alpha 8083" -WindowStyle Minimized
Start-Sleep -Seconds 3
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_beta 8084" -WindowStyle Minimized
Start-Sleep -Seconds 3
Start-Process powershell -ArgumentList "-Command", "cd '$PWD'; cargo run --bin cd-service cd_gamma 8085" -WindowStyle Minimized
Start-Sleep -Seconds 15

# Check if CDs registered with Service Discovery
Write-Host "   Checking CD registration with Service Discovery..." -ForegroundColor Cyan
$services = Invoke-RestMethod -Uri "http://127.0.0.1:8080/lookup_all" -Method GET
Write-Host "   CDs registered in Service Discovery: $($services.Count)" -ForegroundColor Green

# Test CDs
Write-Host "   Testing CDs..." -ForegroundColor Cyan

# CD Alpha
$cd_alpha = Invoke-RestMethod -Uri "http://127.0.0.1:8083/inventory/garrafas" -Method GET
Write-Host "   CD Alpha OK! Bottles: $($cd_alpha.quantity) units" -ForegroundColor Green

# CD Beta
$cd_beta = Invoke-RestMethod -Uri "http://127.0.0.1:8084/inventory/cadernos" -Method GET
Write-Host "   CD Beta OK! Notebooks: $($cd_beta.quantity) units" -ForegroundColor Green

# CD Gamma
$cd_gamma = Invoke-RestMethod -Uri "http://127.0.0.1:8085/inventory/celulares" -Method GET
Write-Host "   CD Gamma OK! Phones: $($cd_gamma.quantity) units" -ForegroundColor Green

# Test product availability query in Hub
Write-Host ""
Write-Host "4. Testing product availability query..." -ForegroundColor Yellow
$who_has = Invoke-RestMethod -Uri "http://127.0.0.1:8082/who_has_product/celulares/5" -Method GET
Write-Host "   Query OK! CDs with phones: $($who_has.Count)" -ForegroundColor Green

# Test product transfer between CDs
Write-Host ""
Write-Host "5. Testing product transfer between CDs..." -ForegroundColor Yellow
Write-Host "   Checking initial inventory..." -ForegroundColor Cyan

# Check initial inventory
$cd_alpha_phones_initial = Invoke-RestMethod -Uri "http://127.0.0.1:8083/inventory/celulares" -Method GET
Write-Host "   CD Alpha - Initial phones: $($cd_alpha_phones_initial.quantity) units" -ForegroundColor White

$cd_gamma_phones_initial = Invoke-RestMethod -Uri "http://127.0.0.1:8085/inventory/celulares" -Method GET
Write-Host "   CD Gamma - Initial phones: $($cd_gamma_phones_initial.quantity) units" -ForegroundColor White

Write-Host "   Waiting for automatic transfers..." -ForegroundColor Cyan
Start-Sleep -Seconds 15

# Test manual transfer
Write-Host "   Testing manual transfer..." -ForegroundColor Cyan
$transfer_request = @{
    product_code = "celulares"
    quantity = 5
    requester_cd_id = "cd_alpha"
}
$transfer_response = Invoke-RestMethod -Uri "http://127.0.0.1:8085/transfer_product" -Method POST -Body ($transfer_request | ConvertTo-Json) -ContentType "application/json"
Write-Host "   Manual transfer response: $transfer_response" -ForegroundColor Green

# Check inventory after transfer
Write-Host "   Checking inventory after transfer..." -ForegroundColor Cyan
$cd_alpha_phones_final = Invoke-RestMethod -Uri "http://127.0.0.1:8083/inventory/celulares" -Method GET
Write-Host "   CD Alpha - Final phones: $($cd_alpha_phones_final.quantity) units" -ForegroundColor White

$cd_gamma_phones_final = Invoke-RestMethod -Uri "http://127.0.0.1:8085/inventory/celulares" -Method GET
Write-Host "   CD Gamma - Final phones: $($cd_gamma_phones_final.quantity) units" -ForegroundColor White

# Check updated registrations
Write-Host ""
Write-Host "6. Checking updated registrations..." -ForegroundColor Yellow
$services = Invoke-RestMethod -Uri "http://127.0.0.1:8080/lookup_all" -Method GET
Write-Host "   Registered CDs: $($services.Count)" -ForegroundColor Green

Write-Host ""
Write-Host "=== TEST COMPLETED ===" -ForegroundColor Green
Write-Host "All services are running:" -ForegroundColor White
Write-Host "  - Service Discovery: http://127.0.0.1:8080" -ForegroundColor White
Write-Host "  - Hub Service: http://127.0.0.1:8082" -ForegroundColor White
Write-Host "  - CD Alpha: http://127.0.0.1:8083" -ForegroundColor White
Write-Host "  - CD Beta: http://127.0.0.1:8084" -ForegroundColor White
Write-Host "  - CD Gamma: http://127.0.0.1:8085" -ForegroundColor White
Write-Host ""
Write-Host "Press any key to stop all services..." -ForegroundColor Red
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

# Stop services
Write-Host "Stopping services..." -ForegroundColor Yellow
Get-Process | Where-Object {$_.ProcessName -eq "cargo"} | Stop-Process -Force
Write-Host "Services stopped!" -ForegroundColor Green 