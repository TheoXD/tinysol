#[rust_sitter::grammar("solidity")]
pub mod grammar {

    #[rust_sitter::language]
    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct SourceUnit;

    // Grammar definitions go here
}