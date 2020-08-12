use bitcoin::{Address, Amount, Transaction, Txid};
use bitcoincore_rpc::bitcoincore_rpc_json::{
    AddressType, GetAddressInfoResult, ListUnspentResultEntry,
};
use bitcoincore_rpc::RawTx;
use bitcoincore_rpc::RpcApi;
use std::result;

/// A wrapper to bitcoind wallet
#[derive(Debug)]
pub struct Wallet {
    pub bitcoind_client: bitcoincore_rpc::Client,
}

pub type Result<T> = result::Result<T, bitcoincore_rpc::Error>;

impl Wallet {
    pub fn new(bitcoind_client: bitcoincore_rpc::Client) -> Self {
        Self { bitcoind_client }
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
