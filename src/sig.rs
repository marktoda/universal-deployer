use crate::address::{check_address, AddressMatch};
use crate::opt::AddressGenerationConfig;
use crate::tx::Recoverable;
use anyhow::Result;
use crossbeam_channel::Receiver;
use ethers::{
    types::{transaction::request::TransactionRequest, Address, Signature, U256},
    utils::get_contract_address,
};
use serde::Serialize;
use std::fmt::Display;

const SIG_V: u64 = 27;
// 0x1fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
// note decoding from string must be done runtime so slows down generation loop
const SIG_R: U256 = U256([u64::MAX, u64::MAX, u64::MAX, u64::MAX / 8]);

#[derive(Clone, Debug, Serialize)]
pub struct SignatureResult {
    pub sig: Signature,
    pub contract: Address,
    pub deployer: Address,
}

impl Display for SignatureResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Sig: {}\nContract: 0x{:x}\nDeployer: 0x{:x}",
            self.sig, self.contract, self.deployer
        )
    }
}

// attempt to find a signature that ecrecovers to an address that deploys a contract matching the
// given config.
// on channel signal, returns the best signature found so far
pub fn find_signature(
    tx: &TransactionRequest,
    config: &AddressGenerationConfig,
    signal: Receiver<()>,
) -> Result<SignatureResult> {
    let mut s: U256 = config.s_start;

    let mut best_s: U256 = s;
    let mut best_match_count: usize = 0;

    println!(
        "Starting search for deployment signature with config: {}",
        config
    );
    loop {
        let result = generate_signature(tx, s)?;

        match check_address(result.contract, config) {
            AddressMatch::Match => {
                return Ok(result);
            }
            AddressMatch::NoMatch(count) => {
                if count > best_match_count {
                    println!(
                        "Found new best signature with contract: {}, match_count: {}",
                        result.contract, count
                    );
                    best_s = s;
                    best_match_count = count;
                }
            }
        }

        if signal.try_recv().is_ok() {
            println!(
                "Received sigint - current s: {} - shutting down cleanly...",
                s
            );
            return generate_signature(tx, best_s);
        }

        s = s.overflowing_add(U256::from(1)).0;
    }
}

// generate a valid signature for the given tx using the given s value
fn generate_signature(tx: &TransactionRequest, s: U256) -> Result<SignatureResult> {
    let sig = Signature {
        v: SIG_V,
        r: SIG_R,
        s,
    };
    let deployer = tx.recover(sig)?;

    Ok(SignatureResult {
        sig,
        deployer,
        contract: get_contract_address(deployer, 0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::types::{Bytes, transaction::request::TransactionRequest};
    use std::str::FromStr;
    use test::Bencher;

    fn create_tx(bytecode: Bytes) -> TransactionRequest {
        TransactionRequest {
            from: None,
            to: None,
            gas: Some(U256::from_dec_str("1000000").unwrap()),
            gas_price: Some(U256::from("10000")),
            nonce: Some(U256::from(0)),
            value: Some(U256::from(0)),
            data: Some(bytecode),
            chain_id: None,
        }
    }

    #[bench]
    fn bench_generate_signature(b: &mut Bencher) {
        let tx = create_tx(Bytes::from_str("1234567890").unwrap());
        b.iter(|| generate_signature(&tx, U256::from(1)));
    }
}
