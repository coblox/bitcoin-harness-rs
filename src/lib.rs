#![warn(
    unused_extern_crates,
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::fallible_impl_from,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::dbg_macro
)]
#![forbid(unsafe_code)]

//! # bitcoin-harness
//! A simple lib to start a bitcoind container, generate blocks and funds addresses.
//! Note: It uses tokio.
//!
//! # Examples
//!
//! ## Just connect to bitcoind and get the network
//!
//! ```rust
//! use bitcoin_harness::{Bitcoind};
//! use bitcoincore_rpc::RpcApi;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let tc_client = testcontainers::clients::Cli::default();
//! let bitcoind = Bitcoind::new(&tc_client, "0.20.0").unwrap();
//!
//! let network = bitcoind.bitcoind_client.get_blockchain_info().unwrap().chain;
//!
//! assert_eq!(network, bitcoin::Network::Regtest.to_string())
//! # }
//! ```
//!
//! ## Create a wallet, fund it and get a UTXO
//!
//! ```rust
//! use bitcoin_harness::{Bitcoind, Wallet};
//! use bitcoincore_rpc::RpcApi;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let tc_client = testcontainers::clients::Cli::default();
//! let bitcoind = Bitcoind::new(&tc_client, "0.19.1").unwrap();
//!
//! bitcoind.init(5).await.unwrap();
//!
//! let wallet = Wallet::new("my_wallet", &bitcoind.node_url, &bitcoind.auth).unwrap();
//! let address = wallet.new_address().unwrap();
//! let amount = bitcoin::Amount::from_btc(3.0).unwrap();
//!
//! bitcoind.mint(&address, amount).await.unwrap();
//!
//! let balance = wallet.balance().unwrap();
//!
//! assert_eq!(balance, amount);
//!
//! let utxos = wallet.list_unspent().unwrap();
//!
//! assert_eq!(utxos.get(0).unwrap().amount, amount);
//! # }
//! ```

pub mod wallet;

use std::time::Duration;
use testcontainers::{clients, images::coblox_bitcoincore::BitcoinCore, Container, Docker};

pub use crate::wallet::Wallet;
use bitcoincore_rpc::RpcApi;
use url::Url;

pub type Result<T> = std::result::Result<T, Error>;

const BITCOIND_RPC_PORT: u16 = 18443;

#[derive(Debug)]
pub struct Bitcoind<'c> {
    pub container: Container<'c, clients::Cli, BitcoinCore>,
    pub node_url: Url,
    pub auth: bitcoincore_rpc::Auth,
    pub bitcoind_client: bitcoincore_rpc::Client,
    pub test_wallet: Wallet,
}

impl<'c> Bitcoind<'c> {
    /// Starts a new regtest bitcoind container
    pub fn new(client: &'c clients::Cli, tag: &str) -> Result<Self> {
        let container = client.run(BitcoinCore::default().with_tag(tag));
        let port = container
            .get_host_port(BITCOIND_RPC_PORT)
            .ok_or(Error::PortNotExposed(BITCOIND_RPC_PORT))?;

        let auth = container.image().auth();
        let url = format!("http://localhost:{}", port);
        let url = Url::parse(&url)?;
        let auth =
            bitcoincore_rpc::Auth::UserPass(auth.username.to_string(), auth.password.to_string());

        let bitcoind_client = bitcoincore_rpc::Client::new(url.to_string(), auth.clone()).unwrap();
        let test_wallet = Wallet::new("testwallet", &url, &auth).unwrap();
        Ok(Self {
            container,
            node_url: url,
            auth,
            bitcoind_client,
            test_wallet,
        })
    }

    /// Create a test wallet, generate enough block to fund it and activate segwit.
    /// Generate enough blocks to make the passed `spendable_quantity` spendable.
    /// Spawn a tokio thread to mine a new block every second.
    pub async fn init(&self, spendable_quantity: u64) -> Result<()> {
        let reward_address = self.test_wallet.new_address().unwrap();

        self.bitcoind_client
            .generate_to_address(101 + spendable_quantity, &reward_address)
            .unwrap();
        let miner =
            bitcoincore_rpc::Client::new(self.node_url.to_string(), self.auth.clone()).unwrap();
        let _ = tokio::spawn(mine(miner, reward_address));

        Ok(())
    }

    /// Send Bitcoin to the specified address, limited to the spendable bitcoin quantity.
    pub async fn mint(&self, address: &bitcoin::Address, amount: bitcoin::Amount) -> Result<()> {
        self.test_wallet.send_to_address(address, amount).unwrap();

        // Confirm the transaction
        let reward_address = self.test_wallet.new_address().unwrap();
        self.bitcoind_client
            .generate_to_address(1, &reward_address)
            .unwrap();

        Ok(())
    }

    pub fn container_id(&self) -> &str {
        self.container.id()
    }
}

async fn mine(miner: bitcoincore_rpc::Client, reward_address: bitcoin::Address) -> Result<()> {
    loop {
        tokio::time::delay_for(Duration::from_secs(1)).await;
        miner.generate_to_address(1, &reward_address).unwrap();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Bitcoin Rpc: ")]
    BitcoindRpc(#[from] bitcoincore_rpc::Error),
    #[error("Url Parsing: ")]
    UrlParseError(#[from] url::ParseError),
    #[error("Docker port not exposed: ")]
    PortNotExposed(u16),
}
