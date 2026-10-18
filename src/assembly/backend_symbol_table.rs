use std::collections::HashMap;
use crate::assembly::assembly_tokens::ASMType;
use crate::parser::ast_tokens::ASTType;
use crate::parser::typechecker::Symbol;

#[derive(Debug, Clone, PartialEq)]
pub enum ASMSymtabEntry {
    ObjEntry { _type: ASMType, is_static: bool },
    FunEntry { defined: bool },
}

pub fn generate_backend_symbol_table(symbols: &HashMap<String, Symbol>) -> HashMap<String, ASMSymtabEntry> {
    let mut backend_symbol_table = HashMap::new();

    for (_, symbol) in symbols {
        let sym_name = symbol.name.clone();
        let entry = match symbol._type {
            ASTType::Int => {
                ASMSymtabEntry::ObjEntry {
                    _type: ASMType::Longword,
                    is_static: symbol.identifier_attrs.is_static()
                }
            }
            ASTType::Long => {
                ASMSymtabEntry::ObjEntry {
                    _type: ASMType::Quadword,
                    is_static: symbol.identifier_attrs.is_static()
                }
            }
            ASTType::FunType(_, _) => {
                ASMSymtabEntry::FunEntry { defined: symbol.identifier_attrs.is_defined() }
            }
        };
        backend_symbol_table.insert(sym_name, entry);
    }

    backend_symbol_table
}

impl ASMSymtabEntry {
    pub fn is_static(&self) -> bool {
        match self {
            ASMSymtabEntry::ObjEntry { is_static, .. } => *is_static,
            ASMSymtabEntry::FunEntry { defined } => panic!("Called is_static on FunEntry: {:?}", self),
        }
    }
    
    pub fn get_type(&self) -> ASMType {
        match self {
            ASMSymtabEntry::ObjEntry { _type, .. } => _type.clone(),
            ASMSymtabEntry::FunEntry { .. } => panic!("Called get_type on FunEntry: {:?}", self),
        }
    }
}