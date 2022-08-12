use crate::opt::AddressGenerationConfig;
use ethers::types::Address;

#[derive(Debug, PartialEq)]
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
            if count >= num_zero_bytes {
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
        match next {
            Some(next_letter) => {
                if next_letter == letter {
                    count += 1;
                } else {
                    return count;
                }
            }
            None => {
                return count;
            }
        }
    }
    return count;
}

#[cfg(test)]
mod tests {
    use ethers::types::{Address, U256};
    use std::str::FromStr;
    use crate::opt::AddressGenerationConfig;

    #[test]
    fn test_count_zero_bytes() {
        let address = Address::from_str("0x0000000000000000000000000000000000000000").unwrap();
        assert_eq!(20, super::count_zero_bytes(address));
        let address = Address::from_str("0xffffffffffffffffffffffffffffffffffffffff").unwrap();
        assert_eq!(0, super::count_zero_bytes(address));
        let address = Address::from_str("0xffffffff00ffffff00ffffffffffffffffffffff").unwrap();
        assert_eq!(2, super::count_zero_bytes(address));
        let address = Address::from_str("0xffffffff00ffffff000fffffffffffffffffffff").unwrap();
        assert_eq!(2, super::count_zero_bytes(address));
        let address = Address::from_str("0xffffffff00ffffff0fffffffffffffffffffffff").unwrap();
        assert_eq!(1, super::count_zero_bytes(address));
        let address = Address::from_str("0xffffffffffffffffffffffffffffffffffffff00").unwrap();
        assert_eq!(1, super::count_zero_bytes(address));
        let address = Address::from_str("0xffffffffffffffffffffffffffffffffffffff01").unwrap();
        assert_eq!(0, super::count_zero_bytes(address));
        let address = Address::from_str("0xfffffffffffffffffffffffffffffffffffffff0").unwrap();
        assert_eq!(0, super::count_zero_bytes(address));
    }

    #[test]
    fn test_count_prefix_match() {
        let address = Address::from_str("0x0000ffffffffffffffffffffffffffffffffffff").unwrap();
        assert_eq!(4, super::count_prefix_match(address, "0000"));
        assert_eq!(0, super::count_prefix_match(address, "ffff"));
        assert_eq!(2, super::count_prefix_match(address, "00ff"));
        assert_eq!(2, super::count_prefix_match(address, "00"));
        assert_eq!(0, super::count_prefix_match(address, "ff"));
        let address = Address::from_str("0xffffffffffffffffffffffffffffffffffffffff").unwrap();
        assert_eq!(0, super::count_prefix_match(address, "0000"));
    }

    #[test]
    fn test_check_address_only_prefix() {
        let config = AddressGenerationConfig {
            prefix: Some("0000".to_string()),
            num_zero_bytes: 0,
            s_start: U256::from_dec_str("1").unwrap(),
        };
        let address = Address::from_str("0x0000ffffffffffffffffffffffffffffffffffff").unwrap();
        assert_eq!(super::check_address(address, &config), super::AddressMatch::Match);
        let address = Address::from_str("0x000000ffffffffffffffffffffffffffffffffff").unwrap();
        assert_eq!(super::check_address(address, &config), super::AddressMatch::Match);
        let address = Address::from_str("0x00ffffffffffffffffffffffffffffffffffffff").unwrap();
        assert_eq!(super::check_address(address, &config), super::AddressMatch::NoMatch(2));
        let address = Address::from_str("0x0fffffffffffffffffffffffffffffffffffffff").unwrap();
        assert_eq!(super::check_address(address, &config), super::AddressMatch::NoMatch(1));
        let address = Address::from_str("0xffffffffffffffffffffffffffffffffffffffff").unwrap();
        assert_eq!(super::check_address(address, &config), super::AddressMatch::NoMatch(0));
    }

    #[test]
    fn test_check_address_only_zeroes() {
        let config = AddressGenerationConfig {
            prefix: None,
            num_zero_bytes: 4,
            s_start: U256::from_dec_str("1").unwrap(),
        };
        let address = Address::from_str("0xff00ff00ff00ff00ffffffffffffffffffffffff").unwrap();
        assert_eq!(super::check_address(address, &config), super::AddressMatch::Match);
        let address = Address::from_str("0xff00ff00ff00ff00ff00ffffffffffffffffffff").unwrap();
        assert_eq!(super::check_address(address, &config), super::AddressMatch::Match);
        let address = Address::from_str("0x00ffffffffffffffffffffffffffffffffffffff").unwrap();
        assert_eq!(super::check_address(address, &config), super::AddressMatch::NoMatch(1));
        let address = Address::from_str("0x0fffffffffffffffffffffffffffffffffff00ff").unwrap();
        assert_eq!(super::check_address(address, &config), super::AddressMatch::NoMatch(1));
        let address = Address::from_str("0xffffffffffffffffffffffffffffffffffffffff").unwrap();
        assert_eq!(super::check_address(address, &config), super::AddressMatch::NoMatch(0));
        let address = Address::from_str("0xf0ff0fff0ff0fff0ffffffffffffffffffffffff").unwrap();
        assert_eq!(super::check_address(address, &config), super::AddressMatch::NoMatch(0));
    }
}
