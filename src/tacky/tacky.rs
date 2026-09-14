use std::collections::HashMap;
use crate::parser::ast_tokens::{ASTBinaryOperator, ASTBlock, ASTBlockItem, ASTConstant, ASTDeclaration, ASTExpression, ASTForInit, ASTFunctionDeclaration, ASTPostfixOperator, ASTProgram, ASTStatement, ASTType, ASTTypedExpression, ASTUnaryOperator, ASTVariableDeclaration};
use crate::tacky::tacky_tokens::{TBinaryOperator, TFunction, TInstruction, TProgram, TStaticVariable, TTopLevel, TUnaryOperator, TValue};

use crate::common::global_counter::{get_global_counter};
use crate::parser::parse_switches::SwitchMetadata;
use crate::parser::typechecker::{StaticInit, Symbol, SymbolAttrs, SymbolInitialValue};
use crate::parser::typechecker::SymbolAttrs::StaticAttr;

pub fn parse_tacky(
    program: ASTProgram,
    ast_symbols: &HashMap<String, Symbol>,
    switch_map: HashMap<String, SwitchMetadata>,
) -> TProgram {
    let mut symbols: HashMap<String, Symbol> = HashMap::new();
    let mut top_level = vec![];

    top_level.extend(convert_symbols_to_tacky(ast_symbols));

    for decl in program.declarations {
        match decl {
            ASTDeclaration::FunctionDeclaration(func_decl) => {
                if func_decl.body.is_some() {
                    let tfunc = parse_function(func_decl, &ast_symbols, switch_map.clone(), &mut symbols);
                    top_level.push(TTopLevel::Function(tfunc));
                }
            }
            ASTDeclaration::VariableDeclaration(_) => {
                // Do nothing
            }
        }
    }

    let tprog = TProgram { top_level };
    tprog
}

fn convert_symbols_to_tacky(ast_symbols: &HashMap<String, Symbol>) -> Vec<TTopLevel> {
    let mut static_vars = vec![];

    for (symbol_name, symbol) in ast_symbols {
        match symbol.clone().identifier_attrs {
            SymbolAttrs::StaticAttr { init, global } => {
                match init {
                    SymbolInitialValue::Initial(static_init) => static_vars.push(TStaticVariable { name: symbol_name.clone(), global, init: static_init.clone() }),
                    SymbolInitialValue::Tentative => {
                        let init_val = match symbol._type.clone() {
                            ASTType::Int => StaticInit::IntInit(0),
                            ASTType::Long => StaticInit::LongInit(0),
                            ASTType::FunType(_, _) => {
                                panic!("Function type is not supported!");
                            },
                        };
                        static_vars.push(TStaticVariable { name: symbol_name.clone(), global, init: init_val.clone() } );
                    },
                    SymbolInitialValue::NoInitializer => {
                        // Do nothing
                    }
                }
            }
            _ => {
                // Do nothing
            }
        }
    }

    static_vars.iter().map(|static_var| TTopLevel::StaticVariable(static_var.clone())).collect()
}

fn parse_function(
    function: ASTFunctionDeclaration,
    ast_symbols: &HashMap<String, Symbol>,
    switch_map: HashMap<String, SwitchMetadata>,
    symbols: &mut HashMap<String, Symbol>,
) -> TFunction {
    let Some(fun_body) = function.body else {
        panic!("Function body is missing!");
    };
    let Some(fun_decl) = ast_symbols.get(&function.name) else {
        panic!("Function not found!");
    };
    let mut function_instructions = parse_block(&fun_body, switch_map, symbols);
    let init_val = match fun_decl._type.fun_return_type() {
        ASTType::Int => ASTConstant::Int(0),
        ASTType::Long => ASTConstant::Long(0),
        ASTType::FunType(_, _) => {
            panic!("Function type is not supported!");
        },
    };
    function_instructions.push(TInstruction::Return(TValue::Constant(init_val)));
    let t_function = TFunction {
        name: function.name,
        global: fun_decl.identifier_attrs.is_global(),
        params: function.params,
        body: function_instructions
    };
    t_function
}

fn parse_file_variable_declaration(
    var_decl: &ASTVariableDeclaration,
    symbols: &HashMap<String, Symbol>,
) -> TTopLevel {
    let Some(symbol) = symbols.get(&var_decl.name) else {
        panic!("Symbol not found!");
    };

    let init = match symbol.identifier_attrs.get_init() {
        Some(SymbolInitialValue::Tentative | SymbolInitialValue::NoInitializer) | None => {
            panic!("Variable is not initialized!");
        }
        Some(SymbolInitialValue::Initial(init_val)) => {
            init_val.clone()
        }
    };
    let static_var = TStaticVariable {
        name: var_decl.name.clone(),
        global: symbol.identifier_attrs.is_global(),
        init,
    };
    TTopLevel::StaticVariable(static_var)
}

fn parse_block(
    block: &ASTBlock,
    switch_map: HashMap<String, SwitchMetadata>,
    symbols: &mut HashMap<String, Symbol>
) -> Vec<TInstruction> {
    let function_body: Vec<TInstruction> = block
        .items
        .iter()
        .flat_map(|block_item| parse_block_item(block_item, switch_map.clone(), symbols))
        .collect();
    function_body
}

fn parse_block_item(
    block_item: &ASTBlockItem,
    switch_map: HashMap<String, SwitchMetadata>,
    symbols: &mut HashMap<String, Symbol>,
) -> Vec<TInstruction> {
    match block_item {
        ASTBlockItem::Statement(statement) => {
            parse_statement(&statement, switch_map, symbols)
        }
        ASTBlockItem::Declaration(declaration) => {
            parse_declaration(&declaration, symbols)
        }
    }
}

fn parse_statement(
    statement: &ASTStatement,
    switch_map: HashMap<String, SwitchMetadata>,
    symbols: &mut HashMap<String, Symbol>
) -> Vec<TInstruction> {
    let mut instructions = Vec::new();

    match statement {
        ASTStatement::Return(expr) => {
            let val = emit_tacky(expr, &mut instructions, symbols);
            instructions.push(TInstruction::Return(val));
        }
        ASTStatement::Expression(exp) => {
            emit_tacky(exp, &mut instructions, symbols);
        }
        ASTStatement::If { condition, then_statement, else_statement } => {
            if let Some(else_statement) = else_statement {
                let condition_val = emit_tacky(condition, &mut instructions, symbols);
                let else_label = make_label("else_label");
                let end_label = make_label("end_label");
                instructions.push(TInstruction::JumpIfZero { val: condition_val, label: else_label.clone() });
                let then_instructions = parse_statement(then_statement.as_ref(), switch_map.clone(), symbols);
                instructions.extend(then_instructions);
                instructions.push(TInstruction::Jump { label: end_label.clone() });
                instructions.push(TInstruction::Label { label: else_label });
                let else_instructions = parse_statement(else_statement.as_ref(), switch_map, symbols);
                instructions.extend(else_instructions);
                instructions.push(TInstruction::Label { label: end_label });
            } else {
                let condition_val = emit_tacky(condition, &mut instructions, symbols);
                let end_label = make_label("end_label");
                instructions.push(TInstruction::JumpIfZero { val: condition_val, label: end_label.clone() });
                let then_instructions = parse_statement(then_statement.as_ref(), switch_map, symbols);
                instructions.extend(then_instructions);
                instructions.push(TInstruction::Label { label: end_label });
            }
        }
        ASTStatement::While { condition, body, label } => {
            let break_label = format!("break_{}", label.clone());
            let continue_label = format!("continue_{}", label.clone());
            instructions.push(TInstruction::Label { label: continue_label.clone() });
            let condition_val = emit_tacky(condition, &mut instructions, symbols);
            instructions.push(TInstruction::JumpIfZero { val: condition_val, label: break_label.clone() });
            instructions.extend(parse_statement(body.as_ref(), switch_map, symbols));
            instructions.push(TInstruction::Jump { label: continue_label });
            instructions.push(TInstruction::Label { label: break_label });
        }
        ASTStatement::DoWhile { body, condition, label } => {
            let start_label = format!("start_{}", label.clone());
            let break_label = format!("break_{}", label.clone());
            let continue_label = format!("continue_{}", label.clone());
            instructions.push(TInstruction::Label { label: start_label.clone() });
            instructions.extend(parse_statement(body.as_ref(), switch_map, symbols));
            instructions.push(TInstruction::Label { label: continue_label });
            let condition_val = emit_tacky(condition, &mut instructions, symbols);
            instructions.push(TInstruction::JumpIfNonZero { val: condition_val, label: start_label.clone() });
            instructions.push(TInstruction::Label { label: break_label.clone() });
        }
        ASTStatement::For { init, condition, post, body, label } => {
            let start_label = format!("start_{}", label.clone());
            let break_label = format!("break_{}", label.clone());
            let continue_label = format!("continue_{}", label.clone());

            instructions.extend(parse_forinit(init, symbols));
            instructions.push(TInstruction::Label { label: start_label.clone() });
             if let Some(condition) = condition {
                let condition_val = emit_tacky(condition, &mut instructions, symbols);
                instructions.push(TInstruction::JumpIfZero { val: condition_val, label: break_label.clone() });
            }

            instructions.extend(parse_statement(body.as_ref(), switch_map, symbols));
            instructions.push(TInstruction::Label { label: continue_label.clone() });

            if let Some(post) = post {
                emit_tacky(post, &mut instructions, symbols);
            }

            instructions.push(TInstruction::Jump { label: start_label.clone() });
            instructions.push(TInstruction::Label { label: break_label });
        }
        ASTStatement::Switch { condition, body, label } => {
            let condition_val = emit_tacky(condition, &mut instructions, symbols);
            let tmp_var = TValue::Var(make_temporary(Some(ASTType::Int), symbols));
            for (case_val, case_label) in &switch_map.get(label).unwrap().cases {
                instructions.push(TInstruction::Binary {
                    op: TBinaryOperator::Equal,
                    src1: condition_val.clone(),
                    src2: TValue::Constant(ASTConstant::Long(*case_val)),
                    dst: tmp_var.clone()
                });
                instructions.push(TInstruction::JumpIfNonZero {
                    val: tmp_var.clone(),
                    label: case_label.clone()
                })
            }
            if let Some(default) = &switch_map.get(label).unwrap().default {
                instructions.push(TInstruction::Jump { label: default.clone() });
            }
            instructions.push(TInstruction::Jump { label: format!("break_{}", label.clone()) });
            instructions.extend(parse_statement(body.as_ref(), switch_map, symbols));
            instructions.push(TInstruction::Label { label: format!("break_{}", label.clone()) });
        }
        ASTStatement::Case { value: _, body, label } => {
            instructions.push(TInstruction::Label { label: label.clone() });
            instructions.extend(parse_statement(body.as_ref(), switch_map, symbols));
        }
        ASTStatement::Default { body, label } => {
            instructions.push(TInstruction::Label { label: label.clone() });
            instructions.extend(parse_statement(body.as_ref(), switch_map, symbols));
        }
        ASTStatement::Break { label } => {
            let break_label = format!("break_{}", label.clone());
            instructions.push(TInstruction::Jump { label: break_label });
        }
        ASTStatement::Continue { label } => {
            let continue_label = format!("continue_{}", label.clone());
            instructions.push(TInstruction::Jump { label: continue_label });
        }
        ASTStatement::Compound(block) => {
            let block_instructions = parse_block(block, switch_map, symbols);
            instructions.extend(block_instructions);
        }
        ASTStatement::Goto { label } => {
            instructions.push(TInstruction::Jump { label: label.clone() });
        }
        ASTStatement::LabelledStatement { label, statement } => {
            instructions.push(TInstruction::Label { label: label.clone() });
            let statement_instructions = parse_statement(statement.as_ref(), switch_map, symbols);
            instructions.extend(statement_instructions);
        }
        ASTStatement::Null => {
            // Do nothing
        }
    }

    instructions
}

fn parse_forinit(init: &ASTForInit, symbols: &mut HashMap<String, Symbol>) -> Vec<TInstruction> {
    let mut instructions = Vec::new();

    match init {
        ASTForInit::InitDecl(declaration) => {
            instructions.extend(parse_variable_declaration(declaration, symbols));
        }
        ASTForInit::InitExp(Some(exp)) => {
            emit_tacky(exp, &mut instructions, symbols);
        }
        ASTForInit::InitExp(None) => {
            // Do nothing
        }
    }

    instructions
}

fn parse_declaration(declaration: &ASTDeclaration, symbols: &mut HashMap<String, Symbol>) -> Vec<TInstruction> {
    let mut instructions = Vec::new();

    match declaration {
        ASTDeclaration::VariableDeclaration(variable_declaration) => {
            instructions.extend(parse_variable_declaration(variable_declaration, symbols));
        }
        ASTDeclaration::FunctionDeclaration(_) => {
            // Do nothing. Functions are declared at the top level
        }
    }

    instructions
}

fn parse_variable_declaration(variable_declaration: &ASTVariableDeclaration, symbols: &mut HashMap<String, Symbol>) -> Vec<TInstruction> {
    let mut instructions = Vec::new();

    if variable_declaration.storage_class.is_none() && let Some(init_unwrapped) = variable_declaration.init.as_ref() {
        let result = emit_tacky(init_unwrapped, &mut instructions, symbols);
        instructions.push(TInstruction::Copy { src: result, dst: TValue::Var(variable_declaration.name.clone())});
    }

    instructions
}

fn emit_tacky(
    typed_exp: &ASTTypedExpression,
    instructions: &mut Vec<TInstruction>,
    symbols: &mut HashMap<String, Symbol>,
) -> TValue {
    match typed_exp.exp.clone() {
        ASTExpression::Constant(val) => {
            TValue::Constant(val)
        }
        ASTExpression::Unary {
            op: op @ (ASTUnaryOperator::Increment | ASTUnaryOperator::Decrement),
            exp
        } => {
            let var = emit_tacky(exp.as_ref(), instructions, symbols);
            let increment = match (op, exp.exp_type.clone()) {
                (ASTUnaryOperator::Increment, Some(ASTType::Int)) => TValue::Constant(ASTConstant::Int(1)),
                (ASTUnaryOperator::Increment, Some(ASTType::Long)) => TValue::Constant(ASTConstant::Long(1)),
                (ASTUnaryOperator::Decrement, Some(ASTType::Int)) => TValue::Constant(ASTConstant::Int(-1)),
                (ASTUnaryOperator::Decrement, Some(ASTType::Long)) => TValue::Constant(ASTConstant::Long(-1)),
                _ => panic!("Unsupported unary operator!")
            };
            let tmp = TValue::Var(make_temporary(typed_exp.exp_type.clone(), symbols));
            instructions.push(TInstruction::Binary { op: TBinaryOperator::Add, src1: increment, src2: var.clone(), dst: tmp.clone() });
            instructions.push(TInstruction::Copy { src: tmp, dst: var.clone() });
            var
        }
        ASTExpression::Unary { op, exp} => {
            let src = emit_tacky(exp.as_ref(), instructions, symbols);
            let dst_name = make_temporary(typed_exp.exp_type.clone(), symbols);
            let dst = TValue::Var(dst_name);
            let tacky_op = convert_unop(op.clone());
            instructions.push(TInstruction::Unary { op: tacky_op, src, dst: dst.clone() });
            dst
        }
        ASTExpression::Binary { op: ASTBinaryOperator::And, e1, e2 } => {
            let v1 = emit_tacky(e1.as_ref(), instructions, symbols);
            let false_label = make_label("false_label");
            let dst = TValue::Var(make_temporary(typed_exp.exp_type.clone(), symbols));
            let end_label = make_label("end_label");
            instructions.push(TInstruction::JumpIfZero { val: v1, label: false_label.clone() });
            let v2 = emit_tacky(e2.as_ref(), instructions, symbols);
            instructions.push(TInstruction::JumpIfZero { val: v2, label: false_label.clone() });
            instructions.push(TInstruction::Copy { src: TValue::Constant(ASTConstant::Int(1)), dst: dst.clone() });
            instructions.push(TInstruction::Jump { label: end_label.clone() });
            instructions.push(TInstruction::Label { label: false_label });
            instructions.push(TInstruction::Copy { src: TValue::Constant(ASTConstant::Int(0)), dst: dst.clone() });
            instructions.push(TInstruction::Label { label: end_label });
            dst
        }
        ASTExpression::Binary { op: ASTBinaryOperator::Or, e1, e2 } => {
            let v1 = emit_tacky(e1.as_ref(), instructions, symbols);
            let true_label = make_label("true_label");
            let dst = TValue::Var(make_temporary(typed_exp.exp_type.clone(), symbols));
            let end_label = make_label("end_label");
            instructions.push(TInstruction::JumpIfNonZero { val: v1, label: true_label.clone() });
            let v2 = emit_tacky(e2.as_ref(), instructions, symbols);
            instructions.push(TInstruction::JumpIfNonZero { val: v2, label: true_label.clone() });
            instructions.push(TInstruction::Copy { src: TValue::Constant(ASTConstant::Int(0)), dst: dst.clone() });
            instructions.push(TInstruction::Jump { label: end_label.clone() });
            instructions.push(TInstruction::Label { label: true_label });
            instructions.push(TInstruction::Copy { src: TValue::Constant(ASTConstant::Int(1)), dst: dst.clone() });
            instructions.push(TInstruction::Label { label: end_label });
            dst
        }
        ASTExpression::Binary { op, e1, e2 } => {
            let v1 = emit_tacky(e1.as_ref(), instructions, symbols);
            let v2 = emit_tacky(e2.as_ref(), instructions, symbols);
            let dst_name = make_temporary(typed_exp.exp_type.clone(), symbols);
            let dst = TValue::Var(dst_name);
            let tacky_op = convert_binop(op.clone());
            instructions.push(TInstruction::Binary { op: tacky_op, src1: v1, src2: v2, dst: dst.clone() });
            dst
        }
        ASTExpression::Assignment { lhs, rhs } => {
            if let ASTExpression::Var(var_name) = lhs.exp {
                let result = emit_tacky(rhs.as_ref(), instructions, symbols);
                let var = TValue::Var(var_name.clone());
                instructions.push(TInstruction::Copy { src: result, dst: var.clone() });
                var
            } else {
                panic!("Unsupported assignment");
            }
        }
        ASTExpression::CompoundAssignment { lhs, rhs, operator } => {
            let v1 = emit_tacky(lhs.as_ref(), instructions, symbols);
            let v2 = emit_tacky(rhs.as_ref(), instructions, symbols);
            let dst_name = make_temporary(typed_exp.exp_type.clone(), symbols);
            let dst = TValue::Var(dst_name);
            let tacky_op = convert_binop(operator.clone());
            instructions.push(TInstruction::Binary { op: tacky_op, src1: v1.clone(), src2: v2, dst: dst.clone() });
            instructions.push(TInstruction::Copy { src: dst, dst: v1.clone() });
            v1
        }
        ASTExpression::Var(name) => {
            TValue::Var(name.clone())
        }
        ASTExpression::Postfix { op, exp } => {
            let Some(exp_type) = exp.exp_type.clone() else {
                panic!("Wrong type!");
            };
            let increment = match op.clone() {
                ASTPostfixOperator::Increment => TValue::Constant(ASTConstant::typed_constant(1, exp_type.clone())),
                ASTPostfixOperator::Decrement => TValue::Constant(ASTConstant::typed_constant(-1, exp_type.clone())),
            };
            let var = emit_tacky(exp.as_ref(), instructions, symbols);
            let tmp = TValue::Var(make_temporary(typed_exp.exp_type.clone(), symbols));
            let tmp2 = TValue::Var(make_temporary(typed_exp.exp_type.clone(), symbols));
            instructions.push(TInstruction::Copy { src: var.clone(), dst: tmp2.clone()});
            instructions.push(TInstruction::Binary { op: TBinaryOperator::Add, src1: increment, src2: var.clone(), dst: tmp.clone()});
            instructions.push(TInstruction::Copy { src: tmp.clone(), dst: var.clone()});
            tmp2
        }
        ASTExpression::Conditional { condition, if_true, if_false } => {
            let condition_val = emit_tacky(condition.as_ref(), instructions, symbols);
            let result_name = make_temporary_identifier("result");
            let result = TValue::Var(result_name);
            let e2_label = make_label("e2_label");
            let end_label = make_label("end_label");
            instructions.push(TInstruction::JumpIfZero { val: condition_val, label: e2_label.clone() });
            let v1 = emit_tacky(if_true.as_ref(), instructions, symbols);
            instructions.push(TInstruction::Copy { src: v1, dst: result.clone() });
            instructions.push(TInstruction::Jump { label: end_label.clone() });
            instructions.push(TInstruction::Label { label: e2_label });
            let v2 = emit_tacky(if_false.as_ref(), instructions, symbols);
            instructions.push(TInstruction::Copy { src: v2, dst: result.clone() });
            instructions.push(TInstruction::Label { label: end_label });
            result
        }
        ASTExpression::FunctionCall { name, args } => {
            let mut args_vec = Vec::new();
            for arg in args {
                let tmp_var_name = make_temporary(typed_exp.exp_type.clone(), symbols);
                let tmp_var = TValue::Var(tmp_var_name);
                let arg_val = emit_tacky(&arg, instructions, symbols);
                instructions.push(TInstruction::Copy { src: arg_val, dst: tmp_var.clone() });
                args_vec.push(tmp_var);
            }

            let result = TValue::Var(make_temporary(typed_exp.exp_type.clone(), symbols));
            instructions.push(TInstruction::FunCall { fun_name: name.clone(), args: args_vec, dst: result.clone() });

            result
        }
        ASTExpression::Cast { target_type, exp } => {
            let result = emit_tacky(exp.as_ref(), instructions, symbols);
            let Some(t) = exp.exp_type.clone() else {
                panic!("Wrong type!");
            };
            if target_type.clone() == t {
                return result;
            }
            let dst_name = make_temporary(typed_exp.exp_type.clone(), symbols);
            //symbols.add
            let dst = TValue::Var(dst_name);
            if t == ASTType::Long {
                instructions.push(TInstruction::SignExtend { src: result, dst: dst.clone() });
            } else {
                instructions.push(TInstruction::Truncate { src: result, dst: dst.clone() });
            }

            dst
        }
    }
}

fn make_temporary(
    var_type: Option<ASTType>,
    symbols: &mut HashMap<String, Symbol>
) -> String {
    let Some(t) = var_type.clone() else {
        panic!("Wrong type!");
    };
    let var_name = make_temporary_identifier("tmp");
    symbols.insert(
        var_name.clone(),
        Symbol {
            name: var_name.clone(),
            _type: t,
            identifier_attrs: SymbolAttrs::LocalAttr,
        }
    );
    var_name
}

fn make_temporary_identifier(prefix: &str) -> String {
    let id = get_global_counter();
    if prefix.is_empty() {
        format!("tmp.{}", id)
    } else {
        format!("{}.{}", prefix, id)
    }
}

fn make_label(prefix: &str) -> String {
    let id = get_global_counter();
    format!("_{}_{}", prefix, id)
}

fn convert_unop(op: ASTUnaryOperator) -> TUnaryOperator {
    match op {
        ASTUnaryOperator::Negate => TUnaryOperator::Negate,
        ASTUnaryOperator::Complement => TUnaryOperator::Complement,
        ASTUnaryOperator::Not => TUnaryOperator::Not,
        _ => panic!("Unsupported unary operator"),
    }
}

fn convert_binop(op: ASTBinaryOperator) -> TBinaryOperator {
    match op {
        ASTBinaryOperator::Add => TBinaryOperator::Add,
        ASTBinaryOperator::Subtract => TBinaryOperator::Subtract,
        ASTBinaryOperator::Multiply => TBinaryOperator::Multiply,
        ASTBinaryOperator::Divide => TBinaryOperator::Divide,
        ASTBinaryOperator::Remainder => TBinaryOperator::Remainder,
        ASTBinaryOperator::BitwiseAnd => TBinaryOperator::BitwiseAnd,
        ASTBinaryOperator::BitwiseOr => TBinaryOperator::BitwiseOr,
        ASTBinaryOperator::BitwiseXor => TBinaryOperator::BitwiseXor,
        ASTBinaryOperator::ShiftLeft => TBinaryOperator::ShiftLeft,
        ASTBinaryOperator::ShiftRight => TBinaryOperator::ShiftRight,
        ASTBinaryOperator::Equal => TBinaryOperator::Equal,
        ASTBinaryOperator::NotEqual => TBinaryOperator::NotEqual,
        ASTBinaryOperator::LessThan => TBinaryOperator::LessThan,
        ASTBinaryOperator::GreaterThan => TBinaryOperator::GreaterThan,
        ASTBinaryOperator::LessThanOrEqual => TBinaryOperator::LessThanOrEqual,
        ASTBinaryOperator::GreaterThanOrEqual => TBinaryOperator::GreaterThanOrEqual,
        ASTBinaryOperator::And | ASTBinaryOperator::Or => panic!("Unsupported binary operator"),
    }
}