use bitcoin::{Address, Amount, Transaction, Txid};
use bitcoincore_rpc::bitcoincore_rpc_json::{
    AddressType, GetAddressInfoResult, ListUnspentResultEntry, WalletCreateFundedPsbtResult,
};
use bitcoincore_rpc::RpcApi;
use bitcoincore_rpc::{json, RawTx};
use serde_json;
use std::collections::HashMap;
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

    pub fn wallet_fund_psbt(
        &self,
        inputs: &[json::CreateRawTransactionInput],
        outputs: &HashMap<String, Amount>,
        locktime: Option<i64>,
    ) -> Result<WalletCreateFundedPsbtResult> {
        self.0
            .wallet_create_funded_psbt(inputs, outputs, locktime, None, None)
    }

    pub fn join_psbt(&self, psbts: &[String]) -> Result<String> {
        let value = serde_json::to_value(psbts)?;
        self.0.call("joinpsbts", &[value])
    }

    pub fn wallet_process_psbt(&self, psbt: &str) -> Result<String> {
        let args = [
            serde_json::to_value(psbt)?,
            serde_json::to_value(true)?,
            serde_json::to_value("ALL")?,
            serde_json::to_value(false)?,
        ];
        self.0.call("walletprocesspsbt", &args)
    }

    pub fn bitcoin_rpc_client(&self) -> &bitcoincore_rpc::Client {
        &self.0
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

#[cfg(all(test))]
mod test {
    use super::*;
    use crate::Bitcoind;
    use bitcoin::util::psbt::PartiallySignedTransaction;
    use bitcoin::Amount;
    use testcontainers::clients;

    #[tokio::test]
    async fn create_and_fund_wallet() {
        let tc_client = clients::Cli::default();
        let bitcoind = Bitcoind::new(&tc_client, "0.19.1").unwrap();
        bitcoind.init(5).await.unwrap();
        let alice = bitcoind.new_wallet("alice").unwrap();
        let bob = bitcoind.new_wallet("bob").unwrap();
        let address = alice.new_address().unwrap();
        let amount = Amount::from_btc(3.0).unwrap();
        bitcoind.mint(&address, amount).await.unwrap();

        let address = bob.new_address().unwrap();
        let amount = Amount::from_btc(3.0).unwrap();
        bitcoind.mint(&address, amount).await.unwrap();

        let address = alice.new_address().unwrap();
        let mut outputs = HashMap::new();
        outputs.insert(address.clone().to_string(), Amount::from_btc(1.0).unwrap());
        let alice_psbt = alice.wallet_fund_psbt(&[], &outputs, None).unwrap();

        let mut outputs = HashMap::new();
        outputs.insert(address.clone().to_string(), Amount::from_btc(1.0).unwrap());
        let bob_psbt = bob.wallet_fund_psbt(&[], &outputs, None).unwrap();

        let alice_combined_psbt = alice
            .join_psbt(&[alice_psbt.clone().psbt, bob_psbt.clone().psbt])
            .unwrap();

        let partial_signed_bitcoin_transaction: PartiallySignedTransaction = {
            let as_hex = base64::decode(alice_combined_psbt).unwrap();
            bitcoin::consensus::deserialize(&as_hex).unwrap()
        };

        // TODO: remove duplicates
        // let transaction = partial_signed_bitcoin_transaction.extract_tx();
        // let inputs = transaction.input.clone();
        // let mut outputs = vec![];
        // transaction.output.iter().for_each(|output| {
        //     if !outputs.contains(output) {
        //         outputs.push(output.clone());
        //     }
        // });
        //
        // let transaction = Transaction {
        //     version: 2,
        //     lock_time: 0,
        //     input: inputs,
        //     output: outputs,
        // };
        // let partial_signed_bitcoin_transaction =
        //     PartiallySignedTransaction::from_unsigned_tx(transaction).unwrap();

        let psbt = {
            let hex_vec = bitcoin::consensus::serialize(&partial_signed_bitcoin_transaction);
            base64::encode(hex_vec)
        };
        println!("{:?}", psbt);
        let result = alice.wallet_process_psbt(&psbt).unwrap();
        println!("{:?}", result);
    }
}
