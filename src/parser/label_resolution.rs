use crate::parser::ast_tokens::{ASTBlockItem, ASTFunctionDefinition, ASTStatement};

pub fn get_labels(function_definition: &ASTFunctionDefinition) -> Vec<String> {
    let mut labels = Vec::new();

    for block_item in &function_definition.body.items {
        labels.extend(extract_labels_from_block_item(block_item));
    }

    check_duplicates(&labels);

    labels
}

fn check_duplicates(labels: &[String]) {
    let mut label_set = std::collections::HashSet::new();
    for label in labels {
        if !label_set.insert(label.clone()) {
            panic!("Label {} is defined more than once", label);
        }
    }
}

fn extract_labels_from_block_item(block_item: &ASTBlockItem) -> Vec<String> {
    match block_item {
        ASTBlockItem::Statement(statement) =>
            extract_labels_from_statement(&statement),
        ASTBlockItem::Declaration(_) =>
            vec![],
    }
}

fn extract_labels_from_statement(statement: &ASTStatement) -> Vec<String> {
    match statement {
        ASTStatement::Return(_) => {
            vec![]
        }
        ASTStatement::Expression(_) => {
            vec![]
        }
        ASTStatement::If { condition: _, then_statement, else_statement } => {
            let mut labels = Vec::new();
            labels.extend(extract_labels_from_statement(then_statement.as_ref()));
            if let Some(else_statement) = else_statement {
                labels.extend(extract_labels_from_statement(else_statement.as_ref()));
            }
            labels
        }
        ASTStatement::While { condition: _, body, label: _ } => {
            let mut labels = Vec::new();
            labels.extend(extract_labels_from_statement(body.as_ref()));
            labels
        }
        ASTStatement::DoWhile { condition: _, body, label: _ } => {
            let mut labels = Vec::new();
            labels.extend(extract_labels_from_statement(body.as_ref()));
            labels
        }
        ASTStatement::For { init: _, condition: _, post: _, body, label: _ } => {
            let mut labels = Vec::new();
            labels.extend(extract_labels_from_statement(body.as_ref()));
            labels
        }
        ASTStatement::Break { label: _ } => {
            vec![]
        }
        ASTStatement::Continue { label: _ } => {
            vec![]
        }
        ASTStatement::Compound(block) => {
            let mut labels = Vec::new();
            for block_item in &block.items {
                labels.extend(extract_labels_from_block_item(block_item));
            }
            labels
        }
        ASTStatement::Goto { label: _ } => {
            vec![]
        }
        ASTStatement::LabelledStatement { label, statement } => {
            let mut labels = vec![label.clone()];
            labels.extend(extract_labels_from_statement(statement.as_ref()));
            labels
        }
        ASTStatement::Null => {
            vec![]
        }
    }
}
