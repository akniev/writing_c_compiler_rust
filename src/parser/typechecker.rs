use std::collections::HashMap;
use std::iter::zip;
use crate::parser::ast_tokens::{ASTBinaryOperator, ASTBlock, ASTBlockItem, ASTConstant, ASTDeclaration, ASTExpression, ASTForInit, ASTFunctionDeclaration, ASTProgram, ASTStatement, ASTStorageClass, ASTType, ASTTypedExpression, ASTUnaryOperator, ASTVariableDeclaration};

/*
Type checking pass validates that all declarations and uses of an identifier have compatible types.
For example, if you declare that x is a variable, you can't call is like a function.

To type check the program, we'll record the type of every identifier in the symbol table.
We'll also record whether each function we encountered is defined or just declared; that is,
whether it has a body.

The symbol table will be our central source of information about every identifier in the program.

In Chapter 11, we change the typechecker to return the updated AST along with the symbol table.
 */

pub fn typecheck(program: ASTProgram) -> (ASTProgram, HashMap<String, Symbol>) {
    let mut symbols = HashMap::new();
    let mut new_declarations = vec![];
    for decl in program.declarations {
        let new_decl = typechecked_declaration(&decl, &mut symbols, false);
        new_declarations.push(new_decl);
    }
    let new_program = ASTProgram { declarations: new_declarations };
    (new_program, symbols)
}

fn typechecked_declaration(
    declaration: &ASTDeclaration,
    symbols: &mut HashMap<String, Symbol>,
    block_scope: bool,
) -> ASTDeclaration {
    match declaration {
        ASTDeclaration::FunctionDeclaration(function_declaration) => {
            let fun_decl = typechecked_function_declaration(function_declaration.clone(), symbols);
            ASTDeclaration::FunctionDeclaration(fun_decl)
        }
        ASTDeclaration::VariableDeclaration(variable_declaration) => {
            if block_scope {
                let var_decl = typechecked_local_variable_declaration(variable_declaration, symbols);
                ASTDeclaration::VariableDeclaration(var_decl)
            } else {
                let var_decl = typechecked_file_scope_variable_declaration(variable_declaration, symbols);
                ASTDeclaration::VariableDeclaration(var_decl)
            }
        }
    }
}

fn typechecked_function_declaration(
    function_declaration: ASTFunctionDeclaration,
    symbols: &mut HashMap<String, Symbol>
) -> ASTFunctionDeclaration {
    let fun_type = function_declaration._type.clone();
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
        name: function_declaration.name.clone(),
        _type: fun_type,
        identifier_attrs: attrs,
    });

    let mut new_body = None;

    if has_body {
        for param in function_declaration.params.clone() {
            symbols.insert(param.clone(), Symbol { name: param, _type: ASTType::Int, identifier_attrs: SymbolAttrs::LocalAttr });
        }
        let function_info = FunctionInfo {
            name: function_declaration.name.clone(),
            return_type: function_declaration._type.clone(),
        };
        new_body = Some(typechecked_block(function_declaration.body.unwrap(), symbols, Some(function_info)));
    }

    ASTFunctionDeclaration {
        name: function_declaration.name,
        params: function_declaration.params,
        body: new_body,
        _type: function_declaration._type,
        storage_class: function_declaration.storage_class,
    }
}

fn typechecked_local_variable_declaration(
    variable_declaration: &ASTVariableDeclaration,
    symbols: &mut HashMap<String, Symbol>
) -> ASTVariableDeclaration {
    let mut t_init = None;
    if variable_declaration.storage_class == Some(ASTStorageClass::Extern) {
        if variable_declaration.init.is_some() {
            panic!("Initializer on local extern variable declaration");
        }
        if symbols.contains_key(&variable_declaration.name.clone()) {
            let old_decl = symbols.get(&variable_declaration.name).unwrap();
            if old_decl._type != variable_declaration._type {
                panic!("Invalid redeclaration");
            }
        } else {
            symbols.insert(
                variable_declaration.name.clone(),
                Symbol {
                    name: variable_declaration.name.clone(),
                    _type: ASTType::Int,
                    identifier_attrs: SymbolAttrs::StaticAttr {
                        init: SymbolInitialValue::NoInitializer,
                        global: true
                    }
                }
            );
        }
    } else if variable_declaration.storage_class == Some(ASTStorageClass::Static) {
        let initial_value = if let Some(unwrapped) = variable_declaration.clone().init
            && let ASTExpression::Constant(init_val) = unwrapped.exp() {
            match init_val {
                ASTConstant::Int(i) => SymbolInitialValue::Initial(StaticInit::IntInit(i.clone() as i32)),
                ASTConstant::Long(i) => SymbolInitialValue::Initial(StaticInit::LongInit(i.clone())),
            }
        } else if variable_declaration.init.is_none() {
            match variable_declaration.clone()._type {
                ASTType::Int => SymbolInitialValue::Initial(StaticInit::IntInit(0)),
                ASTType::Long => SymbolInitialValue::Initial(StaticInit::LongInit(0)),
                _ => panic!("Non-constant initializer on local static variable")
            }
        } else {
            panic!("Non-constant initializer on local static variable")
        };
        symbols.insert(
            variable_declaration.name.clone(),
            Symbol {
                name: variable_declaration.name.clone(),
                _type: ASTType::Int,
                identifier_attrs: SymbolAttrs::StaticAttr { init: initial_value, global: false }
            }
        );
    } else {
        symbols
            .insert(
                variable_declaration.clone().name,
                Symbol {
                    name: variable_declaration.name.clone(),
                    _type: ASTType::Int,
                    identifier_attrs: SymbolAttrs::LocalAttr
                }
            );
        if let Some(init) = variable_declaration.clone().init {
            t_init = Some(typechecked_expression(init, symbols));
        }
    }

    ASTVariableDeclaration {
        name: variable_declaration.name.clone(),
        _type: variable_declaration._type.clone(),
        storage_class: variable_declaration.storage_class.clone(),
        init: t_init,
    }
}

fn typechecked_file_scope_variable_declaration(
    variable_declaration: &ASTVariableDeclaration,
    symbols: &mut HashMap<String, Symbol>
) -> ASTVariableDeclaration {
    let mut initial_value = if let Some(ASTExpression::Constant(init_value)) = variable_declaration.clone().init.map(|e| e.exp().clone()) {
        match init_value {
            ASTConstant::Int(i) => SymbolInitialValue::Initial(StaticInit::IntInit(i.clone() as i32)),
            ASTConstant::Long(i) => SymbolInitialValue::Initial(StaticInit::LongInit(i.clone())),
        }
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
        if old_decl._type != variable_declaration._type {
            panic!("Invalid redeclaration of variable");
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
            _type: variable_declaration._type.clone(),
            identifier_attrs: attrs
        }
    );

    variable_declaration.clone()
}

fn typechecked_block(block: ASTBlock, symbols: &mut HashMap<String, Symbol>, enclosing_function: Option<FunctionInfo>) -> ASTBlock {
    let mut new_block_items = vec![];
    for block_item in block.items {
        match block_item {
            ASTBlockItem::Declaration(declaration) => {
                let decl = typechecked_declaration(&declaration, symbols, true);
                new_block_items.push(ASTBlockItem::Declaration(decl));
            }
            ASTBlockItem::Statement(statement) => {
                let stat = typechecked_statement(statement, symbols, enclosing_function.clone());
                new_block_items.push(ASTBlockItem::Statement(stat));
            }
        }
    }
    ASTBlock { items: new_block_items }
}

fn typechecked_statement(
    statement: ASTStatement,
    symbols: &mut HashMap<String, Symbol>,
    enclosing_function: Option<FunctionInfo>,
) -> ASTStatement {
    match statement {
        ASTStatement::Return(exp) => {
            let Some(function_info) = enclosing_function else {
                panic!("Return statement outside of function");
            };
            let typechecked_exp = typechecked_expression(exp, symbols).with_type(Some(function_info.return_type.clone()));
            ASTStatement::Return(typechecked_exp)
        }
        ASTStatement::Expression(exp) => {
            let typechecked_exp = typechecked_expression(exp, symbols);
            ASTStatement::Expression(typechecked_exp)
        }
        ASTStatement::If { condition, then_statement, else_statement } => {
            let t_cond = typechecked_expression(condition, symbols);
            let t_then = typechecked_statement(then_statement.as_ref().clone(), symbols, enclosing_function.clone());
            let t_else: Option<Box<ASTStatement>> = if let Some(else_statement) = else_statement {
                Some(Box::new(typechecked_statement(else_statement.as_ref().clone(), symbols, enclosing_function.clone())))
            } else {
                None
            };
            ASTStatement::If { condition: t_cond, then_statement: Box::new(t_then), else_statement: t_else }
        }
        ASTStatement::Compound(block) => {
            let t_block = typechecked_block(block, symbols, enclosing_function.clone());
            ASTStatement::Compound(t_block)
        }
        ASTStatement::While { condition, body, label } => {
            let t_cond = typechecked_expression(condition, symbols);
            let t_body = typechecked_statement(body.as_ref().clone(), symbols, enclosing_function.clone());
            ASTStatement::While { condition: t_cond, body: Box::new(t_body), label: label.clone() }
        }
        ASTStatement::DoWhile { body, condition, label } => {
            let t_body = typechecked_statement(body.as_ref().clone(), symbols, enclosing_function.clone());
            let t_cond = typechecked_expression(condition, symbols);
            ASTStatement::DoWhile { body: Box::new(t_body), condition: t_cond, label: label.clone() }
        }
        ASTStatement::For { init, condition, post, body, label } => {
            let new_init = match init {
                ASTForInit::InitExp(exp) => {
                    let t_exp = if let Some(exp) = exp {
                        Some(typechecked_expression(exp, symbols))
                    } else {
                        None
                    };
                    ASTForInit::InitExp(t_exp)
                }
                ASTForInit::InitDecl(variable_declaration) => {
                    if variable_declaration.storage_class.is_some() {
                        panic!("For loop variable declaration with storage class");
                    }
                    let t_decl = typechecked_local_variable_declaration(&variable_declaration, symbols);
                    ASTForInit::InitDecl(t_decl)
                }
            };
            let t_cond = if let Some(condition) = condition {
                Some(typechecked_expression(condition, symbols))
            } else {
                None
            };

            let t_post = if let Some(post) = post {
                Some(typechecked_expression(post, symbols))
            } else {
                None
            };

            let t_statement = typechecked_statement(body.as_ref().clone(), symbols, enclosing_function.clone());

            ASTStatement::For {
                init: new_init,
                condition: t_cond,
                post: t_post,
                body: Box::new(t_statement),
                label: label.clone(),
            }
        }
        ASTStatement::Switch { condition, body, label } => {
            let t_cond = typechecked_expression(condition, symbols);
            let t_body = typechecked_statement(body.as_ref().clone(), symbols, enclosing_function.clone());
            ASTStatement::Switch {
                condition: t_cond,
                body: Box::new(t_body),
                label: label.clone()
            }
        }
        ASTStatement::Case { value, body, label } => {
            let t_val = typechecked_expression(value, symbols);
            let t_body = typechecked_statement(body.as_ref().clone(), symbols, enclosing_function.clone());
            ASTStatement::Case {
                value: t_val,
                body: Box::new(t_body),
                label: label.clone(),
            }
        }
        ASTStatement::Default { body, label } => {
            let t_body = typechecked_statement(body.as_ref().clone(), symbols, enclosing_function.clone());
            ASTStatement::Default {
                body: Box::new(t_body),
                label: label.clone()
            }
        }
        ASTStatement::LabelledStatement { label, statement } => {
            let t_stat = typechecked_statement(statement.as_ref().clone(), symbols, enclosing_function.clone());
            ASTStatement::LabelledStatement { label: label.clone(), statement: Box::new(t_stat) }
        }
        ASTStatement::Break { label: _ } | ASTStatement::Continue { label: _ } | ASTStatement::Goto { label: _ } | ASTStatement::Null => {
            statement
        }
    }
}

fn typechecked_expression(
    expression: ASTTypedExpression,
    symbols: &mut HashMap<String, Symbol>
) -> ASTTypedExpression {
    match expression.exp() {
        ASTExpression::Var(name) => {
            let Some(symbol) = symbols.get(name) else {
                panic!("{} is not defined", name);
            };
            if symbol._type.is_function_type() {
                panic!("{} is not a constant", name);
            }
            expression.with_type(Some(symbol.clone()._type))
        }
        ASTExpression::Constant(c) => {
            let c_type = match c {
                ASTConstant::Int(_) => ASTType::Int,
                ASTConstant::Long(_) => ASTType::Long,
            };
            expression.with_type(Some(c_type))
        }
        ASTExpression::Unary { op, exp } => {
            let t_inner_exp = typechecked_expression(exp.as_ref().clone(), symbols);
            let n_exp = ASTExpression::Unary { op: op.clone(), exp: Box::new(t_inner_exp.clone()) };
            let exp_type = match op {
                ASTUnaryOperator::Not => Some(ASTType::Int),
                _ => t_inner_exp.exp_type,
            };
            ASTTypedExpression {
                exp: n_exp,
                exp_type,
            }
        }
        ASTExpression::Postfix { op, exp } => {
            let t_inner_exp = typechecked_expression(exp.as_ref().clone(), symbols);
            let n_exp = ASTExpression::Postfix { op: op.clone(), exp: Box::new(t_inner_exp.clone()) };
            ASTTypedExpression {
                exp: n_exp,
                exp_type: t_inner_exp.exp_type,
            }
        }
        ASTExpression::Binary { op, e1, e2 } => {
            let t_e1 = typechecked_expression(e1.as_ref().clone(), symbols);
            let t_e2 = typechecked_expression(e2.as_ref().clone(), symbols);
            let common_type = match op {
                ASTBinaryOperator::And
                | ASTBinaryOperator::Or
                | ASTBinaryOperator::Equal
                | ASTBinaryOperator::NotEqual
                | ASTBinaryOperator::LessThan
                | ASTBinaryOperator::GreaterThan
                | ASTBinaryOperator::LessThanOrEqual
                | ASTBinaryOperator::GreaterThanOrEqual => {
                    Some(ASTType::Int)
                },
                _ => get_common_type(&t_e1.exp_type, &t_e2.exp_type)
            };
            let inner_exp = ASTExpression::Binary {
                op: op.clone(),
                e1: Box::new(t_e1.with_type(common_type.clone())),
                e2: Box::new(t_e2.with_type(common_type.clone())),
            };
            ASTTypedExpression {
                exp: inner_exp,
                exp_type: common_type,
            }
        }
        ASTExpression::Assignment { lhs, rhs } => {
            let t_lhs = typechecked_expression(lhs.as_ref().clone(), symbols);
            let t_rhs = typechecked_expression(rhs.as_ref().clone(), symbols);
            let left_type = t_lhs.clone().exp_type;
            let converted_rhs = t_rhs.with_type(left_type.clone());
            let assignment_exp = ASTExpression::Assignment { lhs: Box::new(t_lhs.clone()), rhs: Box::new(converted_rhs) };
            ASTTypedExpression {
                exp: assignment_exp,
                exp_type: left_type,
            }
        }
        ASTExpression::CompoundAssignment { lhs, rhs, operator } => {
            let t_lhs = typechecked_expression(lhs.as_ref().clone(), symbols);
            let t_rhs = typechecked_expression(rhs.as_ref().clone(), symbols);
            let left_type = t_lhs.clone().exp_type;
            let converted_rhs = t_rhs.with_type(left_type.clone());
            let exp = ASTExpression::CompoundAssignment { lhs: Box::new(t_lhs), rhs: Box::new(converted_rhs), operator: operator.clone() };
            ASTTypedExpression {
                exp,
                exp_type: left_type,
            }
        }
        ASTExpression::Conditional { condition, if_true, if_false } => {
            let t_cond = typechecked_expression(condition.as_ref().clone(), symbols);
            let t_if_true = typechecked_expression(if_true.as_ref().clone(), symbols);
            let t_if_false = typechecked_expression(if_false.as_ref().clone(), symbols);
            let common_type = get_common_type(&t_if_true.exp_type, &t_if_false.exp_type);
            let inner_exp = ASTExpression::Conditional {
                condition: Box::new(t_cond),
                if_true: Box::new(t_if_true.with_type(common_type.clone())),
                if_false: Box::new(t_if_false.with_type(common_type.clone())),
            };
            ASTTypedExpression {
                exp: inner_exp,
                exp_type: common_type,
            }
        }
        ASTExpression::FunctionCall { name, args } => {
            let f_type = symbols.get(name).unwrap().clone()._type;
            match f_type {
                ASTType::FunType(param_types, return_type) => {
                    if param_types.len() != args.len() {
                        panic!("{} expects {} arguments, got {}", name, param_types.len(), args.len());
                    }
                    let mut converted_args = vec![];
                    for (arg, param_type) in zip(args, param_types) {
                        let t_arg = typechecked_expression(arg.clone(), symbols);
                        converted_args.push(t_arg.with_type(Some(param_type.as_ref().clone())));
                    }
                    let call_exp = ASTExpression::FunctionCall {
                        name: name.clone(),
                        args: converted_args,
                    };
                    ASTTypedExpression {
                        exp: call_exp,
                        exp_type: Some(return_type.as_ref().clone()),
                    }
                }
                _ => panic!("{} is not a function", name),
            }
        }
        ASTExpression::Cast { target_type, exp } => {
            let t_exp = typechecked_expression(exp.as_ref().clone(), symbols);
            let inner_exp = ASTExpression::Cast {
                target_type: target_type.clone(),
                exp: Box::new(t_exp.clone()),
            };
            inner_exp.to_typed(Some(target_type.clone()))
        }
    }
}



// MARK: - Helpers

fn get_common_type(t1: &Option<ASTType>, t2: &Option<ASTType>) -> Option<ASTType> {
    let Some(t1) = t1 else {
        panic!("Cannot get common type of None");
    };
    let Some(t2) = t2 else {
        panic!("Cannot get common type of None");
    };
    if t1 == t2 {
        Some(t1.clone())
    } else {
        Some(ASTType::Long)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub _type: ASTType,
    pub identifier_attrs: SymbolAttrs,
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
    Initial(StaticInit),
    NoInitializer,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StaticInit {
    IntInit(i32),
    LongInit(i64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionInfo {
    pub name: String,
    pub return_type: ASTType,
}