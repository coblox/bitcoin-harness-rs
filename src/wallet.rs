use bitcoin::{Address, Amount, Transaction, Txid};
use bitcoincore_rpc::bitcoincore_rpc_json::{
    AddressType, GetAddressInfoResult, ListUnspentResultEntry,
};
use bitcoincore_rpc::RawTx;
use bitcoincore_rpc::RpcApi;
use std::result;

pub type Result<T> = result::Result<T, bitcoincore_rpc::Error>;

/// A wrapper to bitcoincore_rpc client
#[derive(Debug)]
pub struct Wallet(bitcoincore_rpc::Client);

impl Wallet {
    pub fn new_address(&self) -> Result<Address> {
        self.0.get_new_address(None, Some(AddressType::Bech32))
    }

    pub fn balance(&self) -> Result<Amount> {
        self.0.get_balance(None, None)
    }

    pub fn send_to_address(&self, address: &Address, amount: Amount) -> Result<Txid> {
        self.0
            .send_to_address(address, amount, None, None, None, None, None, None)
    }

    pub fn send_raw_transaction(&self, transaction: Transaction) -> Result<Txid> {
        self.0.send_raw_transaction(transaction.raw_hex())
    }

    pub fn get_raw_transaction(&self, txid: &Txid) -> Result<Transaction> {
        self.0.get_raw_transaction(txid, None)
    }

    pub fn address_info(&self, address: &Address) -> Result<GetAddressInfoResult> {
        self.0.get_address_info(address)
    }

    pub fn list_unspent(&self) -> Result<Vec<ListUnspentResultEntry>> {
        self.0.list_unspent(None, None, None, None, None)
    }
}

impl From<bitcoincore_rpc::Client> for Wallet {
    fn from(client: bitcoincore_rpc::Client) -> Self {
        Self(client)
    }
}

impl From<Wallet> for bitcoincore_rpc::Client {
    fn from(wallet: Wallet) -> Self {
        wallet.0
    }
}
