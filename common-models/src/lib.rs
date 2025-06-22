use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Representa um produto no catálogo ou no inventário de um Centro de Distribuição (CD).
///
/// Esta é uma estrutura versátil usada em múltiplos contextos:
/// - No `Hub Service`, ela define um tipo de produto no catálogo geral.
/// - Nos `CD Services`, ela representa o estoque de um produto específico.
/// - Em transferências, ela descreve o item que está sendo movido.
///
/// # Attributes
///
/// * `#[derive(Debug, Serialize, Deserialize, Clone)]`:
///   - `Debug`: Permite a formatação da struct para logs e depuração.
///   - `Serialize`, `Deserialize`: Habilita a conversão para e de JSON para comunicação via API.
///   - `Clone`: Permite a criação de cópias da estrutura.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    /// O código único que identifica o produto (ex: "celular", "laptop").
    pub code: String,
    /// O nome descritivo do produto (ex: "Smartphone Modelo X").
    pub name: String,
    /// O preço unitário do produto.
    pub price: f64,
    /// A quantidade do produto em estoque.
    ///
    /// Este campo é opcional (`Option<u32>`) porque um produto no catálogo do Hub não possui
    /// uma quantidade associada, mas ao representar o estoque em um CD, a quantidade é essencial.
    ///
    /// * `#[serde(skip_serializing_if = "Option::is_none")]`: Um atributo do `serde` que
    ///   instrui o serializador a omitir este campo do JSON de saída se o seu valor for `None`.
    ///   Isso mantém os payloads da API limpos e eficientes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u32>,
}

/// Contém as informações de registro e estado de um serviço na rede.
///
/// Usada por um `CD Service` para se registrar no `Service Discovery` e para que o
/// `Service Discovery` armazene o estado de todos os serviços ativos, incluindo o
/// último sinal de vida (heartbeat) de cada um.
///
/// # Attributes
///
/// * `#[derive(Debug, Serialize, Deserialize, Clone)]`:
///   - `Debug`, `Serialize`, `Deserialize`, `Clone`: Ver documentação da struct `Product`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceInfo {
    /// O identificador textual e único do serviço (ex: "cd_alpha").
    pub id: String,
    /// O endereço IP onde o serviço pode ser acessado.
    pub ip: String,
    /// A porta de rede na qual o serviço está escutando.
    pub port: u16,
    /// Timestamp do último heartbeat recebido, crucial para a detecção de falhas.
    ///
    /// * `#[serde(default = "Utc::now")]`: Atributo do `serde` que define o valor deste
    ///   campo como o tempo atual caso ele não seja fornecido durante a desserialização.
    #[serde(default = "Utc::now")]
    pub last_heartbeat: DateTime<Utc>,
}

/// Uma visão simplificada da informação de um serviço, usada para respostas de busca.
///
/// Quando um serviço pede ao `Service Discovery` o endereço de outro (`GET /lookup/{id}`),
/// esta estrutura é retornada. Ela contém apenas o necessário para estabelecer uma
/// comunicação, omitindo dados internos como o `last_heartbeat`.
///
/// # Attributes
///
/// * `#[derive(Debug, Serialize, Deserialize, Clone)]`:
///   - `Debug`, `Serialize`, `Deserialize`, `Clone`: Ver documentação da struct `Product`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceInfoLookup {
    /// O identificador único do serviço.
    pub id: String,
    /// O endereço IP do serviço.
    pub ip: String,
    /// A porta de rede do serviço.
    pub port: u16,
}

/// Representa a disponibilidade de um produto em um CD específico.
///
/// É o principal componente na resposta do endpoint `GET /who_has_product` do Hub,
/// informando ao solicitante quais CDs podem atender a um pedido.
///
/// # Attributes
///
/// * `#[derive(Debug, Serialize, Deserialize)]`:
///   - `Debug`, `Serialize`, `Deserialize`: Ver documentação da struct `Product`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductAvailability {
    /// O ID do `CD Service` que possui o produto disponível.
    pub cd_id: String,
    /// A quantidade total que o CD reportou ter em estoque.
    pub quantity_available: u32,
    /// Detalhes canônicos do produto, geralmente vindos do catálogo do Hub.
    pub product_info: Product,
}

/// Define o corpo da requisição para solicitar uma transferência de produto entre CDs.
///
/// Esta estrutura é enviada como payload de uma requisição `POST /transfer_product`
/// de um CD para outro, formalizando o pedido de transferência P2P.
///
/// # Attributes
///
/// * `#[derive(Debug, Serialize, Deserialize)]`:
///   - `Debug`, `Serialize`, `Deserialize`: Ver documentação da struct `Product`.
#[derive(Debug, Serialize, Deserialize)]
pub struct TransferRequest {
    /// O código do produto a ser transferido.
    pub product_code: String,
    /// A quantidade de unidades do produto solicitada.
    pub quantity: u32,
    /// O ID do CD que está solicitando a transferência, importante para logs e rastreabilidade.
    pub requester_cd_id: String,
}
