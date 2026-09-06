use std::collections::HashMap;
use crate::parser::ast_tokens::{ASTBlock, ASTBlockItem, ASTConstant, ASTDeclaration, ASTExpression, ASTFunctionDeclaration, ASTProgram, ASTStatement};

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchMetadata {
    pub(crate) cases: Vec<(i64, String)>, // values and labels
    pub(crate) default: Option<String>, // label of default case
}

type SwitchMap = HashMap<String, SwitchMetadata>;

pub fn parse_switches(program: ASTProgram) -> HashMap<String, SwitchMetadata> {
    let mut switch_map = HashMap::new();
    for decl in program.declarations {
        parse_declaration(decl, None, &mut switch_map);
    }
    switch_map
}

fn parse_declaration(declaration: ASTDeclaration, switch_label: Option<String>, switch_map: &mut SwitchMap) {
    match declaration {
        ASTDeclaration::VariableDeclaration(_) => {
            // Do nothing
        }
        ASTDeclaration::FunctionDeclaration(function) => {
            parse_function(function, switch_label, switch_map);
        }
    }
}

fn parse_function(function: ASTFunctionDeclaration, switch_label: Option<String>, switch_map: &mut SwitchMap) {
    let Some(function_body) = function.body else {
        return
    };
    parse_block(function_body, switch_label, switch_map);
}

fn parse_block(block: ASTBlock, switch_label: Option<String>, switch_map: &mut SwitchMap) {
    for block_item in block.items {
        parse_block_item(block_item, switch_label.clone(), switch_map);
    }
}

fn parse_block_item(block_item: ASTBlockItem, switch_label: Option<String>, switch_map: &mut SwitchMap) {
    match block_item {
        ASTBlockItem::Statement(statement) => {
            parse_statement(statement, switch_label, switch_map);
        }
        ASTBlockItem::Declaration(_) => {
            // Do nothing
        }
    }
}

fn parse_statement(statement: ASTStatement, switch_label: Option<String>, switch_map: &mut SwitchMap) {
    match statement {
        ASTStatement::Return(_) => {}
        ASTStatement::Expression(_) => {}
        ASTStatement::Null => {}
        ASTStatement::If { condition: _, then_statement, else_statement } => {
            parse_statement(*then_statement, switch_label.clone(), switch_map);
            if let Some(else_statement) = else_statement {
                parse_statement(*else_statement, switch_label, switch_map);
            }
        }
        ASTStatement::While { condition: _, body, label: _ } => {
            parse_statement(body.as_ref().clone(), switch_label, switch_map);
        }
        ASTStatement::DoWhile { condition: _, body, label: _ } => {
            parse_statement(body.as_ref().clone(), switch_label, switch_map);
        }
        ASTStatement::For { init: _, condition: _, post: _, body, label: _ } => {
            parse_statement(body.as_ref().clone(), switch_label, switch_map);
        }
        ASTStatement::Switch { condition: _, body, label } => {
            let switch_metadata = SwitchMetadata { cases: vec![], default: None };
            switch_map.insert(label.clone(), switch_metadata);
            let new_switch_label = label;
            parse_statement(body.as_ref().clone(), Some(new_switch_label), switch_map);
        }
        ASTStatement::Case { value, body, label } => {
            let Some(switch_label) = switch_label else {
                panic!("Case statement outside of switch");
            };
            let ASTExpression::Constant(ast_const) = value else {
                panic!("Case value is not a constant");
            };
            let case_val = match ast_const {
                ASTConstant::Int(val) | ASTConstant::Long(val) => val
            };
            let switch_metadata = switch_map.get_mut(&switch_label).unwrap();
            if switch_metadata.cases.iter().map(|case| case.0).any(|x| x == case_val) {
                panic!("Duplicate case value in switch");
            }
            switch_metadata.cases.push((case_val, label.clone()));
            parse_statement(body.as_ref().clone(), Some(switch_label), switch_map);
        }
        ASTStatement::Default { body, label } => {
            let Some(switch_label) = switch_label else {
                panic!("Default statement outside of switch");
            };
            if switch_map[&switch_label].default.is_some() {
                panic!("Multiple default statements in switch");
            }
            switch_map.get_mut(&switch_label).unwrap().default = Some(label.clone());
            parse_statement(body.as_ref().clone(), Some(switch_label), switch_map);
        }
        ASTStatement::Break { label: _ } => {}
        ASTStatement::Continue { label: _ } => {}
        ASTStatement::Compound(block) => {
            parse_block(block, switch_label, switch_map);
        }
        ASTStatement::Goto { label: _ } => {}
        ASTStatement::LabelledStatement { label: _, statement } => {
            parse_statement(*statement, switch_label.clone(), switch_map);
        }
    }
}