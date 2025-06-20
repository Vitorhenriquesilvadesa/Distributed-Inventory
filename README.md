# Sistema Distribuído de Inventário - Rust

## 📋 Visão Geral

Este projeto implementa um sistema distribuído de inventário usando Rust e Actix-web, demonstrando conceitos avançados de sistemas distribuídos como Service Discovery, Heartbeat, Comunicação P2P e Coordenação Centralizada.

### 🎯 Objetivos do Projeto

- **Service Discovery**: Registro dinâmico de Centros de Distribuição (CDs)
- **Inventário Distribuído**: Cada CD gerencia seu próprio inventário
- **Transferência de Produtos**: CDs se comunicam diretamente para transferir produtos
- **Coordenação Centralizada**: Hub Service coordena consultas de disponibilidade
- **Detecção de Falhas**: Heartbeat para detectar CDs offline

## 🏗️ Arquitetura do Sistema

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CD Alpha      │    │   CD Beta       │    │   CD Gamma      │
│   (Porta 8083)  │    │   (Porta 8084)  │    │   (Porta 8085)  │
│                 │    │                 │    │                 │
│ • Inventário    │    │ • Inventário    │    │ • Inventário    │
│ • Transferência │    │ • Transferência │    │ • Transferência │
│ • Heartbeat     │    │ • Heartbeat     │    │ • Heartbeat     │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌────────────┴─────────────┐
                    │                          │
          ┌─────────▼─────────┐    ┌───────────▼──────────┐
          │ Service Discovery │    │    Hub Service       │
          │   (Porta 8080)    │    │    (Porta 8082)      │
          │                   │    │                      │
          │ • Registro CDs    │    │ • Catálogo Produtos  │
          │ • Heartbeat       │    │ • Consulta CD        │
          │ • Lookup          │    │ • Coordenação        │
          └───────────────────┘    └──────────────────────┘
```

## 📦 Componentes do Sistema

### 1. Service Discovery (Porta 8080)
**Função**: Central de registro e gerenciamento de todos os CDs ativos

**Responsabilidades**:
- Registro automático de novos CDs
- Gerenciamento de heartbeat (batimentos cardíacos)
- Detecção de CDs offline
- Lookup de CDs por ID
- Listagem de todos os CDs ativos

### 2. Hub Service (Porta 8082)
**Função**: Central de coordenação e catálogo de produtos

**Responsabilidades**:
- Gerenciamento de catálogo de produtos
- Coordenação de consultas de disponibilidade
- Consulta a todos os CDs para encontrar produtos
- Retorno de lista de CDs que possuem determinado produto

### 3. CD Service (Portas 8083, 8084, 8085)
**Função**: Centros de distribuição que gerenciam inventário local

**Responsabilidades**:
- Gerenciamento de inventário local
- Registro automático no Service Discovery
- Envio de heartbeat a cada 10 segundos
- Resposta a consultas do Hub sobre disponibilidade
- Transferência de produtos para outros CDs
- Recebimento de produtos de outros CDs
- Solicitação de produtos quando não possui quantidade suficiente

### 4. Common Models
**Função**: Estruturas de dados compartilhadas entre todos os serviços

**Estruturas**:
- `Product`: Informações de produto (código, nome, preço, quantidade)
- `ServiceInfo`: Informações de serviço (ID, IP, porta, heartbeat)
- `TransferRequest`: Solicitação de transferência
- `ProductAvailability`: Disponibilidade de produto em um CD

## 🔌 Endpoints da API

### Service Discovery (http://127.0.0.1:8080)

#### POST /register
**Descrição**: Registra um novo CD no sistema

**Formato da Requisição**:
```json
{
  "id": "cd_alpha",
  "ip": "127.0.0.1",
  "port": 8083,
  "last_heartbeat": "2025-06-20T00:00:00.693032Z"
}
```

**Resposta**:
```json
{
  "message": "Service registered successfully"
}
```

#### GET /lookup/{id}
**Descrição**: Busca informações de um CD específico

**Parâmetros**:
- `id`: ID do CD (ex: "cd_alpha")

**Resposta**:
```json
{
  "id": "cd_alpha",
  "ip": "127.0.0.1",
  "port": 8083,
  "last_heartbeat": "2025-06-20T00:00:00.693032Z"
}
```

#### GET /lookup_all
**Descrição**: Lista todos os CDs registrados

**Resposta**:
```json
[
  {
    "id": "cd_alpha",
    "ip": "127.0.0.1",
    "port": 8083,
    "last_heartbeat": "2025-06-20T00:00:00.693032Z"
  },
  {
    "id": "cd_beta",
    "ip": "127.0.0.1",
    "port": 8084,
    "last_heartbeat": "2025-06-20T00:02:19.676859700Z"
  }
]
```

#### POST /heartbeat/{id}
**Descrição**: Atualiza o heartbeat de um CD

**Parâmetros**:
- `id`: ID do CD

**Resposta**:
```json
{
  "message": "Heartbeat updated"
}
```

### Hub Service (http://127.0.0.1:8082)

#### POST /products
**Descrição**: Registra um novo produto no catálogo

**Formato da Requisição**:
```json
{
  "code": "laptop",
  "name": "Laptop Dell Inspiron",
  "price": 3500.00,
  "description": "Laptop para trabalho e estudos"
}
```

**Resposta**:
```json
{
  "message": "Product registered successfully"
}
```

#### GET /products/{code}
**Descrição**: Consulta informações de um produto específico

**Parâmetros**:
- `code`: Código do produto (ex: "laptop")

**Resposta**:
```json
{
  "code": "laptop",
  "name": "Laptop Dell Inspiron",
  "price": 3500.00,
  "description": "Laptop para trabalho e estudos"
}
```

#### GET /who_has_product/{code}/{quantity}
**Descrição**: Consulta quais CDs possuem determinado produto

**Parâmetros**:
- `code`: Código do produto (ex: "celulares")
- `quantity`: Quantidade necessária (ex: 5)

**Resposta**:
```json
[
  {
    "cd_id": "cd_gamma",
    "quantity_available": 15,
    "product_info": {
      "code": "celulares",
      "name": "Smartphones X",
      "price": 1200.0,
      "quantity": 15
    }
  },
  {
    "cd_id": "cd_alpha",
    "quantity_available": 10,
    "product_info": {
      "code": "celulares",
      "name": "Smartphones X",
      "price": 1200.0,
      "quantity": 10
    }
  }
]
```

### CD Service (http://127.0.0.1:8083, 8084, 8085)

#### GET /inventory/{product_code}
**Descrição**: Consulta o inventário de um produto específico

**Parâmetros**:
- `product_code`: Código do produto (ex: "garrafas")

**Resposta**:
```json
{
  "code": "garrafas",
  "name": "Garrafas de Água",
  "price": 2.50,
  "quantity": 50
}
```

#### POST /transfer_product
**Descrição**: Transfere produtos para outro CD

**Formato da Requisição**:
```json
{
  "product_code": "celulares",
  "quantity": 5,
  "requester_cd_id": "cd_alpha"
}
```

**Resposta**:
```json
{
  "message": "Transfer successful"
}
```

#### POST /receive_product
**Descrição**: Recebe produtos de outro CD

**Formato da Requisição**:
```json
{
  "code": "celulares",
  "name": "Smartphones X",
  "price": 1200.0,
  "quantity": 5
}
```

**Resposta**:
```json
{
  "message": "Product received successfully"
}
```

## 🚀 Como Executar

### Pré-requisitos
- Rust (versão 1.70 ou superior)
- Cargo (gerenciador de pacotes do Rust)

### 1. Clone o Repositório
```bash
git clone <url-do-repositorio>
cd distributed-inventory
```

### 2. Compile o Projeto
```bash
cargo build
```

### 3. Execute os Serviços

#### Opção A: Script Automático
```powershell
.\test_final.ps1
```

#### Opção B: Manual (Terminais Separados)

**Terminal 1 - Service Discovery**:
```bash
cargo run --bin service-discovery
```

**Terminal 2 - Hub Service**:
```bash
cargo run --bin hub-service
```

**Terminal 3 - CD Alpha**:
```bash
cargo run --bin cd-service cd_alpha 8083
```

**Terminal 4 - CD Beta**:
```bash
cargo run --bin cd-service cd_beta 8084
```

**Terminal 5 - CD Gamma**:
```bash
cargo run --bin cd-service cd_gamma 8085
```

## 📊 Inventário Inicial dos CDs

### CD Alpha (Porta 8083)
- **Garrafas**: 50 unidades (R$ 2,50 cada)
- **Celulares**: 10 unidades (R$ 1.200,00 cada)

### CD Beta (Porta 8084)
- **Garrafas**: 30 unidades (R$ 2,50 cada)
- **Cadernos**: 100 unidades (R$ 8,00 cada)

### CD Gamma (Porta 8085)
- **Celulares**: 15 unidades (R$ 1.200,00 cada)
- **Canetas**: 200 unidades (R$ 1,50 cada)

## 🔄 Fluxo de Funcionamento

### 1. Inicialização do Sistema
```
1. Service Discovery inicia na porta 8080
2. Hub Service inicia na porta 8082
3. CDs iniciam e se registram automaticamente no Service Discovery
4. CDs começam a enviar heartbeat a cada 10 segundos
```

### 2. Solicitação de Produto
```
Cenário: CD Alpha precisa de 12 celulares, mas só tem 10

1. CD Alpha verifica inventário local: tem 10 celulares
2. CD Alpha calcula necessidade: precisa de 2 celulares adicionais
3. CD Alpha consulta Hub: "Quem tem 2 celulares?"
4. Hub consulta todos os CDs registrados
5. CD Gamma responde: "Tenho 15 celulares"
6. Hub retorna para CD Alpha: "CD Gamma tem 15 celulares"
7. CD Alpha consulta Service Discovery: "Qual o IP/porta do CD Gamma?"
8. Service Discovery retorna: "CD Gamma está em 127.0.0.1:8085"
9. CD Alpha envia requisição para CD Gamma: "Transfira 2 celulares"
10. CD Gamma transfere 2 celulares para CD Alpha
11. CD Alpha atualiza seu inventário: agora tem 12 celulares
```

### 3. Heartbeat Contínuo
```
A cada 10 segundos:
- CD Alpha → Service Discovery: "Ainda estou vivo!"
- CD Beta → Service Discovery: "Ainda estou vivo!"
- CD Gamma → Service Discovery: "Ainda estou vivo!"

Service Discovery:
- Atualiza timestamp do último heartbeat
- Remove CDs que não enviaram heartbeat por mais de 30 segundos
```

## 🧪 Testes e Demonstração

### Script de Teste Automático
```powershell
.\test_final.ps1
```

Este script:
1. Inicia todos os serviços automaticamente
2. Aguarda cada serviço estar pronto
3. Testa todas as funcionalidades
4. Mostra resultados dos testes
5. Mantém serviços rodando para demonstração

### Testes Manuais

#### 1. Verificar Registro dos CDs
```powershell
Invoke-RestMethod -Uri "http://127.0.0.1:8080/lookup_all" -Method GET
```

#### 2. Consultar Inventário de um CD
```powershell
Invoke-RestMethod -Uri "http://127.0.0.1:8083/inventory/garrafas" -Method GET
```

#### 3. Registrar Produto no Hub
```powershell
$product = @{
    code = "laptop"
    name = "Laptop Dell"
    price = 3500.00
    description = "Laptop para trabalho"
}
Invoke-RestMethod -Uri "http://127.0.0.1:8082/products" -Method POST -Body ($product | ConvertTo-Json) -ContentType "application/json"
```

#### 4. Consultar Quem Tem um Produto
```powershell
Invoke-RestMethod -Uri "http://127.0.0.1:8082/who_has_product/celulares/5" -Method GET
```

## 📹 Gravação de Vídeo de Documentação

### Roteiro Sugerido

#### 1. Introdução (2-3 minutos)
- Apresentação do projeto
- Explicação dos objetivos
- Visão geral da arquitetura

#### 2. Demonstração da Arquitetura (3-4 minutos)
- Mostrar diagrama da arquitetura
- Explicar cada componente
- Destacar as melhorias implementadas

#### 3. Execução do Sistema (5-6 minutos)
- Executar `.\test_final.ps1`
- Mostrar logs em tempo real
- Explicar o que está acontecendo em cada etapa

#### 4. Demonstração dos Endpoints (8-10 minutos)
- **Service Discovery**:
  - Mostrar registro automático dos CDs
  - Demonstrar heartbeat
  - Testar lookup de CDs
- **Hub Service**:
  - Registrar produto no catálogo
  - Consultar produto
  - Testar consulta de disponibilidade
- **CD Services**:
  - Consultar inventário de cada CD
  - Demonstrar transferência de produtos
  - Mostrar logs de comunicação

#### 5. Cenário Completo (5-6 minutos)
- Simular CD Alpha precisando de produtos
- Mostrar todo o fluxo de transferência
- Demonstrar atualização de inventário

#### 6. Funcionalidades Avançadas (3-4 minutos)
- Heartbeat e detecção de falhas
- Service Discovery dinâmico
- Comunicação P2P entre CDs

#### 7. Conclusão (2-3 minutos)
- Resumo das funcionalidades
- Destaque das melhorias
- Demonstração de escalabilidade

### Pontos Importantes para o Vídeo

#### 1. Preparação
- Ter todos os terminais organizados
- Scripts de teste prontos
- Exemplos de requisições preparados

#### 2. Durante a Gravação
- Explicar cada comando executado
- Mostrar logs em tempo real
- Destacar momentos importantes
- Usar cores diferentes nos terminais

#### 3. Demonstrações Específicas
- **Registro Automático**: Mostrar como CDs se registram
- **Heartbeat**: Explicar o sistema de batimentos cardíacos
- **Transferência**: Demonstrar comunicação P2P
- **Falhas**: Simular CD offline e mostrar detecção

#### 4. Ferramentas Úteis
- **Postman** ou **Insomnia** para testar endpoints
- **Múltiplos terminais** para mostrar logs simultâneos
- **Diagramas** para explicar arquitetura

## 🔧 Configuração e Personalização

### Alterando Portas
Para alterar as portas dos serviços, edite os arquivos:
- `service-discovery/src/main.rs` (linha 25)
- `hub-service/src/main.rs` (linha 15)
- `cd-service/src/main.rs` (linha 20)

### Adicionando Novos CDs
Para adicionar um novo CD:
```bash
cargo run --bin cd-service cd_delta 8086
```

### Modificando Inventário Inicial
Edite o arquivo `cd-service/src/main.rs` nas linhas 30-90 para modificar o inventário inicial dos CDs.

## 🐛 Troubleshooting

### Problemas Comuns

#### 1. Porta já em uso
**Erro**: `Address already in use`
**Solução**: Verifique se não há outro processo usando a porta ou altere a porta

#### 2. CDs não se registram
**Erro**: CDs não aparecem no Service Discovery
**Solução**: Verifique se o Service Discovery está rodando antes dos CDs

#### 3. Transferência falha
**Erro**: Transferência de produtos não funciona
**Solução**: Verifique se todos os CDs estão rodando e se o Hub está funcionando

#### 4. Heartbeat não funciona
**Erro**: CDs não enviam heartbeat
**Solução**: Verifique conectividade de rede e se o Service Discovery está acessível

## 📈 Melhorias Futuras

### Funcionalidades Sugeridas
1. **Persistência de Dados**: Banco de dados para inventário
2. **Autenticação**: Sistema de autenticação para APIs
3. **Monitoramento**: Dashboard para monitorar o sistema
4. **Load Balancing**: Balanceamento de carga entre CDs
5. **Cache**: Sistema de cache para consultas frequentes
6. **Logs Estruturados**: Logs em formato JSON para análise
7. **Métricas**: Coleta de métricas de performance
8. **Testes Automatizados**: Suite completa de testes

### Escalabilidade
- **Horizontal**: Adicionar mais CDs facilmente
- **Vertical**: Melhorar performance de cada serviço
- **Geográfica**: Distribuir CDs em diferentes localizações

## 📚 Referências

- [Actix-web Documentation](https://actix.rs/docs/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Documentation](https://tokio.rs/docs/)
- [Reqwest Documentation](https://docs.rs/reqwest/)

## 👥 Autores

Este projeto foi desenvolvido como trabalho de Sistemas Distribuídos, demonstrando conceitos avançados de arquitetura distribuída usando Rust.

---

**Sistema Distribuído de Inventário** - Uma implementação robusta e profissional de um sistema distribuído usando Rust e Actix-web, demonstrando Service Discovery, Heartbeat, Comunicação P2P e Coordenação Centralizada. 