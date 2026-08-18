use std::collections::HashMap;
use crate::common::global_counter::get_unique_label;
use crate::parser::ast_tokens::{ASTBlock, ASTBlockItem, ASTDeclaration, ASTExpression, ASTFunctionDeclaration, ASTProgram, ASTStatement, ASTUnaryOperator, ASTForInit, ASTVariableDeclaration, ASTStorageClass};
use crate::parser::label_resolution::get_labels;

pub fn resolve_identifiers(program: ASTProgram) -> ASTProgram {
    let mut resolve_context = ResolveContext { identifier_map: HashMap::new(), labels: vec![] };
    let mut new_declarations = vec![];
    for declaration in program.declarations {
        new_declarations.push(resolve_declaration(declaration, &mut resolve_context, true));
    }

    ASTProgram { declarations: new_declarations }
}

fn resolve_block(
    block: ASTBlock,
    resolve_context: &mut ResolveContext,
) -> ASTBlock {
    let mut new_items = vec![];

    for block_item in block.items {
        new_items.push(resolve_block_item(block_item, resolve_context))
    }

    ASTBlock { items: new_items }
}

fn clear_current_scope_flag(variable_map: &mut VariableMap) {
    for entry in variable_map.values_mut() {
        entry.from_current_scope = false;
    }
}

fn resolve_block_item(
    block_item: ASTBlockItem,
    resolve_context: &mut ResolveContext,
) -> ASTBlockItem {
    match block_item {
        ASTBlockItem::Statement(statement) => {
            let new_statement = resolve_statement(statement, resolve_context);
            ASTBlockItem::Statement(new_statement)
        }
        ASTBlockItem::Declaration(declaration) => {
            let new_declaration = resolve_declaration(declaration, resolve_context, false);
            ASTBlockItem::Declaration(new_declaration)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VariableMapEntry {
    name: String,
    has_external_linkage: bool,
    from_current_scope: bool,
}

type VariableMap = HashMap<String, VariableMapEntry>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolveContext {
    identifier_map: VariableMap,
    labels: Vec<String>,
}

impl ResolveContext {
    fn clone_for_new_scope(&self) -> ResolveContext {
        let mut new_variable_map = self.identifier_map.clone();
        clear_current_scope_flag(&mut new_variable_map);
        ResolveContext { identifier_map: new_variable_map, labels: self.labels.clone() }
    }
}

fn resolve_statement(
    statement: ASTStatement,
    resolve_context: &mut ResolveContext,
) -> ASTStatement {
    match statement {
        ASTStatement::Return(exp) => {
            let new_exp = resolve_expression(exp, resolve_context);
            ASTStatement::Return(new_exp)
        }
        ASTStatement::Expression(exp) => {
            let new_exp = resolve_expression(exp, resolve_context);
            ASTStatement::Expression(new_exp)
        }
        ASTStatement::Null => {
            ASTStatement::Null
        }
        ASTStatement::If { condition, then_statement, else_statement } => {
            let new_then_statement = resolve_statement(*then_statement, resolve_context);
            let new_else_statement = else_statement.map(|else_statement| resolve_statement(*else_statement, resolve_context));
            let new_condition = resolve_expression(condition.clone(), resolve_context);
            ASTStatement::If { condition: new_condition, then_statement: Box::new(new_then_statement), else_statement: new_else_statement.map(Box::new) }
        }
        ASTStatement::While { condition, body, label } => {
            let body = body.as_ref();
            let new_body = resolve_statement(body.clone(), resolve_context);
            let new_condition = resolve_expression(condition.clone(), resolve_context);
            ASTStatement::While { condition: new_condition, body: Box::new(new_body), label }
        }
        ASTStatement::DoWhile { condition, body, label } => {
            let body = body.as_ref();
            let new_body = resolve_statement(body.clone(), resolve_context);
            let new_condition = resolve_expression(condition.clone(), resolve_context);
            ASTStatement::DoWhile { condition: new_condition, body: Box::new(new_body), label }
        }
        ASTStatement::For { init, condition, post, body, label } => {
            let mut for_init_resolve_context = resolve_context.clone_for_new_scope();
            let new_init = resolve_forinit(init, &mut for_init_resolve_context);

            let new_condition = condition.clone().map(|object|resolve_expression(object, &mut for_init_resolve_context));
            let new_post = post.map(|object|resolve_expression(object, &mut for_init_resolve_context));

            let mut for_body_resolve_context = for_init_resolve_context.clone_for_new_scope();
            let new_body = resolve_statement(body.as_ref().clone(), &mut for_body_resolve_context);
            ASTStatement::For { init: new_init, condition: new_condition, post: new_post, body: Box::new(new_body), label }
        }
        ASTStatement::Switch { condition, body, label } => {
            let new_condition = resolve_expression(condition.clone(), resolve_context);
            let new_body = resolve_statement(body.as_ref().clone(), resolve_context);
            ASTStatement::Switch { condition: new_condition, body: Box::new(new_body), label }
        }
        ASTStatement::Case { value, body, label } => {
            let new_value = resolve_expression(value.clone(), resolve_context);
            let new_body = resolve_statement(body.as_ref().clone(), resolve_context);
            ASTStatement::Case { value: new_value, body: Box::new(new_body), label }
        }
        ASTStatement::Default { body, label } => {
            let new_body = resolve_statement(body.as_ref().clone(), resolve_context);
            ASTStatement::Default { body: Box::new(new_body), label }
        }
        ASTStatement::Break { label } => {
            ASTStatement::Break { label }
        }
        ASTStatement::Continue { label } => {
            ASTStatement::Continue { label }
        }
        ASTStatement::Compound(block) => {
            let mut new_resolve_context = resolve_context.clone_for_new_scope();
            let new_block = resolve_block(block, &mut new_resolve_context);
            ASTStatement::Compound(new_block)
        }
        ASTStatement::Goto { label } => {
            if !resolve_context.labels.contains(&label) {
                panic!("Label {} is not defined", label);
            }
            ASTStatement::Goto { label }
        }
        ASTStatement::LabelledStatement { label, statement } => {
            let new_statement = resolve_statement(*statement, resolve_context);
            ASTStatement::LabelledStatement { label, statement: Box::new(new_statement) }
        }
    }
}

fn resolve_forinit(init: ASTForInit, resolve_context: &mut ResolveContext) -> ASTForInit {
    match init {
        ASTForInit::InitDecl(declaration) => {
            let new_declaration = resolve_block_scope_variable_declaration(declaration, resolve_context);
            ASTForInit::InitDecl(new_declaration)
        }
        ASTForInit::InitExp(expression) => {
            let new_expression = expression.map(|object|resolve_expression(object, resolve_context));
            ASTForInit::InitExp(new_expression)
        }
    }
}

fn resolve_declaration(
    declaration: ASTDeclaration,
    resolve_context: &mut ResolveContext,
    file_scope: bool,
) -> ASTDeclaration {
    match declaration {
        ASTDeclaration::VariableDeclaration(variable_declaration) => {
            let variable_declaration = if file_scope {
                resolve_file_scope_variable_declaration(variable_declaration, resolve_context)
            } else {
                resolve_block_scope_variable_declaration(variable_declaration, resolve_context)
            };
            ASTDeclaration::VariableDeclaration(variable_declaration)
        }
        ASTDeclaration::FunctionDeclaration(function_declaration) => {
            let ASTFunctionDeclaration { name, params: _, body, storage_class } = function_declaration.clone();
            if !file_scope && body != None {
                panic!("Nested function definitions are not allowed");
            }
            if !file_scope && storage_class == Some(ASTStorageClass::Static) {
                panic!("Static functions are not allowed in block scope");
            }

            resolve_context.identifier_map.insert(name.clone(), VariableMapEntry { name: name.clone(), has_external_linkage: true, from_current_scope: true });
            let function_declaration = resolve_function_declaration(function_declaration, resolve_context);
            ASTDeclaration::FunctionDeclaration(function_declaration)
        }
    }
}

fn resolve_block_scope_variable_declaration(variable_declaration: ASTVariableDeclaration, resolve_context: &mut ResolveContext) -> ASTVariableDeclaration {
    let ASTVariableDeclaration { name, init, storage_class } = variable_declaration;
    if resolve_context.identifier_map.contains_key(&name) {
        let prev_entry = resolve_context.identifier_map.get(&name).unwrap();
        if prev_entry.from_current_scope {
            if !(prev_entry.has_external_linkage &&  storage_class == Some(ASTStorageClass::Extern)) {
                panic!("Conflicting local declaration!");
            }
        }
    }

    if storage_class == Some(ASTStorageClass::Extern) {
        resolve_context.identifier_map.insert(name.clone(), VariableMapEntry { name: name.clone(), has_external_linkage: true, from_current_scope: true });
        ASTVariableDeclaration { name, init, storage_class }
    } else {
        let unique_name = get_unique_label(&name);
        resolve_context.identifier_map.insert(name.clone(), VariableMapEntry { name: unique_name.clone(), has_external_linkage: false, from_current_scope: true });
        let new_init = init.map(|init| resolve_expression(init, resolve_context));
        ASTVariableDeclaration { name: unique_name, init: new_init, storage_class }
    }
}

fn resolve_file_scope_variable_declaration(variable_declaration: ASTVariableDeclaration, resolve_context: &mut ResolveContext) -> ASTVariableDeclaration {
    let ASTVariableDeclaration { name, init, storage_class } = variable_declaration;
    resolve_context.identifier_map.insert(name.clone(), VariableMapEntry { name: name.clone(), has_external_linkage: true, from_current_scope: true });
    // We don't need to resolve the init expression here, because it has to be a constant at file scope
    ASTVariableDeclaration { name, init, storage_class }
}

fn resolve_function_declaration(
    function_declaration: ASTFunctionDeclaration,
    resolve_context: &mut ResolveContext,
) -> ASTFunctionDeclaration {
    let ASTFunctionDeclaration { name, params: _, body: _, storage_class: _ } = function_declaration.clone();
    if resolve_context.identifier_map.contains_key(&name) && !resolve_context.identifier_map.get(&name).unwrap().has_external_linkage {
        panic!("Duplicate declaration with different linkage for function {}", name);
    }
    resolve_context.identifier_map.insert(
        name.clone(),
        VariableMapEntry { name: name.clone(), has_external_linkage: true, from_current_scope: true }
    );

    let mut function_context = resolve_context.clone_for_new_scope();
    let labels = get_labels(&function_declaration);
    function_context.labels = labels.clone();

    let mut new_params = vec![];
    for param in function_declaration.params.clone() {
        let new_param = resolve_param(param, &mut function_context);
        new_params.push(new_param);
    }
    let new_body = function_declaration.body.map(|b|resolve_block(b, &mut function_context));
    ASTFunctionDeclaration {
        name: function_declaration.name,
        params: new_params,
        body: new_body,
        storage_class: function_declaration.storage_class,
    }
}

fn resolve_param(param: String, resolve_context: &mut ResolveContext) -> String {
    if resolve_context.identifier_map.contains_key(&param) && resolve_context.identifier_map.get(&param).unwrap().from_current_scope {
        panic!("Parameter {} is already defined", param);
    }
    let unique_name = get_unique_label(&param);
    resolve_context.identifier_map.insert(param, VariableMapEntry { name: unique_name.clone(), has_external_linkage: false, from_current_scope: true });
    unique_name
}

fn resolve_expression(
    exp: ASTExpression,
    resolve_context: &mut ResolveContext,
) -> ASTExpression {
    match exp {
        ASTExpression::Var(name) => {
            let unique_name = resolve_context.identifier_map.get(&name).unwrap().name.clone();
            ASTExpression::Var(unique_name.clone())
        }
        ASTExpression::Constant(int) => {
            ASTExpression::Constant(int)
        }
        ASTExpression::Unary { op, exp } => {
            if matches!(op, ASTUnaryOperator::Increment | ASTUnaryOperator::Decrement) && !matches!(exp.as_ref(), ASTExpression::Var(_)) {
                panic!("lvalue expected for increment/decrement operator");
            }
            let new_exp = resolve_expression(*exp, resolve_context);
            ASTExpression::Unary { op, exp: Box::new(new_exp) }
        }
        ASTExpression::Binary { op, lhs, rhs } => {
            let new_lhs = resolve_expression(*lhs, resolve_context);
            let new_rhs = resolve_expression(*rhs, resolve_context);
            ASTExpression::Binary { op, lhs: Box::new(new_lhs), rhs: Box::new(new_rhs) }
        }
        ASTExpression::Assignment { lhs, rhs } => {
            if !matches!(lhs.as_ref(), ASTExpression::Var(_)) {
                panic!("Left-hand side of assignment must be a variable");
            }
            let new_lhs = resolve_expression(*lhs, resolve_context);
            let new_rhs = resolve_expression(*rhs, resolve_context);
            ASTExpression::Assignment { lhs: Box::new(new_lhs), rhs: Box::new(new_rhs) }
        }
        ASTExpression::CompoundAssignment { lhs, rhs, operator } => {
            if !matches!(lhs.as_ref(), ASTExpression::Var(_)) {
                panic!("Left-hand side of assignment must be a variable");
            }
            let new_lhs = resolve_expression(*lhs, resolve_context);
            let new_rhs = resolve_expression(*rhs, resolve_context);
            ASTExpression::CompoundAssignment { lhs: Box::new(new_lhs), rhs: Box::new(new_rhs), operator }
        }
        ASTExpression::Postfix { op, exp } => {
            if !matches!(exp.as_ref(), ASTExpression::Var(_)) {
                panic!("Postfix operator must be applied to a variable");
            }
            let new_exp = resolve_expression(*exp, resolve_context);
            ASTExpression::Postfix { op, exp: Box::new(new_exp) }
        }
        ASTExpression::Conditional { condition, if_true, if_false } => {
            let new_condition = resolve_expression(*condition, resolve_context);
            let new_if_true = resolve_expression(*if_true, resolve_context);
            let new_if_false = resolve_expression(*if_false, resolve_context);
            ASTExpression::Conditional { condition: Box::new(new_condition), if_true: Box::new(new_if_true), if_false: Box::new(new_if_false) }
        }
        ASTExpression::FunctionCall { name, args } => {
            if resolve_context.identifier_map.contains_key(&name) {
                let new_fun_name = resolve_context.identifier_map.get(&name).unwrap().name.clone();
                let mut new_args = vec![];
                for arg in args.clone() {
                    let new_arg = resolve_expression(arg, resolve_context);
                    new_args.push(new_arg);
                }
                ASTExpression::FunctionCall { name: new_fun_name, args: new_args }
            } else {
                panic!("Function {} is not defined", name);
            }
        }
    }
}