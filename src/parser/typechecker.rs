use std::collections::HashMap;
use crate::parser::ast_tokens::{ASTBlock, ASTBlockItem, ASTDeclaration, ASTExpression, ASTForInit, ASTFunctionDeclaration, ASTProgram, ASTStatement, ASTVariableDeclaration};

pub fn typecheck(program: ASTProgram) -> HashMap<String, Symbol> {
    let mut symbols = HashMap::new();
    for fun_decl in program.functions {
        typecheck_function_declaration(fun_decl, &mut symbols);
    }
    symbols
}

fn typecheck_function_declaration(function_declaration: ASTFunctionDeclaration, symbols: &mut HashMap<String, Symbol>) {
    let fun_type = Type::FunType(function_declaration.params.len() as i32);
    let has_body = function_declaration.body.is_some();
    let mut already_defined = false;

    if symbols.contains_key(&function_declaration.name) {
        let old_decl = symbols.get(&function_declaration.name).unwrap();
        if old_decl._type != fun_type {
            panic!("Function {} already defined with different type", function_declaration.name);
        }
        already_defined = old_decl.defined;
        if already_defined && has_body {
            panic!("Function {} already defined with body", function_declaration.name);
        }
    }

    symbols.insert(function_declaration.name.clone(), Symbol {
        name: function_declaration.name,
        _type: fun_type,
        defined: already_defined || has_body,
    });

    if has_body {
        for param in function_declaration.params {
            symbols.insert(param.clone(), Symbol { name: param, _type: Type::Int, defined: false });
        }
        typecheck_block(function_declaration.body.unwrap(), symbols);
    }
}

fn typecheck_block(block: ASTBlock, symbols: &mut HashMap<String, Symbol>) {
    for block_item in block.items {
        match block_item {
            ASTBlockItem::Declaration(declaration) => {
                typecheck_declaration(&declaration, symbols);
            }
            ASTBlockItem::Statement(statement) => {
                typecheck_statement(statement, symbols);
            }
        }
    }
}

fn typecheck_declaration(declaration: &ASTDeclaration, symbols: &mut HashMap<String, Symbol>) {
    match declaration {
        ASTDeclaration::FunctionDeclaration(function_declaration) => {
            typecheck_function_declaration(function_declaration.clone(), symbols);
        }
        ASTDeclaration::VariableDeclaration(variable_declaration) => {
            typecheck_variable_declaration(variable_declaration, symbols);
        }
    }
}

fn typecheck_variable_declaration(variable_declaration: &ASTVariableDeclaration, symbols: &mut HashMap<String, Symbol>) {
    symbols
        .insert(
            variable_declaration.clone().name,
            Symbol { name: variable_declaration.name.clone(), _type: Type::Int, defined: true }
        );
    if let Some(init) = variable_declaration.clone().init {
        typecheck_expression(init, symbols);
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
                    typecheck_variable_declaration(&variable_declaration, symbols);
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
            if symbols.get(&name).unwrap()._type != Type::Int {
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
            if f_type == Type::Int {
                panic!("{} is not a function", name);
            }
            if let Type::FunType(param_count) = f_type && param_count != args.len() as i32 {
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
    pub _type: Type,
    pub defined: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Int,
    FunType(i32)
}