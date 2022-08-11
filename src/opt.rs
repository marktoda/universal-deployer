use crate::util::{add_hex_prefix, strip_hex_prefix};
use anyhow::Result;
use ethers::types::{Bytes, U256};
use serde::Deserialize;
use std::{fs, fmt::Display};
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "universal-deployer",
    about = "Tool to generate single-use keyless contract deployment transactions"
)]
pub struct Opts {
    #[structopt(
        short = "a",
        long = "artifact",
        help = "Path to a compiled contract artifact"
    )]
    artifact: Option<String>,
    #[structopt(
        short = "b",
        long = "bytecode",
        help = "The bytecode of the contract to deploy"
    )]
    bytecode: Option<String>,
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
}

#[derive(Debug)]
pub struct AddressGenerationConfig {
    pub prefix: Option<String>,
    pub num_zero_bytes: usize,
    pub s_start: U256,
}

impl Display for AddressGenerationConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = self.prefix.as_ref().map(|p| format!("\n\t- Prefix: {}", p)).unwrap_or_default();
        let num_zero_bytes = if self.num_zero_bytes == 0 {
            "".to_string()
        } else {
            format!("\n\t- Min zero bytes: {}", self.num_zero_bytes)
        };
        let s_start = format!("\n\t- S start: {}", self.s_start);
        write!(
            f,
            "\nAddress Generation Config{}{}{}",
            prefix,
            num_zero_bytes,
            s_start
        )
    }
}

#[derive(Debug)]
pub struct TransactionConfig {
    pub bytecode: Bytes,
}

#[derive(Debug)]
pub struct Config {
    pub tx_config: TransactionConfig,
    pub gen_config: AddressGenerationConfig,
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
            tx_config: TransactionConfig { bytecode: parse_bytecode(&this)? },
            gen_config: AddressGenerationConfig {
                prefix: this.prefix.map(|s| add_hex_prefix(&s)),
                num_zero_bytes: this.num_zero_bytes.unwrap_or_default(),
                s_start: U256::from_str_radix(
                    &this.s_start.unwrap_or_else(|| "3".to_string()),
                    16,
                )?,
            },
        })
    }
}

fn parse_bytecode(opts: &Opts) -> Result<Bytes> {
    let tail: String = opts.constructor_args.as_ref().map_or("".to_string(), |a| strip_hex_prefix(a));
    let mut bytecode: String = match (opts.artifact.clone(), opts.bytecode.clone()) {
        (Some(artifact), None) => {
            let artifact: Artifact =
                serde_json::from_str(&fs::read_to_string(artifact).unwrap()).unwrap();
            match artifact.bytecode {
                BytecodeEnum::Hardhat(bytecode) => bytecode,
                BytecodeEnum::Forge(ForgeBytecode { object }) => object,
            }
        }
        (None, Some(bytecode)) => bytecode,
        _ => panic!("Must provide either an artifact or bytecode"),
    };
    bytecode.push_str(&tail);
    Ok(Bytes::from_str(&bytecode)?)
}
