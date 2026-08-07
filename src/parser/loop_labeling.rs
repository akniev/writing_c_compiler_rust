use crate::common::global_counter::get_unique_label;
use crate::parser::ast_tokens::{ASTBlock, ASTBlockItem, ASTFunctionDefinition, ASTProgram, ASTStatement};

pub fn label_loops(program: ASTProgram) -> ASTProgram {
    let new_function_definition = label_loops_for_function(program.function_definition, None);
    ASTProgram { function_definition: new_function_definition }
}

fn label_loops_for_function(function: ASTFunctionDefinition, label: Option<String>) -> ASTFunctionDefinition {
    let new_body = label_loops_for_block(function.body, label);
    ASTFunctionDefinition { name: function.name, body: new_body }
}

fn label_loops_for_block(block: ASTBlock, label: Option<String>) -> ASTBlock {
    let mut new_items = vec![];
    for block_item in block.items {
        new_items.push(label_loops_for_block_item(block_item, label.clone()));
    }
    ASTBlock { items: new_items }
}

fn label_loops_for_block_item(block_item: ASTBlockItem, label: Option<String>) -> ASTBlockItem {
    match block_item {
        ASTBlockItem::Statement(statement) => {
            let new_statement = label_loops_for_statement(statement, label);
            ASTBlockItem::Statement(new_statement)
        }
        ASTBlockItem::Declaration(declaration) => {
            ASTBlockItem::Declaration(declaration)
        }
    }
}

fn label_loops_for_statement(statement: ASTStatement, label: Option<String>) -> ASTStatement {
    match statement {
        ASTStatement::Return(exp) => {
            ASTStatement::Return(exp)
        }
        ASTStatement::Expression(exp) => {
            ASTStatement::Expression(exp)
        }
        ASTStatement::Null => {
            ASTStatement::Null
        }
        ASTStatement::If { condition, then_statement, else_statement } => {
            let new_then_statement = label_loops_for_statement(*then_statement, label.clone());
            let new_else_statement = else_statement.map(|else_statement| label_loops_for_statement(*else_statement, label.clone()));
            ASTStatement::If { condition, then_statement: Box::new(new_then_statement), else_statement: new_else_statement.map(Box::new) }
        }
        ASTStatement::While { condition, body, label: _ } => {
            let new_label = get_unique_label("while");
            let new_body = label_loops_for_statement(body.as_ref().clone(), Some(new_label.clone()));
            ASTStatement::While { condition, body: Box::new(new_body), label: new_label }
        }
        ASTStatement::DoWhile { condition, body, label: _ } => {
            let new_label = get_unique_label("do-while");
            let new_body = label_loops_for_statement(body.as_ref().clone(), Some(new_label.clone()));
            ASTStatement::DoWhile { condition, body: Box::new(new_body), label: new_label }
        }
        ASTStatement::For { init, condition, post, body, label: _ } => {
            let new_label = get_unique_label("for");
            let new_body = label_loops_for_statement(body.as_ref().clone(), Some(new_label.clone()));
            ASTStatement::For { init, condition, post, body: Box::new(new_body), label: new_label }
        }
        ASTStatement::Break { label: _ } => {
            match label {
                Some(label) => ASTStatement::Break { label },
                None => panic!("Break statement outside of loop"),
            }
        }
        ASTStatement::Continue { label: _ } => {
            match label {
                Some(label) => ASTStatement::Continue { label },
                None => panic!("Continue statement outside of loop"),
            }
        }
        ASTStatement::Compound(block) => {
            let new_block = label_loops_for_block(block, label);
            ASTStatement::Compound(new_block)
        }
        ASTStatement::Goto { label } => {
            ASTStatement::Goto { label }
        }
        ASTStatement::LabelledStatement { label: st_label, statement } => {
            let new_statement = label_loops_for_statement(*statement, label);
            ASTStatement::LabelledStatement { label: st_label, statement: Box::new(new_statement) }
        }
    }
}
