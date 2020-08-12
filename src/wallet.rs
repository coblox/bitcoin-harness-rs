use bitcoin::{Address, Amount, Transaction, Txid};
use bitcoincore_rpc::bitcoincore_rpc_json::{
    AddressType, GetAddressInfoResult, ListUnspentResultEntry, LoadWalletResult,
};
use bitcoincore_rpc::RawTx;
use bitcoincore_rpc::RpcApi;
use std::result;
use url::Url;

/// A wrapper to bitcoind wallet
#[derive(Debug)]
pub struct Wallet {
    name: String,
    pub bitcoind_client: bitcoincore_rpc::Client,
}

pub type Result<T> = result::Result<T, bitcoincore_rpc::Error>;

impl Wallet {
    /// Create a wallet on the bitcoind instance or use the wallet with the same name
    /// if it exists.
    pub fn new(name: &str, url: &Url, auth: &bitcoincore_rpc::Auth) -> Result<Self> {
        let bitcoind_client = bitcoincore_rpc::Client::new(url.to_string(), auth.clone()).unwrap();
        let mut url = url.clone();
        url.set_path(format!("wallet/{}", name).as_str());
        let wallet_client = bitcoincore_rpc::Client::new(url.to_string(), auth.clone()).unwrap();

        let wallet = Self {
            name: name.to_string(),
            bitcoind_client: wallet_client,
        };

        wallet.init(bitcoind_client)?;

        Ok(wallet)
    }

    fn init(&self, bitcoind_client: bitcoincore_rpc::Client) -> Result<LoadWalletResult> {
        bitcoind_client.create_wallet(&self.name, None)
    }

    pub fn new_address(&self) -> Result<Address> {
        self.bitcoind_client
            .get_new_address(None, Some(AddressType::Bech32))
    }

    pub fn balance(&self) -> Result<Amount> {
        self.bitcoind_client.get_balance(None, None)
    }

    pub fn send_to_address(&self, address: &Address, amount: Amount) -> Result<Txid> {
        self.bitcoind_client
            .send_to_address(address, amount, None, None, None, None, None, None)
    }

    pub fn send_raw_transaction(&self, transaction: Transaction) -> Result<Txid> {
        self.bitcoind_client
            .send_raw_transaction(transaction.raw_hex())
    }

    pub fn get_raw_transaction(&self, txid: &Txid) -> Result<Transaction> {
        self.bitcoind_client.get_raw_transaction(txid, None)
    }

    pub fn address_info(&self, address: &Address) -> Result<GetAddressInfoResult> {
        self.bitcoind_client.get_address_info(address)
    }

    pub fn list_unspent(&self) -> Result<Vec<ListUnspentResultEntry>> {
        self.bitcoind_client
            .list_unspent(None, None, None, None, None)
    }
}
