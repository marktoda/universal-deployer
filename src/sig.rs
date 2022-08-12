use anyhow::Result;
use crossbeam_channel::Receiver;
use ethers::{
    types::{transaction::eip2718::TypedTransaction, Address, Signature, U256},
    utils::get_contract_address,
};
use std::fmt::Display;
use crate::opt::AddressGenerationConfig;
use crate::tx::Recoverable;
use crate::address::{AddressMatch, check_address};

const SIG_V: u64 = 27;
const SIG_R: &str = "0x79ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

#[derive(Clone, Debug)]
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
    tx: &TypedTransaction,
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
        let address_match = check_address(result.contract, config);

        match address_match {
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
            println!("Received sigint, shutting down cleanly...");
            return generate_signature(tx, best_s);
        }

        if s.saturating_sub(config.s_start).as_u64() % 100000 == 0 && s != config.s_start {
            println!("Still running! Current s: {}", s);
        }

        s = s.overflowing_add(U256::from(1)).0;
    }
}

// generate a valid signature for the given tx using the given s value
fn generate_signature(tx: &TypedTransaction, s: U256) -> Result<SignatureResult> {
    let sig = Signature {
        v: SIG_V,
        r: U256::from_str_radix(SIG_R, 16)?,
        s,
    };
    let deployer = tx.recover(sig)?;

    Ok(SignatureResult {
        sig,
        deployer,
        contract: get_contract_address(deployer, 0),
    })
}
