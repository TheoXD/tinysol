use ethnum::U256;
use std::collections::HashMap;
use crate::solidity::grammar::*;
use keccak_hash::{keccak};

pub struct Stack {
    stackarr: [U256; 1024],
    top: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            stackarr: [U256::ZERO; 1024],
            top: 0,
        }
    }

    pub fn push32(&mut self, value: U256) {
        if self.top < 1024 {
            self.stackarr[self.top] = value;
            self.top += 1;
        }
    }

    pub fn push1(&mut self, value: u8) {
        self.push32(U256::from(value));
    }

    pub fn pop(&mut self) -> Option<U256> {
        if self.top == 0 {
            None
        } else {
            self.top -= 1;
            Some(self.stackarr[self.top])
        }   //no semicolon in Rust means this expression is returned, 
            //and it will return either None or Some() depending on the condition
    }

    pub fn swap(&mut self) {
        self.stackarr.swap(self.top - 1, self.top - 2);
    }
}

#[derive(Debug, Clone)]
pub enum OP {
    PUSH32(U256),
    PUSH1(u8),
    POP,
    DUP1,
    SWAP1,
    SLOAD,
    SSTORE,
    ISZERO,
    RETURN,
}

#[derive(Debug, Clone, Default)]
pub struct ContractStorage {
    slots: Vec<U256>
}

pub struct VM<'a> {
    pub stack: Stack,
    program: Vec<OP>,
    pc: usize,
    calldata: &'a [u8],
}

impl<'a> VM<'a> {
    pub fn new(program: Vec<OP>, calldata: &'a [u8]) -> Self {
        Self {
            stack: Stack::new(),
            program,
            pc: 0,
            calldata: calldata,
        }
    }

    pub fn run(&mut self, storage: ContractStorage) -> ContractStorage {
        let mut storage = storage;
        while self.pc < self.program.len() {
            match self.program[self.pc] {
                OP::PUSH32(word) => {
                    self.stack.push32(word);
                    self.pc += 1;
                },
                OP::PUSH1(value) => {
                    self.stack.push1(value);
                    self.pc += 1;
                },
                OP::POP => {
                    self.stack.pop();
                    self.pc += 1;
                },
                OP::SWAP1 => {
                    self.stack.swap();
                    self.pc += 1;
                },
                OP::DUP1 => {
                    let top = self.stack.pop().unwrap();
                    self.stack.push32(top);
                    self.stack.push32(top);
                    self.pc += 1;
                },
                OP::SLOAD => {
                    let key = self.stack.pop().unwrap();
                    let val = storage.slots[key.as_usize()];
                    self.stack.push32(val);
                    self.pc += 1;
                },
                OP::SSTORE => {
                    let key = self.stack.pop().unwrap();
                    let val = self.stack.pop().unwrap();
                    storage.slots[key.as_usize()] = val;
                    self.pc += 1;
                },
                OP::RETURN => {
                    self.pc += 1;
                    break;
                },
                OP::ISZERO => {
                    let top = self.stack.pop().unwrap();

                    if top == U256::ZERO {
                        self.stack.push32(U256::ONE);
                    } else {
                        self.stack.push32(U256::ZERO);
                    }
                    self.pc += 1;
                },
            }
        };
        storage
    }
}

#[derive(Debug, Default, Clone)]
pub struct Contract {
    pub name: String,
    pub functions: HashMap<String, Function>,
    pub variable_map: HashMap<String, usize>,
    pub storage: ContractStorage,
}

impl Contract {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Contract::default()
        }
    }

    pub fn call(&self, calldata: &str) -> (Contract, Vec<Expression>) {
        match self.functions.get(&calldata.to_string()) {
            Some(function) => {
                let mut vm = VM::new(function.program.clone(), calldata.as_bytes());
                let new_storage = vm.run(self.storage.clone());
        
                //Read return values from stack
                let mut ret: Vec<Expression> = vec![];
                function.returns.iter().for_each(|param| {
                    if let Some(r) = vm.stack.pop() {
                        match param {
                            Parameter { ty: Expression::Type(Type::Bool(_)), .. } => {
                                    ret.push(Expression::BoolLiteral(r == U256::ONE));
                            },
                            _ => {},
                        }
                    }
                });
        
                (Contract {
                    storage: if let FuncMutability::View | FuncMutability::Pure = function.mutability { self.storage.clone() } else { new_storage },
                    ..self.clone()
                }, ret)
            }
            None => {
                return (self.clone(), vec![]);
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Function {
    program: Vec<OP>,
    pub visibility: FuncVisibility,
    pub mutability: FuncMutability,
    pub returns: Vec<Parameter>,
}

#[derive(Debug, Clone, Default)]
pub enum FuncVisibility {
    Public,
    Private,
    #[default]
    Internal,
    External,
}
#[derive(Debug, Clone, Default)]
pub enum FuncMutability {
    Constant,
    #[default]
    NonPayable,
    Payable,
    View,
    Pure,
}

pub fn create_contracts(source_unit: SourceUnit) -> Vec<Contract> {
    handle_source_unit(source_unit)
}

fn handle_source_unit(source_unit: SourceUnit) -> Vec<Contract> {
    source_unit.parts.iter().flat_map(|part| handle_source_unit_part(part.clone())).collect::<Vec<Contract>>()
}

fn handle_source_unit_part(part: SourceUnitPart) -> Option<Contract> {
    match part {
        SourceUnitPart::ContractDefinition(_, name, _, parts, _) => {
            let mut contract = Contract::new(name);
            let _ = parts.iter().map(|part| handle_contract_part(part.clone(), &mut contract)).collect::<Vec<_>>();
            Some(contract)
        },
        _ => None,
    }
}

fn handle_contract_part(part: ContractPart, contract: &mut Contract) {
    match part {
        ContractPart::FunctionDefinition(_, name, params, attr_list, ret_params, _, statement, _) => {
            if let Some(statement) = statement {
                //TODO: handle function arguments
                let program = handle_statement(statement, contract);
                
                let (visibility, mutability) = handle_attrs(attr_list.clone());
                
                let mut returns = vec![];
                if let Some(FunctionReturnParams::ParameterList(_, ParameterList::Params(_, Some(ret_param), _))) = ret_params.clone() {
                    returns = ret_param.params;
                }

                contract.functions.insert(
                    find_function_signature(name.clone(), params.clone()),
                    Function {
                        program: program,
                        visibility: visibility,
                        mutability: mutability,
                        returns: returns,
                        ..Function::default()
                    }
                );
            }
        },
        ContractPart::VariableDefinition(ty, visibility, name, _) => {
            contract.variable_map.insert(name, contract.variable_map.len());
            contract.storage.slots.push(U256::ZERO);
        },
        ContractPart::ConstructorDefinition(_, params, attr_list, _, statement, _) => {
            //TODO
        }
    }
}

fn handle_attrs(attr_list: Vec<Option<FunctionAttribute>>) -> (FuncVisibility, FuncMutability) {
    let mut visibility = FuncVisibility::default();
    let mut mutability = FuncMutability::default();

    attr_list.iter().for_each(|attr| {
        if let Some(attr) = attr {
            match attr {
                FunctionAttribute::Visibility(v) => {
                    visibility = match v {
                        Visibility::Public(_) => FuncVisibility::Public,
                        Visibility::Private(_) => FuncVisibility::Private,
                        Visibility::Internal(_) => FuncVisibility::Internal,
                        Visibility::External(_) => FuncVisibility::External,
                    }
                },
                FunctionAttribute::Mutability(m) => {
                    mutability = match m {
                        Mutability::Constant(_) => FuncMutability::Constant,
                        Mutability::Payable(_) => FuncMutability::Payable,
                        Mutability::View(_) => FuncMutability::View,
                        Mutability::Pure(_) => FuncMutability::Pure,
                    }
                },
            }
        }
    });
    (visibility, mutability)
}

fn handle_statement(statement: Statement, contract: &mut Contract) -> Vec<OP> {
    match statement {
        Statement::Expression(expr, _) => {
            handle_expression(expr, contract)
        },
        Statement::Return(_, expr, _) => {
            match expr {
                Some(expr) => [handle_expression(expr, contract), vec![OP::RETURN]].concat(),
                None => vec![OP::RETURN],
            }
        },
    }
}

fn handle_expression(expr: Expression, contract: &mut Contract) -> Vec<OP> {
    match expr {
        Expression::BoolLiteral(val) => {
            vec![]
        },
        Expression::Variable(identifier) => {
            let mut slot = 0;
            if let Some(found) = contract.variable_map.get(&identifier.name.clone()) {
                slot = *found;
            }

            vec![
                OP::PUSH1(slot as u8),
                OP::SLOAD
            ]
        },
        Expression::Assign(left, _, right) => {
            if let Expression::Variable(identifier) = *left {
                let mut slot = 0;
                if let Some(found) = contract.variable_map.get(&identifier.name.clone()) {
                    slot = *found;
                }
                [handle_expression(*right, contract),
                vec![OP::PUSH1(slot as u8), OP::SSTORE]].concat()
            } else {
                vec![]
            }
        },
        Expression::Not(_, expr) => {
            [handle_expression(*expr, contract), vec![OP::ISZERO]].concat()
        },
        Expression::Type(ty) => {
            match ty {
                Type::Bool(_) => vec![], //TODO
                _ => vec![],
            }
        },
    }
}

fn find_function_signature(name: String, params: ParameterList) -> String {
    if let ParameterList::Params((), Some(p), ()) = params {
        let params_string = p.params.iter().map(|param| {
            match param.ty {
                Expression::Type(Type::Bool(_)) => Some("bool"),
                _ => None,
            }
        })
        .collect::<Option<Vec<&str>>>().map(|v| v.join(",")).unwrap_or_default();

        get_func_sig(format!("{}({})", name, params_string))
    } else {
        get_func_sig(format!("{}()", name))
    }
}

pub fn get_func_sig(in_str: String) -> String {
    keccak(in_str.as_bytes())[..4].to_vec().iter().map(|b| format!("{:02x}", b)).collect::<String>()
}