use crate::opt::TransactionConfig;
use anyhow::Result;
use ethers::providers::{Http, Middleware, Provider};
use ethers::types::{
    transaction::{eip2718::TypedTransaction, request::TransactionRequest},
    Address, Signature, U256,
};

// creates a typedtransaction for a general contract deployment with the given bytecode
pub async fn create_transaction(config: &TransactionConfig) -> Result<TransactionRequest> {
    build_transaction_request(config, Some(get_gas(config).await?))
}

async fn get_gas(config: &TransactionConfig) -> Result<U256> {
    if config.gas_limit.is_some() {
        return Ok(U256::from_dec_str(
            &config
                .gas_limit
                .as_ref()
                .ok_or(anyhow::anyhow!("Invalid gas limit"))?,
        )?);
    }

    let gas = if config.rpc_url.is_some() {
        let provider = Provider::<Http>::try_from(
            config
                .rpc_url
                .as_ref()
                .ok_or(anyhow::anyhow!("Invalid rpc url"))?,
        )?;
        let tx = build_transaction_request(config, None)?;
        let estimate = provider.estimate_gas(&TypedTransaction::Legacy(tx)).await?;
        // 20% buffer for chain-specific gas costs or other weirdness
        estimate
            .saturating_mul(U256::from(6))
            .checked_div(U256::from(5))
            .ok_or(anyhow::anyhow!("Gas estimate too high"))?
    } else {
        // default to 1 million :shrug:
        U256::from_dec_str("1000000")?
    };

    Ok(gas)
}

fn build_transaction_request(
    config: &TransactionConfig,
    gas: Option<U256>,
) -> Result<TransactionRequest> {
    Ok(TransactionRequest {
        from: None,
        to: None,
        gas,
        gas_price: Some(U256::from_dec_str(&config.gas_price)?),
        value: None,
        data: Some(config.bytecode.clone()),
        nonce: Some(U256::from_dec_str("0")?),
        chain_id: None,
    })
}

// easy helper to recover the signer for a tx
pub trait Recoverable {
    fn recover(&self, sig: Signature) -> Result<Address>;
    fn get_cost(&self) -> U256;
}

impl Recoverable for TransactionRequest {
    fn recover(&self, sig: Signature) -> Result<Address> {
        Ok(sig.recover(self.sighash())?)
    }

    fn get_cost(&self) -> U256 {
        self.gas_price
            .unwrap_or_default()
            .saturating_mul(self.gas.unwrap_or_default())
    }
}
