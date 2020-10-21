//! An incomplete async bitcoind rpc client that support multi-wallet features

use crate::json_rpc;
use ::bitcoin::{
    consensus::encode::serialize_hex, hashes::hex::FromHex, Address, Amount, Network, Transaction,
    Txid,
};
use bitcoin::{consensus::encode, Script};
use reqwest::Url;
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

pub const JSONRPC_VERSION: &str = "1.0";

#[derive(Debug, Clone)]
pub struct Client {
    rpc_client: json_rpc::Client,
}

impl Client {
    pub fn new(url: Url) -> Self {
        Client {
            rpc_client: json_rpc::Client::new(url),
        }
    }

    pub async fn network(&self) -> Result<Network> {
        let blockchain_info = self.blockchain_info().await?;

        Ok(blockchain_info.chain)
    }

    pub async fn median_time(&self) -> Result<u32> {
        let blockchain_info = self.blockchain_info().await?;

        Ok(blockchain_info.mediantime)
    }

    pub async fn block_height(&self) -> Result<u32> {
        let block_height = self
            .rpc_client
            .send::<Vec<()>, _>(json_rpc::Request::new(
                "getblockcount",
                vec![],
                JSONRPC_VERSION.into(),
            ))
            .await?;

        Ok(block_height)
    }

    async fn blockchain_info(&self) -> Result<BlockchainInfo> {
        let blockchain_info = self
            .rpc_client
            .send::<Vec<()>, BlockchainInfo>(json_rpc::Request::new(
                "getblockchaininfo",
                vec![],
                JSONRPC_VERSION.into(),
            ))
            .await?;

        Ok(blockchain_info)
    }

    pub async fn create_wallet(
        &self,
        wallet_name: &str,
        disable_private_keys: Option<bool>,
        blank: Option<bool>,
        passphrase: Option<String>,
        avoid_reuse: Option<bool>,
    ) -> Result<CreateWalletResponse> {
        let response = self
            .rpc_client
            .send(json_rpc::Request::new(
                "createwallet",
                vec![
                    json_rpc::serialize(wallet_name)?,
                    json_rpc::serialize(disable_private_keys)?,
                    json_rpc::serialize(blank)?,
                    json_rpc::serialize(passphrase)?,
                    json_rpc::serialize(avoid_reuse)?,
                ],
                JSONRPC_VERSION.into(),
            ))
            .await?;
        Ok(response)
    }

    pub async fn get_balance(
        &self,
        wallet_name: &str,
        minimum_confirmation: Option<u32>,
        include_watch_only: Option<bool>,
        avoid_reuse: Option<bool>,
    ) -> Result<Amount> {
        let response = self
            .rpc_client
            .send_with_path(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new(
                    "getbalance",
                    vec![
                        json_rpc::serialize('*')?,
                        json_rpc::serialize(minimum_confirmation)?,
                        json_rpc::serialize(include_watch_only)?,
                        json_rpc::serialize(avoid_reuse)?,
                    ],
                    JSONRPC_VERSION.into(),
                ),
            )
            .await?;
        let amount = Amount::from_btc(response)?;
        Ok(amount)
    }

    pub async fn set_hd_seed(
        &self,
        wallet_name: &str,
        new_key_pool: Option<bool>,
        wif_private_key: Option<String>,
    ) -> Result<()> {
        self.rpc_client
            .send_with_path(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new(
                    "sethdseed",
                    vec![
                        json_rpc::serialize(new_key_pool)?,
                        json_rpc::serialize(wif_private_key)?,
                    ],
                    JSONRPC_VERSION.into(),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn get_new_address(
        &self,
        wallet_name: &str,
        label: Option<String>,
        address_type: Option<String>,
    ) -> Result<Address> {
        let address = self
            .rpc_client
            .send_with_path(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new(
                    "getnewaddress",
                    vec![
                        json_rpc::serialize(label)?,
                        json_rpc::serialize(address_type)?,
                    ],
                    JSONRPC_VERSION.into(),
                ),
            )
            .await?;
        Ok(address)
    }

    pub async fn get_wallet_info(&self, wallet_name: &str) -> Result<WalletInfoResponse> {
        let response = self
            .rpc_client
            .send_with_path::<Vec<()>, _>(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new("getwalletinfo", vec![], JSONRPC_VERSION.into()),
            )
            .await?;
        Ok(response)
    }

    pub async fn send_to_address(
        &self,
        wallet_name: &str,
        address: Address,
        amount: Amount,
    ) -> Result<Txid> {
        let txid: String = self
            .rpc_client
            .send_with_path(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new(
                    "sendtoaddress",
                    vec![
                        json_rpc::serialize(address)?,
                        json_rpc::serialize(amount.as_btc())?,
                    ],
                    JSONRPC_VERSION.into(),
                ),
            )
            .await?;
        let txid = Txid::from_hex(&txid)?;

        Ok(txid)
    }

    pub async fn send_raw_transaction(
        &self,
        wallet_name: &str,
        transaction: Transaction,
    ) -> Result<Txid> {
        let txid: String = self
            .rpc_client
            .send_with_path(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new(
                    "sendrawtransaction",
                    vec![serialize_hex(&transaction)],
                    JSONRPC_VERSION.into(),
                ),
            )
            .await?;
        let txid = Txid::from_hex(&txid)?;
        Ok(txid)
    }

    pub async fn get_raw_transaction(&self, wallet_name: &str, txid: Txid) -> Result<Transaction> {
        let hex: String = self
            .rpc_client
            .send_with_path(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new(
                    "getrawtransaction",
                    vec![json_rpc::serialize(txid)?],
                    JSONRPC_VERSION.into(),
                ),
            )
            .await?;
        let bytes: Vec<u8> = FromHex::from_hex(&hex)?;
        let transaction = bitcoin::consensus::encode::deserialize(&bytes)?;

        Ok(transaction)
    }

    pub async fn get_transaction(
        &self,
        wallet_name: &str,
        txid: Txid,
    ) -> Result<WalletTransactionInfo> {
        let res = self
            .rpc_client
            .send_with_path(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new(
                    "gettransaction",
                    vec![json_rpc::serialize(txid)?],
                    JSONRPC_VERSION.into(),
                ),
            )
            .await?;

        Ok(res)
    }

    pub async fn dump_wallet(&self, wallet_name: &str, filename: &std::path::Path) -> Result<()> {
        let _: DumpWalletResponse = self
            .rpc_client
            .send_with_path(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new(
                    "dumpwallet",
                    vec![json_rpc::serialize(filename)?],
                    JSONRPC_VERSION.into(),
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn list_wallets(&self) -> Result<Vec<String>> {
        let wallets: Vec<String> = self
            .rpc_client
            .send::<Vec<()>, _>(json_rpc::Request::new(
                "listwallets",
                vec![],
                JSONRPC_VERSION.into(),
            ))
            .await?;
        Ok(wallets)
    }

    pub async fn derive_addresses(
        &self,
        descriptor: &str,
        range: Option<[u64; 2]>,
    ) -> Result<Vec<Address>> {
        let addresses: Vec<Address> = self
            .rpc_client
            .send(json_rpc::Request::new(
                "deriveaddresses",
                vec![
                    json_rpc::serialize(descriptor)?,
                    json_rpc::serialize(range)?,
                ],
                JSONRPC_VERSION.into(),
            ))
            .await?;
        Ok(addresses)
    }

    pub async fn get_descriptor_info(&self, descriptor: &str) -> Result<GetDescriptorInfoResponse> {
        self.rpc_client
            .send(json_rpc::Request::new(
                "getdescriptorinfo",
                vec![json_rpc::serialize(descriptor)?],
                JSONRPC_VERSION.into(),
            ))
            .await
            .map_err(Into::into)
    }

    pub async fn generate_to_address(
        &self,
        nblocks: u32,
        address: Address,
        max_tries: Option<u32>,
    ) -> Result<Vec<BlockHash>> {
        let response = self
            .rpc_client
            .send(json_rpc::Request::new(
                "generatetoaddress",
                vec![
                    json_rpc::serialize(nblocks)?,
                    json_rpc::serialize(address)?,
                    json_rpc::serialize(max_tries)?,
                ],
                JSONRPC_VERSION.into(),
            ))
            .await?;
        Ok(response)
    }

    pub async fn list_unspent(
        &self,
        wallet_name: &str,
        min_conf: Option<u32>,
        max_conf: Option<u32>,
        addresses: Option<Vec<Address>>,
        include_unsafe: Option<bool>,
    ) -> Result<Vec<Unspent>> {
        let unspents = self
            .rpc_client
            .send_with_path(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new(
                    "listunspent",
                    vec![
                        json_rpc::serialize(min_conf)?,
                        json_rpc::serialize(max_conf)?,
                        json_rpc::serialize(addresses)?,
                        json_rpc::serialize(include_unsafe)?,
                    ],
                    JSONRPC_VERSION.into(),
                ),
            )
            .await?;
        Ok(unspents)
    }

    pub async fn fund_psbt(
        &self,
        wallet_name: &str,
        inputs: &[bitcoincore_rpc_json::CreateRawTransactionInput],
        address: Address,
        amount: Amount,
    ) -> Result<String> {
        let mut outputs_converted = serde_json::Map::new();
        outputs_converted.insert(
            address.to_string(),
            serde_json::Value::from(amount.as_btc()),
        );
        let psbt: bitcoincore_rpc_json::WalletCreateFundedPsbtResult = self
            .rpc_client
            .send_with_path(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new(
                    "walletcreatefundedpsbt",
                    vec![
                        json_rpc::serialize(inputs)?,
                        json_rpc::serialize(outputs_converted)?,
                    ],
                    JSONRPC_VERSION.into(),
                ),
            )
            .await?;
        Ok(psbt.psbt)
    }

    pub async fn join_psbts(&self, wallet_name: &str, psbts: &[String]) -> Result<PsbtBase64> {
        let psbt = self
            .rpc_client
            .send_with_path(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new(
                    "joinpsbts",
                    vec![json_rpc::serialize(psbts)?],
                    JSONRPC_VERSION.into(),
                ),
            )
            .await?;
        Ok(psbt)
    }
    pub async fn wallet_process_psbt(
        &self,
        wallet_name: &str,
        psbt: PsbtBase64,
    ) -> Result<ProcessedPsbt> {
        let psbt = self
            .rpc_client
            .send_with_path(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new(
                    "walletprocesspsbt",
                    vec![json_rpc::serialize(psbt)?],
                    JSONRPC_VERSION.into(),
                ),
            )
            .await?;
        Ok(psbt)
    }
    pub async fn finalize_psbt(
        &self,
        wallet_name: &str,
        psbt: PsbtBase64,
    ) -> Result<FinalizedPsbt> {
        let psbt = self
            .rpc_client
            .send_with_path(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new(
                    "finalizepsbt",
                    vec![json_rpc::serialize(psbt)?],
                    JSONRPC_VERSION.into(),
                ),
            )
            .await?;
        Ok(psbt)
    }

    pub async fn address_info(&self, wallet_name: &str, address: &Address) -> Result<AddressInfo> {
        let address_info = self
            .rpc_client
            .send_with_path(
                format!("/wallet/{}", wallet_name),
                json_rpc::Request::new(
                    "getaddressinfo",
                    vec![json_rpc::serialize(address.to_string())?],
                    JSONRPC_VERSION.into(),
                ),
            )
            .await?;
        Ok(address_info)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("JSON Rpc: ")]
    JsonRpc(#[from] json_rpc::Error),
    #[error("Parse amount: ")]
    ParseAmount(#[from] bitcoin::util::amount::ParseAmountError),
    #[error("Hex decode: ")]
    Hex(#[from] bitcoin::hashes::hex::Error),
    #[error("Bitcoin decode: ")]
    BitcoinDecode(#[from] bitcoin::consensus::encode::Error),
}

#[derive(Debug, Deserialize)]
struct BlockchainInfo {
    chain: Network,
    mediantime: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PsbtBase64(pub String);

impl From<ProcessedPsbt> for PsbtBase64 {
    fn from(processed_psbt: ProcessedPsbt) -> Self {
        Self(processed_psbt.psbt)
    }
}
impl From<String> for PsbtBase64 {
    fn from(base64_string: String) -> Self {
        Self(base64_string)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessedPsbt {
    psbt: String,
    complete: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FinalizedPsbt {
    hex: String,
    complete: bool,
}

impl FinalizedPsbt {
    pub fn transaction(&self) -> Result<Transaction> {
        let data = hex::decode(&self.hex).unwrap();
        let transaction = encode::deserialize(data.as_slice())?;
        Ok(transaction)
    }
}

#[derive(Debug, Deserialize)]
pub struct BlockHash(String);

#[derive(Debug, Deserialize)]
pub struct CreateWalletResponse {
    name: String,
    warning: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WalletInfoResponse {
    #[serde(rename = "walletname")]
    pub wallet_name: String,
    #[serde(rename = "walletversion")]
    pub wallet_version: u32,
    #[serde(rename = "txcount")]
    pub tx_count: u32,
    #[serde(rename = "keypoololdest")]
    pub keypool_oldest: u32,
    #[serde(rename = "keypoolsize_hd_internal")]
    pub keypool_size_hd_internal: u32,
    pub unlocked_until: Option<u32>,
    #[serde(rename = "paytxfee")]
    pub pay_tx_fee: f64,
    #[serde(rename = "hdseedid")]
    pub hd_seed_id: Option<String>, // Hash 160
    pub private_keys_enabled: bool,
    pub avoid_reuse: bool,
    pub scanning: ScanProgress,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct DumpWalletResponse {
    filename: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GetDescriptorInfoResponse {
    pub descriptor: String,
    pub checksum: String,
    #[serde(rename = "isrange")]
    pub is_range: bool,
    #[serde(rename = "issolvable")]
    pub is_solvable: bool,
    #[serde(rename = "hasprivatekeys")]
    pub has_private_keys: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ScanProgress {
    Bool(bool),
    Progress { duration: u32, progress: f64 },
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Unspent {
    #[serde(rename = "txid")]
    pub tx_id: Txid,
    pub vout: u32,
    pub address: Address,
    pub label: String,
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: String,
    pub amount: f64,
    pub confirmations: u64,
    #[serde(rename = "redeemScript")]
    pub redeem_script: Option<String>,
    #[serde(rename = "witnessScript")]
    pub witness_script: Option<String>,
    pub spendable: bool,
    pub solvable: bool,
    pub reused: Option<bool>,
    pub desc: String,
    pub safe: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AddressInfo {
    pub address: Address,
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: Script,
    #[serde(rename = "ismine")]
    pub is_mine: bool,
    pub solvable: bool,
    pub desc: String,
    #[serde(rename = "iswatchonly")]
    pub is_watch_only: bool,
    #[serde(rename = "isscript")]
    pub is_script: bool,
    #[serde(rename = "iswitness")]
    pub is_witness: bool,
    pub witness_version: u64,
    pub witness_program: String,
    pub pubkey: String,
    #[serde(rename = "ischange")]
    pub is_change: bool,
    pub timestamp: u64,
    #[serde(rename = "hdkeypath")]
    pub hd_key_path: String,
    #[serde(rename = "hdseedid")]
    pub hd_seedid: String,
    #[serde(rename = "hdmasterfingerprint")]
    pub hd_master_fingerprint: String,
    pub labels: Vec<String>,
}

/// Response to the RPC command `gettransaction`.
///
/// It only defines one field, but can be expanded to include all the
/// fields returned by `bitcoind` (see:
/// https://bitcoincore.org/en/doc/0.19.0/rpc/wallet/gettransaction/)
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct WalletTransactionInfo {
    pub fee: f64,
}

#[cfg(all(test, feature = "test-docker"))]
mod test {
    use super::*;
    use crate::Bitcoind;
    use testcontainers::clients;

    #[tokio::test]
    async fn get_network_info() {
        let client = {
            let tc_client = clients::Cli::default();
            let blockchain = Bitcoind::new(&tc_client, "0.19.1").unwrap();

            Client::new(blockchain.node_url)
        };

        let network = client.network().await.unwrap();

        assert_eq!(network, Network::Regtest)
    }

    #[tokio::test]
    async fn get_median_time() {
        let client = {
            let tc_client = clients::Cli::default();
            let blockchain = Bitcoind::new(&tc_client, "0.19.1").unwrap();

            Client::new(blockchain.node_url)
        };

        let _mediant_time = client.median_time().await.unwrap();
    }

    #[test]
    fn decode_wallet_info() {
        let json = r#"{
        "walletname":"nectar_7426b018",
        "walletversion":169900,
        "balance":0.00000000,
        "unconfirmed_balance":0.00000000,
        "immature_balance":0.00000000,
        "txcount":0,
        "keypoololdest":1592792998,
        "keypoolsize":1000,
        "keypoolsize_hd_internal":1000,
        "paytxfee":0.00000000,
        "hdseedid":"4959e065fd8e278e4ffe62254897ddac18b02674",
        "private_keys_enabled":true,
        "avoid_reuse":false,
        "scanning":false
        }"#;

        let info: WalletInfoResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(
            info,
            WalletInfoResponse {
                wallet_name: "nectar_7426b018".into(),
                wallet_version: 169_900,
                tx_count: 0,
                keypool_oldest: 1_592_792_998,
                keypool_size_hd_internal: 1000,
                unlocked_until: None,
                pay_tx_fee: 0.0,
                hd_seed_id: Some("4959e065fd8e278e4ffe62254897ddac18b02674".into()),
                private_keys_enabled: true,
                avoid_reuse: false,
                scanning: ScanProgress::Bool(false)
            }
        )
    }
}
