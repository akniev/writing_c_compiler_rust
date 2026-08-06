use crate::parser::ast_tokens::{ASTBinaryOperator, ASTBlockItem, ASTDeclaration, ASTExpression, ASTFunctionDefinition, ASTPostfixOperator, ASTProgram, ASTStatement, ASTUnaryOperator};
use crate::tacky::tacky_tokens::{TBinaryOperator, TFunctionDefinition, TInstruction, TProgram, TUnaryOperator, TValue};

use crate::common::global_counter::{get_global_counter};

pub fn parse_tacky(program: ASTProgram) -> Result<TProgram, String> {
    let t_function = parse_function(program.function_definition)?;
    let t_program = TProgram { function_definition: t_function };
    Ok(t_program)
}

fn parse_function(function: ASTFunctionDefinition) -> Result<TFunctionDefinition, String> {
    let mut function_body: Vec<TInstruction> = function
        .body
        .iter()
        .flat_map(|block_item| parse_block_item(block_item))
        .collect();
    function_body.push(TInstruction::Return(TValue::Constant(0)));
    let t_function = TFunctionDefinition { name: function.name, body: function_body };
    Ok(t_function)
}

fn parse_block_item(block_item: &ASTBlockItem) -> Vec<TInstruction> {
    match block_item {
        ASTBlockItem::Statement(statement) => {
            parse_statement(&statement)
        }
        ASTBlockItem::Declaration(declaration) => {
            parse_declaration(&declaration)
        }
    }
}

fn parse_statement(statement: &ASTStatement) -> Vec<TInstruction> {
    let mut instructions = Vec::new();

    match statement {
        ASTStatement::Return(expr) => {
            let val = emit_tacky(expr, &mut instructions);
            instructions.push(TInstruction::Return(val));
        }
        ASTStatement::Expression(exp) => {
            emit_tacky(exp, &mut instructions);
        }
        ASTStatement::If { condition, then_statement, else_statement } => {
            if let Some(else_statement) = else_statement {
                let condition_val = emit_tacky(condition, &mut instructions);
                let else_label = make_label("else_label");
                let end_label = make_label("end_label");
                instructions.push(TInstruction::JumpIfZero { val: condition_val, label: else_label.clone() });
                let then_instructions = parse_statement(then_statement.as_ref());
                instructions.extend(then_instructions);
                instructions.push(TInstruction::Jump { label: end_label.clone() });
                instructions.push(TInstruction::Label { label: else_label });
                let else_instructions = parse_statement(else_statement.as_ref());
                instructions.extend(else_instructions);
                instructions.push(TInstruction::Label { label: end_label });
            } else {
                let condition_val = emit_tacky(condition, &mut instructions);
                let end_label = make_label("end_label");
                instructions.push(TInstruction::JumpIfZero { val: condition_val, label: end_label.clone() });
                let then_instructions = parse_statement(then_statement.as_ref());
                instructions.extend(then_instructions);
                instructions.push(TInstruction::Label { label: end_label });
            }
        }
        ASTStatement::Null => {
            // Do nothing
        }
    }

    instructions
}

fn parse_declaration(declaration: &ASTDeclaration) -> Vec<TInstruction> {
    let mut instructions = Vec::new();

    match declaration {
        ASTDeclaration::VariableDeclaration { name: var_name, init} => {
            if let Some(init_unwrapped) = init {
                let result = emit_tacky(init_unwrapped, &mut instructions);
                instructions.push(TInstruction::Copy { src: result, dst: TValue::Var(var_name.clone())});
            }
        }
    }

    instructions
}

fn emit_tacky(exp: &ASTExpression, instructions: &mut Vec<TInstruction>) -> TValue {
    match exp {
        ASTExpression::Constant(val) => {
            TValue::Constant(*val)
        }
        ASTExpression::Unary {
            op: op @ (ASTUnaryOperator::Increment | ASTUnaryOperator::Decrement),
            exp
        } => {
            let var = emit_tacky(exp, instructions);
            let increment = match op {
                ASTUnaryOperator::Increment => TValue::Constant(1),
                ASTUnaryOperator::Decrement => TValue::Constant(-1),
                _ => panic!("Unsupported unary operator!")
            };
            let tmp = TValue::Var(make_temporary());
            instructions.push(TInstruction::Binary { op: TBinaryOperator::Add, src1: increment, src2: var.clone(), dst: tmp.clone() });
            instructions.push(TInstruction::Copy { src: tmp, dst: var.clone() });
            var
        }
        ASTExpression::Unary { op, exp} => {
            let src = emit_tacky(exp, instructions);
            let dst_name = make_temporary();
            let dst = TValue::Var(dst_name);
            let tacky_op = convert_unop(op.clone());
            instructions.push(TInstruction::Unary { op: tacky_op, src, dst: dst.clone() });
            dst
        }
        ASTExpression::Binary { op: ASTBinaryOperator::And, lhs: e1, rhs: e2 } => {
            let v1 = emit_tacky(e1, instructions);
            let false_label = make_label("false_label");
            let dst = TValue::Var(make_temporary());
            let end_label = make_label("end_label");
            instructions.push(TInstruction::JumpIfZero { val: v1, label: false_label.clone() });
            let v2 = emit_tacky(e2, instructions);
            instructions.push(TInstruction::JumpIfZero { val: v2, label: false_label.clone() });
            instructions.push(TInstruction::Copy { src: TValue::Constant(1), dst: dst.clone() });
            instructions.push(TInstruction::Jump { label: end_label.clone() });
            instructions.push(TInstruction::Label { label: false_label });
            instructions.push(TInstruction::Copy { src: TValue::Constant(0), dst: dst.clone() });
            instructions.push(TInstruction::Label { label: end_label });
            dst
        }
        ASTExpression::Binary { op: ASTBinaryOperator::Or, lhs: e1, rhs: e2 } => {
            let v1 = emit_tacky(e1, instructions);
            let true_label = make_label("true_label");
            let dst = TValue::Var(make_temporary());
            let end_label = make_label("end_label");
            instructions.push(TInstruction::JumpIfNonZero { val: v1, label: true_label.clone() });
            let v2 = emit_tacky(e2, instructions);
            instructions.push(TInstruction::JumpIfNonZero { val: v2, label: true_label.clone() });
            instructions.push(TInstruction::Copy { src: TValue::Constant(0), dst: dst.clone() });
            instructions.push(TInstruction::Jump { label: end_label.clone() });
            instructions.push(TInstruction::Label { label: true_label });
            instructions.push(TInstruction::Copy { src: TValue::Constant(1), dst: dst.clone() });
            instructions.push(TInstruction::Label { label: end_label });
            dst
        }
        ASTExpression::Binary { op, lhs: e1, rhs: e2 } => {
            let v1 = emit_tacky(e1, instructions);
            let v2 = emit_tacky(e2, instructions);
            let dst_name = make_temporary();
            let dst = TValue::Var(dst_name);
            let tacky_op = convert_binop(op.clone());
            instructions.push(TInstruction::Binary { op: tacky_op, src1: v1, src2: v2, dst: dst.clone() });
            dst
        }
        ASTExpression::Assignment { lhs, rhs } => {
            if let ASTExpression::Var(var_name) = lhs.as_ref() {
                let result = emit_tacky(rhs, instructions);
                let var = TValue::Var(var_name.clone());
                instructions.push(TInstruction::Copy { src: result, dst: var.clone() });
                var
            } else {
                panic!("Unsupported assignment");
            }
        }
        ASTExpression::CompoundAssignment { lhs, rhs, operator } => {
            let v1 = emit_tacky(lhs.as_ref(), instructions);
            let v2 = emit_tacky(rhs.as_ref(), instructions);
            let dst_name = make_temporary();
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
            let increment = match op.clone() {
                ASTPostfixOperator::Increment => TValue::Constant(1),
                ASTPostfixOperator::Decrement => TValue::Constant(-1)
            };
            let var = emit_tacky(exp, instructions);
            let tmp = TValue::Var(make_temporary());
            let tmp2 = TValue::Var(make_temporary());
            instructions.push(TInstruction::Copy { src: var.clone(), dst: tmp2.clone()});
            instructions.push(TInstruction::Binary { op: TBinaryOperator::Add, src1: increment, src2: var.clone(), dst: tmp.clone()});
            instructions.push(TInstruction::Copy { src: tmp.clone(), dst: var.clone()});
            tmp2
        }
        ASTExpression::Conditional { condition, if_true, if_false } => {
            let condition_val = emit_tacky(condition, instructions);
            let result_name = make_temporary_identifier("result");
            let result = TValue::Var(result_name);
            let e2_label = make_label("e2_label");
            let end_label = make_label("end_label");
            instructions.push(TInstruction::JumpIfZero { val: condition_val, label: e2_label.clone() });
            let v1 = emit_tacky(if_true, instructions);
            instructions.push(TInstruction::Copy { src: v1, dst: result.clone() });
            instructions.push(TInstruction::Jump { label: end_label.clone() });
            instructions.push(TInstruction::Label { label: e2_label });
            let v2 = emit_tacky(if_false, instructions);
            instructions.push(TInstruction::Copy { src: v2, dst: result.clone() });
            instructions.push(TInstruction::Label { label: end_label });
            result
        }
    }
}

fn make_temporary() -> String {
    make_temporary_identifier("tmp")
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