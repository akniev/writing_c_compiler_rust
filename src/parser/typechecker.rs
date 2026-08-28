use std::collections::HashMap;
use crate::parser::ast_tokens::{ASTBlock, ASTBlockItem, ASTDeclaration, ASTExpression, ASTForInit, ASTFunctionDeclaration, ASTProgram, ASTStatement, ASTStorageClass, ASTVariableDeclaration};

pub fn typecheck(program: ASTProgram) -> HashMap<String, Symbol> {
    let mut symbols = HashMap::new();
    for decl in program.declarations {
        typecheck_declaration(&decl, &mut symbols, false);
    }
    symbols
}

fn typecheck_declaration(
    declaration: &ASTDeclaration,
    symbols: &mut HashMap<String, Symbol>,
    block_scope: bool,
) {
    match declaration {
        ASTDeclaration::FunctionDeclaration(function_declaration) => {
            typecheck_function_declaration(function_declaration.clone(), symbols);
        }
        ASTDeclaration::VariableDeclaration(variable_declaration) => {
            if block_scope {
                typecheck_local_variable_declaration(variable_declaration, symbols);
            } else {
                typecheck_file_scope_variable_declaration(variable_declaration, symbols);
            }
        }
    }
}

fn typecheck_function_declaration(function_declaration: ASTFunctionDeclaration, symbols: &mut HashMap<String, Symbol>) {
    let fun_type = SymbolType::FunType(function_declaration.params.len() as i32);
    let has_body = function_declaration.body.is_some();
    let mut already_defined = false;
    let mut global = function_declaration.storage_class != Some(ASTStorageClass::Static);

    if symbols.contains_key(&function_declaration.name) {
        let old_decl = symbols.get(&function_declaration.name).unwrap();
        if old_decl._type != fun_type {
            panic!("Function {} already defined with different type", function_declaration.name);
        }
        already_defined = old_decl.identifier_attrs.is_defined();
        if already_defined && has_body {
            panic!("Function {} already defined with body", function_declaration.name);
        }

        if old_decl.identifier_attrs.is_global() && function_declaration.storage_class == Some(ASTStorageClass::Static) {
            panic!("Static function declaration follows non-static");
        }
        global = old_decl.identifier_attrs.is_global();
    }

    let attrs = SymbolAttrs::FunAttr { defined: already_defined || has_body, global };

    symbols.insert(function_declaration.name.clone(), Symbol {
        name: function_declaration.name,
        _type: fun_type,
        identifier_attrs: attrs,
    });

    if has_body {
        for param in function_declaration.params {
            symbols.insert(param.clone(), Symbol { name: param, _type: SymbolType::Int, identifier_attrs: SymbolAttrs::LocalAttr });
        }
        typecheck_block(function_declaration.body.unwrap(), symbols);
    }
}

fn typecheck_local_variable_declaration(
    variable_declaration: &ASTVariableDeclaration,
    symbols: &mut HashMap<String, Symbol>
) {
    if variable_declaration.storage_class == Some(ASTStorageClass::Extern) {
        if variable_declaration.init.is_some() {
            panic!("Initializer on local extern variable declaration");
        }
        if symbols.contains_key(&variable_declaration.name.clone()) {
            let old_decl = symbols.get(&variable_declaration.name).unwrap();
            if old_decl._type != SymbolType::Int {
                panic!("Function redeclared as variable");
            }
        } else {
            symbols.insert(
                variable_declaration.name.clone(),
                Symbol {
                    name: variable_declaration.name.clone(),
                    _type: SymbolType::Int,
                    identifier_attrs: SymbolAttrs::StaticAttr {
                        init: SymbolInitialValue::NoInitializer,
                        global: true
                    }
                }
            );
        }
    } else if variable_declaration.storage_class == Some(ASTStorageClass::Static) {
        let initial_value = if let Some(ASTExpression::Constant(init_val)) = variable_declaration.init {
            SymbolInitialValue::Initial(init_val)
        } else if variable_declaration.init.is_none() {
            SymbolInitialValue::Initial(0)
        } else {
            panic!("Non-constant initializer on local static variable")
        };
        symbols.insert(
            variable_declaration.name.clone(),
            Symbol {
                name: variable_declaration.name.clone(),
                _type: SymbolType::Int,
                identifier_attrs: SymbolAttrs::StaticAttr { init: initial_value, global: false }
            }
        );
    } else {
        symbols
            .insert(
                variable_declaration.clone().name,
                Symbol {
                    name: variable_declaration.name.clone(),
                    _type: SymbolType::Int,
                    identifier_attrs: SymbolAttrs::LocalAttr
                }
            );
        if let Some(init) = variable_declaration.clone().init {
            typecheck_expression(init, symbols);
        }
    }
}

fn typecheck_file_scope_variable_declaration(
    variable_declaration: &ASTVariableDeclaration,
    symbols: &mut HashMap<String, Symbol>
) {
    let mut initial_value = if let Some(ASTExpression::Constant(init_value)) = variable_declaration.init {
        SymbolInitialValue::Initial(init_value)
    } else if variable_declaration.init.is_none() {
        if variable_declaration.storage_class == Some(ASTStorageClass::Extern) {
            SymbolInitialValue::NoInitializer
        } else {
            SymbolInitialValue::Tentative
        }
    } else {
        panic!("Non-constant initializer!")
    };

    let mut global = variable_declaration.storage_class != Some(ASTStorageClass::Static);

    if symbols.contains_key(&variable_declaration.name) {
        let old_decl = symbols.get(&variable_declaration.name).unwrap();
        if old_decl._type != SymbolType::Int {
            panic!("Function redeclared as variable");
        }
        if variable_declaration.storage_class == Some(ASTStorageClass::Extern) {
            global = old_decl.identifier_attrs.is_global();
        } else if old_decl.identifier_attrs.is_global() != global {
            panic!("Conflicting variable linkage")
        }

        let old_init = old_decl.identifier_attrs.get_init();

        if matches!(old_init, Some(SymbolInitialValue::Initial(_))) {
            if matches!(initial_value, SymbolInitialValue::Initial(_)) {
                panic!("Conflicting file scope variable definitions");
            } else {
                initial_value = old_init.unwrap();
            }
        } else if !matches!(initial_value, SymbolInitialValue::Initial(_)) && matches!(old_init, Some(SymbolInitialValue::Tentative)) {
            initial_value = SymbolInitialValue::Tentative;
        }
    }

    let attrs = SymbolAttrs::StaticAttr { init: initial_value, global };
    symbols.insert(
        variable_declaration.name.clone(),
        Symbol {
            name: variable_declaration.name.clone(),
            _type: SymbolType::Int,
            identifier_attrs: attrs
        }
    );
}

fn typecheck_block(block: ASTBlock, symbols: &mut HashMap<String, Symbol>) {
    for block_item in block.items {
        match block_item {
            ASTBlockItem::Declaration(declaration) => {
                typecheck_declaration(&declaration, symbols, true);
            }
            ASTBlockItem::Statement(statement) => {
                typecheck_statement(statement, symbols);
            }
        }
    }
}

fn typecheck_statement(statement: ASTStatement, symbols: &mut HashMap<String, Symbol>) {
    match statement {
        ASTStatement::Return(exp) => {
            typecheck_expression(exp, symbols);
        }
        ASTStatement::Expression(exp) => {
            typecheck_expression(exp, symbols);
        }
        ASTStatement::If { condition, then_statement, else_statement } => {
            typecheck_expression(condition, symbols);
            typecheck_statement(then_statement.as_ref().clone(), symbols);
            if let Some(else_statement) = else_statement {
                typecheck_statement(else_statement.as_ref().clone(), symbols);
            }
        }
        ASTStatement::Compound(block) => {
            typecheck_block(block, symbols)
        }
        ASTStatement::While { condition, body, label: _ } => {
            typecheck_expression(condition, symbols);
            typecheck_statement(body.as_ref().clone(), symbols);
        }
        ASTStatement::DoWhile { body, condition, label: _ } => {
            typecheck_statement(body.as_ref().clone(), symbols);
            typecheck_expression(condition, symbols);
        }
        ASTStatement::For { init, condition, post, body, label: _ } => {
            match init {
                ASTForInit::InitExp(exp) => {
                    if let Some(exp) = exp {
                        typecheck_expression(exp, symbols);
                    }
                }
                ASTForInit::InitDecl(variable_declaration) => {
                    if variable_declaration.storage_class.is_some() {
                        panic!("For loop variable declaration with storage class");
                    }
                    typecheck_local_variable_declaration(&variable_declaration, symbols);
                }
            }
            if let Some(condition) = condition {
                typecheck_expression(condition, symbols);
            }
            if let Some(post) = post {
                typecheck_expression(post, symbols);
            }
            typecheck_statement(body.as_ref().clone(), symbols);
        }
        ASTStatement::Switch { condition, body, label: _ } => {
            typecheck_expression(condition, symbols);
            typecheck_statement(body.as_ref().clone(), symbols);
        }
        ASTStatement::Case { value, body, label: _ } => {
            typecheck_expression(value, symbols);
            typecheck_statement(body.as_ref().clone(), symbols);
        }
        ASTStatement::Default { body, label: _ } => {
            typecheck_statement(body.as_ref().clone(), symbols);
        }
        ASTStatement::LabelledStatement { label: _, statement } => {
            typecheck_statement(statement.as_ref().clone(), symbols);
        }
        ASTStatement::Break { label: _ } | ASTStatement::Continue { label: _ } | ASTStatement::Goto { label: _ } | ASTStatement::Null => {
            // Do nothing
        }
    }
}

fn typecheck_expression(expression: ASTExpression, symbols: &mut HashMap<String, Symbol>) {
    match expression {
        ASTExpression::Var(name) => {
            if symbols.get(&name).unwrap()._type != SymbolType::Int {
                panic!("{} is not an integer", name);
            }
        }
        ASTExpression::Constant(_) => {
            // Do nothing
        }
        ASTExpression::Unary { op: _, exp } => {
            typecheck_expression(exp.as_ref().clone(), symbols);
        }
        ASTExpression::Postfix { op: _, exp } => {
            typecheck_expression(exp.as_ref().clone(), symbols);
        }
        ASTExpression::Binary { op: _, lhs, rhs } => {
            typecheck_expression(lhs.as_ref().clone(), symbols);
            typecheck_expression(rhs.as_ref().clone(), symbols);
        }
        ASTExpression::Assignment { lhs, rhs } => {
            typecheck_expression(lhs.as_ref().clone(), symbols);
            typecheck_expression(rhs.as_ref().clone(), symbols);
        }
        ASTExpression::CompoundAssignment { lhs, rhs, operator: _ } => {
            typecheck_expression(lhs.as_ref().clone(), symbols);
            typecheck_expression(rhs.as_ref().clone(), symbols);
        }
        ASTExpression::Conditional { condition, if_true, if_false } => {
            typecheck_expression(condition.as_ref().clone(), symbols);
            typecheck_expression(if_true.as_ref().clone(), symbols);
            typecheck_expression(if_false.as_ref().clone(), symbols);
        }
        ASTExpression::FunctionCall { name, args } => {
            let f_type = symbols.get(&name).unwrap().clone()._type;
            if f_type == SymbolType::Int {
                panic!("{} is not a function", name);
            }
            if let SymbolType::FunType(param_count) = f_type && param_count != args.len() as i32 {
                panic!("{} expects {} arguments, got {}", name, param_count, args.len());
            }
            for arg in args {
                typecheck_expression(arg, symbols);
            }
        }

    }
}



// MARK: - Helpers

#[derive(Clone, Debug, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub _type: SymbolType,
    pub identifier_attrs: SymbolAttrs,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SymbolType {
    Int,
    FunType(i32)
}

#[derive(Clone, Debug, PartialEq)]
pub enum SymbolAttrs {
    FunAttr { defined: bool, global: bool },
    StaticAttr { init: SymbolInitialValue, global: bool },
    LocalAttr,
}

impl SymbolAttrs {
    pub fn is_global(&self) -> bool {
        match self {
            SymbolAttrs::FunAttr { global, .. } => *global,
            SymbolAttrs::StaticAttr { global, .. } => *global,
            SymbolAttrs::LocalAttr => false,
        }
    }

    pub fn is_defined(&self) -> bool {
        match self {
            SymbolAttrs::FunAttr { defined, .. } => *defined,
            SymbolAttrs::StaticAttr { init: _, .. } => {
                panic!("Not implemented yet")
            },
            SymbolAttrs::LocalAttr => true,
        }
    }

    pub fn get_init(&self) -> Option<SymbolInitialValue> {
        match self {
            SymbolAttrs::FunAttr { .. } => None,
            SymbolAttrs::StaticAttr { init, .. } => Some(init.clone()),
            SymbolAttrs::LocalAttr => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SymbolInitialValue {
    Tentative,
    Initial(i32),
    NoInitializer,
}