use crate::common::global_counter::get_unique_label;
use crate::parser::ast_tokens::{ASTBlock, ASTBlockItem, ASTDeclaration, ASTFunctionDeclaration, ASTProgram, ASTStatement};

#[derive(Debug, Clone, PartialEq, Eq)]
enum CurrentContext {
    Loop,
    Switch,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LabelContext {
    loop_label: Option<String>,
    switch_label: Option<String>,
    current_context: CurrentContext,
}

pub fn label_program(program: ASTProgram) -> ASTProgram {
    let mut label_context = LabelContext {
        loop_label: None,
        switch_label: None,
        current_context: CurrentContext::None,
    };

    let new_declarations= program
        .declarations
        .into_iter()
        .map(|decl| label_declaration(decl, &mut label_context))
        .collect();
    ASTProgram { declarations: new_declarations }
}

fn label_declaration(declaration: ASTDeclaration, label_context: &mut LabelContext) -> ASTDeclaration {
    match declaration {
        ASTDeclaration::VariableDeclaration(variable_declaration) => {
            ASTDeclaration::VariableDeclaration(variable_declaration)
        }
        ASTDeclaration::FunctionDeclaration(function_declaration) => {
            ASTDeclaration::FunctionDeclaration(label_function(function_declaration, label_context))
        }
    }
}

fn label_function(function: ASTFunctionDeclaration, label_context: &mut LabelContext) -> ASTFunctionDeclaration {
    panic!("Temporarily disabled");
    // let new_body = function.body.map(|b|label_block(b, label_context));
    // ASTFunctionDeclaration {
    //     name: function.name,
    //     params: function.params,
    //     body: new_body,
    //     storage_class: function.storage_class,
    // }
}

fn label_block(block: ASTBlock, label_context: &mut LabelContext) -> ASTBlock {
    let mut new_items = vec![];
    for block_item in block.items {
        new_items.push(label_block_item(block_item, label_context));
    }
    ASTBlock { items: new_items }
}

fn label_block_item(block_item: ASTBlockItem, label_context: &mut LabelContext) -> ASTBlockItem {
    match block_item {
        ASTBlockItem::Statement(statement) => {
            let new_statement = label_statement(statement, label_context);
            ASTBlockItem::Statement(new_statement)
        }
        ASTBlockItem::Declaration(declaration) => {
            ASTBlockItem::Declaration(declaration)
        }
    }
}

fn label_statement(statement: ASTStatement, label_context: &mut LabelContext) -> ASTStatement {
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
            let new_then_statement = label_statement(*then_statement, label_context);
            let new_else_statement = else_statement.map(|else_statement| label_statement(*else_statement, label_context));
            ASTStatement::If { condition, then_statement: Box::new(new_then_statement), else_statement: new_else_statement.map(Box::new) }
        }
        ASTStatement::While { condition, body, label: _ } => {
            let new_label = get_unique_label("while");
            let mut new_context = LabelContext { loop_label: Some(new_label.clone()), current_context: CurrentContext::Loop, ..label_context.clone() };
            let new_body = label_statement(body.as_ref().clone(), &mut new_context);
            ASTStatement::While { condition, body: Box::new(new_body), label: new_label }
        }
        ASTStatement::DoWhile { condition, body, label: _ } => {
            let new_label = get_unique_label("do_while");
            let mut new_context = LabelContext { loop_label: Some(new_label.clone()), current_context: CurrentContext::Loop, ..label_context.clone() };
            let new_body = label_statement(body.as_ref().clone(), &mut new_context);
            ASTStatement::DoWhile { condition, body: Box::new(new_body), label: new_label }
        }
        ASTStatement::For { init, condition, post, body, label: _ } => {
            let new_label = get_unique_label("for");
            let mut new_context = LabelContext { loop_label: Some(new_label.clone()), current_context: CurrentContext::Loop, ..label_context.clone() };
            let new_body = label_statement(body.as_ref().clone(), &mut new_context);
            ASTStatement::For { init, condition, post, body: Box::new(new_body), label: new_label }
        }
        ASTStatement::Switch { condition, body, label: _ } => {
            let new_label = get_unique_label("switch");
            let mut new_context = LabelContext { switch_label: Some(new_label.clone()), current_context: CurrentContext::Switch, ..label_context.clone() };
            let new_body = label_statement(body.as_ref().clone(), &mut new_context);
            ASTStatement::Switch { condition, body: Box::new(new_body), label: new_label }
        }
        ASTStatement::Case { value, body, label: _ } => {
            if let Some(switch_label) = label_context.clone().switch_label {
                let new_label = get_unique_label(&format!("{}_case", switch_label));
                let new_body = label_statement(body.as_ref().clone(), label_context);
                ASTStatement::Case { value, body: Box::new(new_body), label: new_label }
            } else {
                panic!("Case statement outside of switch");
            }
        }
        ASTStatement::Default { body, label: _ } => {
            if let Some(switch_label) = label_context.clone().switch_label {
                let new_label = format!("{}_default", switch_label);
                let new_body = label_statement(body.as_ref().clone(), label_context);
                ASTStatement::Default { body: Box::new(new_body), label: new_label }
            } else {
                panic!("Default statement outside of switch");
            }
        }
        ASTStatement::Break { label: _ } => {
            match label_context {
                LabelContext { loop_label: Some(loop_label), current_context: CurrentContext::Loop, .. } => {
                    ASTStatement::Break { label: loop_label.clone() }
                }
                LabelContext { switch_label: Some(switch_label), current_context: CurrentContext::Switch, .. } => {
                    ASTStatement::Break { label: switch_label.clone() }
                }
                _ => panic!("Break statement outside of loop or switch"),
            }
        }
        ASTStatement::Continue { label: _ } => {
            match label_context.loop_label.clone() {
                Some(label) => ASTStatement::Continue { label },
                None => panic!("Continue statement outside of loop"),
            }
        }
        ASTStatement::Compound(block) => {
            let new_block = label_block(block, label_context);
            ASTStatement::Compound(new_block)
        }
        ASTStatement::Goto { label } => {
            ASTStatement::Goto { label }
        }
        ASTStatement::LabelledStatement { label: st_label, statement } => {
            let new_statement = label_statement(*statement, label_context);
            ASTStatement::LabelledStatement { label: st_label, statement: Box::new(new_statement) }
        }
    }
}
