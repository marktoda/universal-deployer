use crossbeam_channel::{Receiver, bounded};
use ethers::types::Bytes;
use anyhow::Result;
use std::fmt::Display;
mod opt;
use opt::Opts;
mod tx;
use tx::create_transaction;
mod sig;
use sig::{find_signature, SignatureResult};
mod util;

fn main() {
    match find() {
        Ok(result) => {
            println!("{}", result);
        }
        Err(err) => {
            println!("ERROR! {:?}", err);
        }
    }
}

#[derive(Debug)]
pub struct FindResult {
    info: SignatureResult,
    tx: Bytes,
}

impl Display for FindResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Info: {}\nRaw Tx: {}", self.info, self.tx)
    }
}

fn find() -> Result<FindResult> {
    let opts = Opts::parse()?;

    let tx = create_transaction(&opts.tx_config)?;

    let receiver = signal_channel()?;
    let info = find_signature(&tx, &opts.gen_config, receiver)?;
    Ok(FindResult {
        info: info.clone(),
        tx: tx.rlp_signed(&info.sig),
    })
}

// channel for sigint to finish work before exiting
fn signal_channel() -> Result<Receiver<()>> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}
