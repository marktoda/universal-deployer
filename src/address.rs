use ethers::types::Address;
use crate::opt::AddressGenerationConfig;

pub enum AddressMatch {
    Match,
    NoMatch(usize),
}

pub fn check_address(address: Address, config: &AddressGenerationConfig) -> AddressMatch {
    match (&config.prefix, config.num_zero_bytes) {
        (None, 0) => AddressMatch::Match,
        (Some(prefix), 0) => {
            let count = count_prefix_match(address, &prefix);
            if count == prefix.len() {
                AddressMatch::Match
            } else {
                AddressMatch::NoMatch(count)
            }
        }
        (None, num_zero_bytes) => {
            let count = count_zero_bytes(address);
            if count == num_zero_bytes {
                AddressMatch::Match
            } else {
                AddressMatch::NoMatch(count)
            }
        }
        (Some(prefix), num_zero_bytes) => {
            let prefix_count = count_prefix_match(address, &prefix);
            let zero_count = count_zero_bytes(address);
            if prefix_count == prefix.len() && zero_count == num_zero_bytes {
                AddressMatch::Match
            } else {
                // prefer prefix if both are specified
                let match_count = if prefix_count == prefix.len() {
                    prefix_count + zero_count
                } else {
                    prefix_count
                };
                AddressMatch::NoMatch(match_count)
            }
        }
    }
}

fn count_zero_bytes(address: Address) -> usize {
    address.as_bytes().iter().filter(|&x| *x == 0).count()
}

fn count_prefix_match(address: Address, prefix: &str) -> usize {
    let mut count = 0;
    let mut prefix_chars = prefix.chars();
    for letter in format!("{:x}", address).chars() {
        let next = prefix_chars.next();
        if next.is_none() {
            return count;
        } else if next.unwrap() != letter {
            return count;
        } else {
            count += 1;
        }
    }
    return count;
}
