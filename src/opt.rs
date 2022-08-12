use crate::util::strip_hex_prefix;
use anyhow::Result;
use ethers::types::{Bytes, U256};
use serde::Deserialize;
use std::str::FromStr;
use std::{fmt::Display, fs};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "universal-deployer",
    about = "Tool to generate single-use keyless contract deployment transactions"
)]
pub struct Opts {
    #[structopt(help = "Path to a contract artifact file or hex contract bytecode")]
    bytecode_or_artifact: String,
    #[structopt(
        short = "c",
        long = "constructor-args",
        help = "ABI encoded constructor args to pass to the deployment"
    )]
    constructor_args: Option<String>,
    #[structopt(
        short = "p",
        long = "prefix",
        help = "A prefix for the deployed contract address"
    )]
    prefix: Option<String>,
    #[structopt(
        short = "n",
        long = "num-zero-bytes",
        help = "The number of zero bytes to exist in the deployed contract address"
    )]
    num_zero_bytes: Option<usize>,
    #[structopt(
        short = "s",
        long = "s-start",
        help = "The S value to start with, useful when running multiple instances to grind"
    )]
    s_start: Option<String>,
    #[structopt(
        long = "gas-price",
        help = "The gas price to use for the transaction. Recommended to use a generally high price to allow confirmation on many chains",
        default_value = "100000000000"
    )]
    gas_price: String,
    #[structopt(
        long = "gas-limit",
        help = "The gas limit to use for the transaction. Recommended to use a generally overestimated limit to allow confirmation on many chains"
    )]
    gas_limit: Option<String>,
    #[structopt(
        short = "r",
        long = "rpc-url",
        help = "Optional RPC url to estimate deployment gas limit"
    )]
    rpc_url: Option<String>,
    #[structopt(short = "j", long = "json", help = "Print output in json format")]
    json: bool,
}

#[derive(Debug)]
pub struct AddressGenerationConfig {
    pub prefix: Option<String>,
    pub num_zero_bytes: usize,
    pub s_start: U256,
}

impl Display for AddressGenerationConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = self
            .prefix
            .as_ref()
            .map(|p| format!("\n\t- Prefix: {}", p))
            .unwrap_or_default();
        let num_zero_bytes = if self.num_zero_bytes == 0 {
            "".to_string()
        } else {
            format!("\n\t- Min zero bytes: {}", self.num_zero_bytes)
        };
        let s_start = format!("\n\t- S start: {}", self.s_start);
        write!(
            f,
            "\nAddress Generation Config{}{}{}",
            prefix, num_zero_bytes, s_start
        )
    }
}

#[derive(Debug)]
pub struct TransactionConfig {
    pub bytecode: Bytes,
    pub gas_price: String,
    pub gas_limit: Option<String>,
    pub rpc_url: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    pub tx_config: TransactionConfig,
    pub address_config: AddressGenerationConfig,
    pub json: bool,
}

#[derive(Deserialize)]
struct Artifact {
    bytecode: BytecodeEnum,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BytecodeEnum {
    Hardhat(String),
    Forge(ForgeBytecode),
}

#[derive(Deserialize)]
struct ForgeBytecode {
    object: String,
}

impl Opts {
    pub fn parse() -> Result<Config> {
        let this = Opts::from_args();

        Ok(Config {
            json: this.json,
            tx_config: TransactionConfig {
                bytecode: parse_bytecode(&this)?,
                gas_price: this.gas_price,
                gas_limit: this.gas_limit,
                rpc_url: this.rpc_url,
            },
            address_config: AddressGenerationConfig {
                prefix: this.prefix.map(|s| strip_hex_prefix(&s)),
                num_zero_bytes: this.num_zero_bytes.unwrap_or_default(),
                s_start: U256::from_str_radix(
                    &this.s_start.unwrap_or_else(|| "1".to_string()),
                    16,
                )?,
            },
        })
    }
}

fn parse_bytecode(opts: &Opts) -> Result<Bytes> {
    let tail: String = opts
        .constructor_args
        .as_ref()
        .map_or("".to_string(), |a| strip_hex_prefix(a));

    let valid_hex = hex::decode(strip_hex_prefix(&opts.bytecode_or_artifact));
    let mut bytecode: String = match valid_hex {
        Ok(_) => opts.bytecode_or_artifact.to_string(),
        Err(_) => {
            let artifact: Artifact =
                serde_json::from_str(&fs::read_to_string(&opts.bytecode_or_artifact)?)?;
            match artifact.bytecode {
                BytecodeEnum::Hardhat(bytecode) => bytecode,
                BytecodeEnum::Forge(ForgeBytecode { object }) => object,
            }
        }
    };
    bytecode.push_str(&tail);
    Ok(Bytes::from_str(&bytecode)?)
}
