#[rust_sitter::grammar("solidity")]
pub mod grammar {

    #[rust_sitter::language]
    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct SourceUnit{
        pub parts: Vec<SourceUnitPart>
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum SourceUnitPart {
        ContractDefinition(
            #[rust_sitter::leaf(text = "contract")] (),
            #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |s| s.to_string())] String,
            #[rust_sitter::leaf(text = "{")] (),
            Vec<ContractPart>,
            #[rust_sitter::leaf(text = "}")] (),
        )
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum ContractPart {
        VariableDefinition(
            Type,
            Option<Visibility>,
            #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |s| s.to_string())] String,
            //TODO: Add initializer expression
            #[rust_sitter::leaf(text = ";")] (),
        ),
        FunctionDefinition(
            #[rust_sitter::leaf(text = "function")] (),
            #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |s| s.to_string())] String,

            ParameterList,

            #[rust_sitter::repeat(non_empty = true)]
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = " ")] ()
            )]
            Vec<Option<FunctionAttribute>>,
            
            Option<FunctionReturnParams>,
            
            #[rust_sitter::leaf(text = "{")] (),
            Option<Statement>,
            #[rust_sitter::leaf(text = "}")] (),
        ),
        ConstructorDefinition(
            #[rust_sitter::leaf(text = "constructor")] (),

            ParameterList,

            #[rust_sitter::repeat(non_empty = true)]
            #[rust_sitter::delimited(
                #[rust_sitter::leaf(text = " ")] ()
            )]
            Vec<Option<FunctionAttribute>>,
            
            #[rust_sitter::leaf(text = "{")] (),
            Option<Statement>,
            #[rust_sitter::leaf(text = "}")] (),
        )
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum Statement {
        Expression(
            Expression,
            #[rust_sitter::leaf(text = ";")] (),
        ),
        Return(
            #[rust_sitter::leaf(text = "return")] (),
            Option<Expression>,
            #[rust_sitter::leaf(text = ";")] (),
        )
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum ParameterList {
        Param(
            #[rust_sitter::leaf(text = "(")] (),
            Option<Parameter>,
            #[rust_sitter::leaf(text = ")")] ()
        )
        //TODO: capture multiple parameters
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum FunctionReturnParams {
        ParameterList(
            #[rust_sitter::leaf(text = "returns")] (),
            ParameterList,
        )
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct Parameter {
        pub ty: Expression,
        //TODO: add storage
        pub name: Option<Identifier>
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |s| s.to_string())]
        pub name: String
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum Expression {
        BoolLiteral(
            #[rust_sitter::leaf(pattern = r"true|false", transform = |v| v.parse::<bool>().unwrap())]
            bool,
        ),
        Variable(Identifier),
        #[rust_sitter::prec_right(1)]
        Assign(
            Box<Expression>,
            #[rust_sitter::leaf(text = "=")] (),
            Box<Expression>,
        ),
        Not(
            #[rust_sitter::leaf(text = "!")] (),
            Box<Expression>,
        ),
        Type(Type)
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum FunctionAttribute {
        Visibility(Visibility),
        Mutability(Mutability)
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum Mutability {
        Pure(#[rust_sitter::leaf(text = "pure")] ()),
        View(#[rust_sitter::leaf(text = "view")] ()),
        Constant(#[rust_sitter::leaf(text = "constant")] ()),
        Payable(#[rust_sitter::leaf(text = "payable")] ())
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum Type {
        Bool(#[rust_sitter::leaf(text = "bool")] ())
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum Visibility {
        Internal(#[rust_sitter::leaf(text = "internal")] ()),
        External(#[rust_sitter::leaf(text = "external")] ()), //External keyword only applies to functions
        Private(#[rust_sitter::leaf(text = "private")] ()),
        Public(#[rust_sitter::leaf(text = "public")] ())
    }

    // ------------- Extras -------------

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }

    #[rust_sitter::extra]
    struct Newline {
        #[rust_sitter::leaf(pattern = r"\n")]
        _new_line: (),
    }

    #[rust_sitter::extra]
    struct SingleLineComment {
        #[rust_sitter::leaf(pattern = r"//.*")]
        _comment: (),
    }
}