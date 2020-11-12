use bitcoin::{Address, Network, Script, Transaction, Txid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[jsonrpc_client::api]
pub trait BitcoindRpcApi {
    async fn createwallet(
        &self,
        wallet_name: &str,
        disable_private_keys: Option<bool>,
        blank: Option<bool>,
        passphrase: Option<String>,
        avoid_reuse: Option<bool>,
    ) -> CreateWalletResponse;

    async fn deriveaddresses(&self, descriptor: &str, range: Option<[u64; 2]>) -> Vec<Address>;

    async fn dumpwallet(&self, filename: &std::path::Path) -> DumpWalletResponse;

    async fn finalizepsbt(&self, psbt: PsbtBase64) -> FinalizePsbtResponse;

    async fn generatetoaddress(
        &self,
        nblocks: u32,
        address: Address,
        max_tries: Option<u32>,
    ) -> Vec<BlockHash>;

    async fn getaddressinfo(&self, address: &Address) -> AddressInfoResponse;

    async fn getbalance(
        &self,
        account: Account,
        minimum_confirmation: Option<u32>,
        include_watch_only: Option<bool>,
        avoid_reuse: Option<bool>,
    ) -> f64;

    async fn getblock(&self, block_hash: &bitcoin::BlockHash) -> GetBlockResponse;

    async fn getblockchaininfo(&self) -> GetBlockchainInfoResponse;

    async fn getblockcount(&self) -> u32;

    async fn getdescriptorinfo(&self, descriptor: &str) -> GetDescriptorInfoResponse;

    async fn getnewaddress(&self, label: Option<String>, address_type: Option<String>) -> Address;

    async fn gettransaction(&self, txid: Txid) -> GetTransactionResponse;

    async fn getwalletinfo(&self) -> GetWalletInfoResponse;

    async fn joinpsbts(&self, psbts: &[String]) -> PsbtBase64;

    async fn listunspent(
        &self,
        min_conf: Option<u32>,
        max_conf: Option<u32>,
        addresses: Option<Vec<Address>>,
        include_unsafe: Option<bool>,
    ) -> Vec<Unspent>;

    async fn listwallets(&self) -> Vec<String>;

    async fn sendrawtransaction(&self, transaction: Transaction) -> String;

    /// amount is btc
    async fn sendtoaddress(&self, address: Address, amount: f64) -> String;

    async fn sethdseed(&self, new_key_pool: Option<bool>, wif_private_key: Option<String>) -> ();

    /// Outputs are {address, btc amount}
    async fn walletcreatefundedpsbt(
        &self,
        inputs: &[bitcoincore_rpc_json::CreateRawTransactionInput],
        outputs: HashMap<String, f64>,
    ) -> WalletCreateFundedPsbtResponse;

    async fn walletprocesspsbt(&self, psbt: PsbtBase64) -> WalletProcessPsbtResponse;
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AddressInfoResponse {
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

#[derive(Debug, Deserialize)]
pub struct CreateWalletResponse {
    pub name: String,
    pub warning: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DumpWalletResponse {
    pub filename: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FinalizePsbtResponse {
    pub hex: String,
    pub complete: bool,
}

#[derive(Debug, Deserialize)]
pub struct GetBlockchainInfoResponse {
    pub chain: Network,
    #[serde(rename = "mediantime")]
    pub median_time: u32,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct GetBlockResponse {
    pub height: u32,
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

#[derive(Clone, Debug, Deserialize)]
pub enum GetRawTransactionResponse {
    Normal(String),
    Verbose {
        #[serde(rename = "blockhash")]
        block_hash: Option<bitcoin::BlockHash>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct GetTransactionResponse {
    pub fee: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GetWalletInfoResponse {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct PsbtBase64(pub String);

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

#[derive(Debug, Deserialize, Serialize)]
pub struct WalletCreateFundedPsbtResponse {
    pub psbt: String,
    pub fee: f64,
    #[serde(rename = "changepos")]
    pub change_position: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WalletProcessPsbtResponse {
    psbt: String,
    complete: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ScanProgress {
    Bool(bool),
    Progress { duration: u32, progress: f64 },
}

#[derive(Debug, Serialize)]
#[serde(rename = "*")]
pub struct Account;

#[derive(Debug, Deserialize)]
pub struct BlockHash(String);

impl From<WalletProcessPsbtResponse> for PsbtBase64 {
    fn from(processed_psbt: WalletProcessPsbtResponse) -> Self {
        Self(processed_psbt.psbt)
    }
}

impl From<String> for PsbtBase64 {
    fn from(base64_string: String) -> Self {
        Self(base64_string)
    }
}
