use crate::bitcoind_rpc::{Client, Result, Unspent, WalletInfoResponse};
use bitcoin::{Address, Amount, Transaction, Txid};
use url::Url;

/// A wrapper to bitcoind wallet
#[derive(Debug)]
pub struct Wallet {
    name: String,
    bitcoind_client: Client,
}

impl Wallet {
    /// Create a wallet on the bitcoind instance or use the wallet with the same name
    /// if it exists.
    pub async fn new(name: &str, url: Url) -> Result<Self> {
        let bitcoind_client = Client::new(url);

        let wallet = Self {
            name: name.to_string(),
            bitcoind_client,
        };

        wallet.init().await?;

        Ok(wallet)
    }

    async fn init(&self) -> Result<()> {
        match self.info().await {
            Err(_) => self
                .bitcoind_client
                .create_wallet(&self.name, None, None, None, None)
                .await
                .map(|_| ()),
            Ok(_) => Ok(()),
        }
    }

    pub async fn info(&self) -> Result<WalletInfoResponse> {
        Ok(self.bitcoind_client.get_wallet_info(&self.name).await?)
    }

    pub async fn new_address(&self) -> Result<Address> {
        self.bitcoind_client
            .get_new_address(&self.name, None, Some("bech32".into()))
            .await
    }

    pub async fn balance(&self) -> Result<Amount> {
        self.bitcoind_client
            .get_balance(&self.name, None, None, None)
            .await
    }

    pub async fn send_to_address(&self, address: Address, amount: Amount) -> Result<Txid> {
        self.bitcoind_client
            .send_to_address(&self.name, address, amount)
            .await
    }

    pub async fn send_raw_transaction(&self, transaction: Transaction) -> Result<Txid> {
        self.bitcoind_client
            .send_raw_transaction(&self.name, transaction)
            .await
    }

    pub async fn get_raw_transaction(&self, txid: Txid) -> Result<Transaction> {
        self.bitcoind_client
            .get_raw_transaction(&self.name, txid)
            .await
    }

    pub async fn list_unspent(&self) -> Result<Vec<Unspent>> {
        self.bitcoind_client
            .list_unspent(&self.name, None, None, None, None)
            .await
    }
}
