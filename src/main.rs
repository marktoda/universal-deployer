use anyhow::Result;
use crossbeam_channel::{bounded, Receiver};
use ethers::types::Bytes;
use std::fmt::Display;
use serde::Serialize;
mod opt;
use opt::{Config, Opts};
mod tx;
use tx::create_transaction;
mod sig;
use sig::{find_signature, SignatureResult};
mod util;
mod address;

fn main() {
    let config = Opts::parse().unwrap();
    let result = find(&config);
    output(&config, result).unwrap();
}

#[derive(Debug, Serialize)]
pub struct FindResult {
    info: SignatureResult,
    tx: Bytes,
}

impl Display for FindResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Info: {}\nRaw Tx: {}", self.info, self.tx)
    }
}

// create a transaction to deploy the given contract
fn find(opts: &Config) -> Result<FindResult> {
    let tx = create_transaction(&opts.tx_config)?;

    let receiver = signal_channel()?;
    let info = find_signature(&tx, &opts.gen_config, receiver)?;
    Ok(FindResult {
        info: info.clone(),
        tx: tx.rlp_signed(&info.sig),
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
