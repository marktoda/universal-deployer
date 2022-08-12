pub fn strip_hex_prefix(hex: &str) -> String {
    if hex.starts_with("0x") {
        hex.to_string().chars().skip(2).collect()
    } else {
        hex.to_string()
    }
}
