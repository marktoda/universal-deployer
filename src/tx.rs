use crate::opt::TransactionConfig;
use anyhow::Result;
use ethers::types::{
    transaction::{eip2718::TypedTransaction, request::TransactionRequest},
    Address, Signature, U256,
};

// creates a typedtransaction for a general contract deployment with the given bytecode
pub fn create_transaction(config: &TransactionConfig) -> Result<TypedTransaction> {
    let request = TransactionRequest {
        from: None,
        to: None,
        gas: Some(U256::from_dec_str(&config.gas_limit)?),
        gas_price: Some(U256::from_dec_str(&config.gas_price)?),
        value: None,
        data: Some(config.bytecode.clone()),
        nonce: Some(U256::from_dec_str("0")?),
        chain_id: None,
    };

    Ok(TypedTransaction::Legacy(request))
}

// easy helper to recover the signer for a tx
pub trait Recoverable {
    fn recover(&self, sig: Signature) -> Result<Address>;
}

impl Recoverable for TypedTransaction {
    fn recover(&self, sig: Signature) -> Result<Address> {
        Ok(sig.recover(self.sighash())?)
    }
}
