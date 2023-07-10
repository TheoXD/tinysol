pub mod solidity;
use crate::solidity::grammar::*;

fn main() {
    let code = std::fs::read_to_string("./contracts/empty.sol").expect("Unable to read source file");
    let parsed = parse(code.as_str());
    println!("{:#?}", parsed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let code = std::fs::read_to_string("./contracts/empty.sol").expect("Unable to read source file");
        let parsed = parse(code.as_str());
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), SourceUnit );
    }
}