pub fn hex_to_binary(hex: &[u8]) -> String {
    let mut result = String::new();

    for symbol in hex {
        result.push_str(&format!("{:b}", symbol));
    }

    result
}
