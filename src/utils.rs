
fn to_binary(c: char) -> &'static str {
    match c {
        '0' => "0000",
        '1' => "0001",
        '2' => "0010",
        '3' => "0011",
        '4' => "0100",
        '5' => "0101",
        '6' => "0110",
        '7' => "0111",
        '8' => "1000",
        '9' => "1001",
        'a' => "1010",
        'b' => "1011",
        'c' => "1100",
        'd' => "1101",
        'e' => "1110",
        'f' => "1111",
        _ => "",
    }
}

fn convert_to_binary_from_hex(hex: &str) -> String {
    hex.chars().map(to_binary).collect()
}

/// Get is matched difficulty hash.
pub fn get_is_hash_matches_difficulty(hash: &str, difficulty: usize) -> bool {
    let hash_in_binary = convert_to_binary_from_hex(hash);
    let required_prefix = "0".repeat(difficulty);
    hash_in_binary.starts_with(&required_prefix)
}

pub fn from_hex(hex: &str, target: &mut [u8]) -> Result<usize, ()> {
    if hex.len() % 2 == 1 || hex.len() > target.len() * 2 {
        return Err(());
    }

    let mut b = 0;
    let mut idx = 0;
    for c in hex.bytes() {
        b <<= 4;
        match c {
            b'A'..=b'F' => b |= c - b'A' + 10,
            b'a'..=b'f' => b |= c - b'a' + 10,
            b'0'..=b'9' => b |= c - b'0',
            _ => return Err(()),
        }
        if (idx & 1) == 1 {
            target[idx / 2] = b;
            b = 0;
        }
        idx += 1;
    }
    Ok(idx / 2)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_binary() {
        assert_eq!(to_binary('0').to_string(), "0000".to_string());
        assert_eq!(to_binary('1').to_string(), "0001".to_string());
        assert_eq!(to_binary('2').to_string(), "0010".to_string());
        assert_eq!(to_binary('3').to_string(), "0011".to_string());
        assert_eq!(to_binary('4').to_string(), "0100".to_string());
        assert_eq!(to_binary('5').to_string(), "0101".to_string());
        assert_eq!(to_binary('6').to_string(), "0110".to_string());
        assert_eq!(to_binary('7').to_string(), "0111".to_string());
        assert_eq!(to_binary('8').to_string(), "1000".to_string());
        assert_eq!(to_binary('9').to_string(), "1001".to_string());
        assert_eq!(to_binary('a').to_string(), "1010".to_string());
        assert_eq!(to_binary('b').to_string(), "1011".to_string());
        assert_eq!(to_binary('c').to_string(), "1100".to_string());
        assert_eq!(to_binary('d').to_string(), "1101".to_string());
        assert_eq!(to_binary('e').to_string(), "1110".to_string());
        assert_eq!(to_binary('f').to_string(), "1111".to_string());
    }

    #[test]
    fn test_convert_to_binary_from_hex() {
        assert_eq!(convert_to_binary_from_hex("abcd").to_string(), "1010101111001101".to_string());
    }

    #[test]
    fn test_hash_matches_difficulty() {
        assert!(get_is_hash_matches_difficulty("abcd", 0));
        assert!(!get_is_hash_matches_difficulty("abcd", 1));
        assert!(get_is_hash_matches_difficulty("1bcd", 3));
        assert!(!get_is_hash_matches_difficulty("2bcd", 3));
        assert!(get_is_hash_matches_difficulty("0000", 16));
    }
}
