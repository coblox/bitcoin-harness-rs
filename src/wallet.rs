use crate::bitcoind_rpc::{
    AddressInfo, Client, CreateFundedPsbtOptions, CreateFundedPsbtResult, Result, Unspent,
    WalletCreateFundedPsbtInput, WalletInfoResponse,
};
use bitcoin::{Address, Amount, Transaction, Txid};
use std::collections::HashMap;
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

    pub async fn address_info(&self, address: &Address) -> Result<AddressInfo> {
        self.bitcoind_client.address_info(&self.name, address).await
    }

    pub async fn list_unspent(&self) -> Result<Vec<Unspent>> {
        self.bitcoind_client
            .list_unspent(&self.name, None, None, None, None)
            .await
    }

    pub async fn wallet_create_funded_psbt(
        &self,
        tx_ins: &[WalletCreateFundedPsbtInput],
        outputs: &HashMap<String, Amount>,
        locktime: Option<i64>,
        options: Option<CreateFundedPsbtOptions>,
        bip32derivs: Option<bool>,
    ) -> Result<CreateFundedPsbtResult> {
        self.bitcoind_client
            .wallet_create_funded_psbt(&self.name, tx_ins, outputs, locktime, options, bip32derivs)
            .await
    }
}

#[cfg(all(test, feature = "test-docker"))]
mod test {
    use super::*;
    use crate::Bitcoind;
    use bitcoin::util::psbt::PartiallySignedTransaction;
    use testcontainers::clients;

    #[tokio::test]
    async fn partial_signed_transaction_test() {
        let wallet = {
            let tc_client = clients::Cli::default();
            let bitcoind = Bitcoind::new(&tc_client, "0.19.1").unwrap();
            bitcoind.init(5).await.unwrap();

            let wallet = Wallet::new("test_wallet", bitcoind.node_url.clone())
                .await
                .unwrap();

            let address = wallet.new_address().await.unwrap();
            let amount = bitcoin::Amount::from_btc(3.0).unwrap();
            bitcoind.mint(address, amount).await.unwrap();
            wallet
        };

        let address = wallet.new_address().await.unwrap();
        let mut output = HashMap::new();
        output.insert(address.to_string(), Amount::from_btc(1.1337).unwrap());
        let result = wallet
            .wallet_create_funded_psbt(&[], &output, None, None, None)
            .await
            .unwrap();
        let hex = base64::decode(result.psbt.clone()).unwrap();
        let psbt: PartiallySignedTransaction = bitcoin::consensus::deserialize(&hex).unwrap();

        assert_eq!(
            psbt.inputs.len(),
            1,
            "Should have 1 input as the wallet only has 1 tx"
        );
        assert_eq!(
            psbt.outputs.len(),
            2,
            "Should have 2 outputs, 1 for the target and one change"
        );
    }
}
