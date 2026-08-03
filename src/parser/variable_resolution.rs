use std::collections::HashMap;
use crate::common::global_counter::get_unique_variable_name;
use crate::parser::ast_tokens::{ASTBlockItem, ASTDeclaration, ASTExpression, ASTFunctionDefinition, ASTProgram, ASTStatement};

pub fn resolve_variables(program: ASTProgram) -> ASTProgram {
    let mut variable_map: HashMap<String, String> = HashMap::new();
    let new_function_definition = resolve_function_definition(program.function_definition, &mut variable_map);
    ASTProgram { function_definition: new_function_definition }
}

fn resolve_function_definition(
    function_definition: ASTFunctionDefinition,
    variable_map: &mut HashMap<String, String>,
) -> ASTFunctionDefinition {
    let mut new_body = vec![];

    for block_item in function_definition.body {
        new_body.push(resolve_block_item(block_item, variable_map))
    }

    ASTFunctionDefinition { name: function_definition.name, body: new_body }
}

fn resolve_block_item(
    block_item: ASTBlockItem,
    variable_map: &mut HashMap<String, String>,
) -> ASTBlockItem {
    match block_item {
        ASTBlockItem::Statement(statement) => {
            let new_statement = resolve_statement(statement, variable_map);
            ASTBlockItem::Statement(new_statement)
        }
        ASTBlockItem::Declaration(declaration) => {
            let new_declaration = resolve_declaration(declaration, variable_map);
            ASTBlockItem::Declaration(new_declaration)
        }
    }
}

fn resolve_statement(
    statement: ASTStatement,
    variable_map: &mut HashMap<String, String>
) -> ASTStatement {
    match statement {
        ASTStatement::Return(exp) => {
            let new_exp = resolve_expression(exp, variable_map);
            ASTStatement::Return(new_exp)
        }
        ASTStatement::Expression(exp) => {
            let new_exp = resolve_expression(exp, variable_map);
            ASTStatement::Expression(new_exp)
        }
        ASTStatement::Null => {
            ASTStatement::Null
        }
    }
}

fn resolve_declaration(
    declaration: ASTDeclaration,
    variable_map: &mut HashMap<String, String>,
) -> ASTDeclaration {
    match declaration {
        ASTDeclaration::VariableDeclaration { name, init } => {
            if variable_map.contains_key(&name) {
                panic!("Variable {} is already defined", name);
            }
            let unique_name = get_unique_variable_name(&name);
            variable_map.insert(name, unique_name.clone());
            let new_init = init.map(|init| resolve_expression(init, variable_map));
            ASTDeclaration::VariableDeclaration { name: unique_name, init: new_init }
        }
    }
}

fn resolve_expression(
    exp: ASTExpression,
    variable_map: &mut HashMap<String, String>,
) -> ASTExpression {
    match exp {
        ASTExpression::Var(name) => {
            if let Some(unique_name) = variable_map.get(&name) {
                return ASTExpression::Var(unique_name.clone());
            } else {
                panic!("Variable {} is not defined", name);
            }
        }
        ASTExpression::Constant(int) => {
            ASTExpression::Constant(int)
        }
        ASTExpression::Unary { op, exp } => {
            let new_exp = resolve_expression(*exp, variable_map);
            ASTExpression::Unary { op, exp: Box::new(new_exp) }
        }
        ASTExpression::Binary { op, lhs, rhs } => {
            let new_lhs = resolve_expression(*lhs, variable_map);
            let new_rhs = resolve_expression(*rhs, variable_map);
            ASTExpression::Binary { op, lhs: Box::new(new_lhs), rhs: Box::new(new_rhs) }
        }
        ASTExpression::Assignment { lhs, rhs } => {
            if !matches!(lhs.as_ref(), ASTExpression::Var(_)) {
                panic!("Left-hand side of assignment must be a variable");
            }
            let new_lhs = resolve_expression(*lhs, variable_map);
            let new_rhs = resolve_expression(*rhs, variable_map);
            ASTExpression::Assignment { lhs: Box::new(new_lhs), rhs: Box::new(new_rhs) }
        }
    }
}