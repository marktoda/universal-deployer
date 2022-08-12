#![feature(test)]
extern crate test;

use anyhow::Result;
use crossbeam_channel::{bounded, Receiver};
use ethers::types::{transaction::request::TransactionRequest, Bytes, U256};
use serde::Serialize;
use std::fmt::Display;
mod opt;
use opt::{Config, Opts};
mod tx;
use tx::{create_transaction, Recoverable};
mod sig;
use sig::{find_signature, SignatureResult};
mod address;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Opts::parse()?;
    let result = find(&config).await;
    output(&config, result)?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct FindResult {
    info: SignatureResult,
    tx: TransactionRequest,
    tx_raw: Bytes,
    tx_cost: U256,
}

impl Display for FindResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n==== Info ====\n{}\n\n==== Raw Tx ====\n{}\n\nSend exactly {} native tokens to the deployer address",
            self.info,
            self.tx_raw,
            self.tx_cost
        )
    }
}

// create a transaction to deploy the given contract
async fn find(opts: &Config) -> Result<FindResult> {
    let tx = create_transaction(&opts.tx_config).await?;

    let receiver = signal_channel()?;
    let info = find_signature(&tx, &opts.address_config, receiver)?;
    Ok(FindResult {
        info: info.clone(),
        tx: tx.clone(),
        tx_raw: tx.rlp_signed(&info.sig),
        tx_cost: tx.get_cost(),
    })
}

// output the find result
fn output(config: &Config, result: Result<FindResult>) -> Result<()> {
    match result {
        Ok(result) => {
            if config.json {
                println!("{}", serde_json::to_string(&result)?);
            } else {
                println!("{}", result);
            }
        }
        Err(err) => {
            eprintln!("{:?}", err);
        }
    }
    Ok(())
}

// channel for sigint to finish work before exiting
fn signal_channel() -> Result<Receiver<()>> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}
