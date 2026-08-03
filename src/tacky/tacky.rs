use crate::parser::ast_tokens::{ASTBinaryOperator, ASTExpression, ASTProgram, ASTStatement, ASTUnaryOperator};
use crate::tacky::tacky_tokens::{TBinaryOperator, TFunctionDefinition, TInstruction, TProgram, TUnaryOperator, TValue};

use crate::common::global_counter::{get_global_counter};

pub fn parse_tacky(program: ASTProgram) -> Result<TProgram, String> {
    panic!("Not implemented");
    // let ast_function = program.function_definition;
    // let function_body = parse_function_body(ast_function.body);
    // let t_function = TFunctionDefinition { name: ast_function.name, body: function_body };
    // let t_program = TProgram { function_definition: t_function };
    // Ok(t_program)
}

fn parse_function_body(body: ASTStatement) -> Vec<TInstruction> {
    let mut instructions = Vec::new();

    match body {
        ASTStatement::Return(expr) => {
            let val = emit_tacky(expr, &mut instructions);
            instructions.push(TInstruction::Return(val));
        }
        _ => {
            panic!("Unsupported statement");
        }
    }

    return instructions;
}

fn emit_tacky(exp: ASTExpression, instructions: &mut Vec<TInstruction>) -> TValue {
    match exp {
        ASTExpression::Constant(val) => {
            TValue::Constant(val)
        }
        ASTExpression::Unary { op, exp} => {
            let src = emit_tacky(*exp, instructions);
            let dst_name = make_temporary();
            let dst = TValue::Var(dst_name);
            let tacky_op = convert_unop(op);
            instructions.push(TInstruction::Unary { op: tacky_op, src, dst: dst.clone() });
            dst
        }
        ASTExpression::Binary { op: ASTBinaryOperator::And, lhs: e1, rhs: e2 } => {
            let v1 = emit_tacky(*e1, instructions);
            let false_label = make_label("false_label");
            let dst = TValue::Var(make_temporary());
            let end_label = make_label("end_label");
            instructions.push(TInstruction::JumpIfZero { val: v1, label: false_label.clone() });
            let v2 = emit_tacky(*e2, instructions);
            instructions.push(TInstruction::JumpIfZero { val: v2, label: false_label.clone() });
            instructions.push(TInstruction::Copy { src: TValue::Constant(1), dst: dst.clone() });
            instructions.push(TInstruction::Jump { label: end_label.clone() });
            instructions.push(TInstruction::Label { label: false_label });
            instructions.push(TInstruction::Copy { src: TValue::Constant(0), dst: dst.clone() });
            instructions.push(TInstruction::Label { label: end_label });
            dst
        }
        ASTExpression::Binary { op: ASTBinaryOperator::Or, lhs: e1, rhs: e2 } => {
            let v1 = emit_tacky(*e1, instructions);
            let true_label = make_label("true_label");
            let dst = TValue::Var(make_temporary());
            let end_label = make_label("end_label");
            instructions.push(TInstruction::JumpIfNonZero { val: v1, label: true_label.clone() });
            let v2 = emit_tacky(*e2, instructions);
            instructions.push(TInstruction::JumpIfNonZero { val: v2, label: true_label.clone() });
            instructions.push(TInstruction::Copy { src: TValue::Constant(0), dst: dst.clone() });
            instructions.push(TInstruction::Jump { label: end_label.clone() });
            instructions.push(TInstruction::Label { label: true_label });
            instructions.push(TInstruction::Copy { src: TValue::Constant(1), dst: dst.clone() });
            instructions.push(TInstruction::Label { label: end_label });
            dst
        }
        ASTExpression::Binary { op, lhs: e1, rhs: e2 } => {
            let v1 = emit_tacky(*e1, instructions);
            let v2 = emit_tacky(*e2, instructions);
            let dst_name = make_temporary();
            let dst = TValue::Var(dst_name);
            let tacky_op = convert_binop(op);
            instructions.push(TInstruction::Binary { op: tacky_op, src1: v1, src2: v2, dst: dst.clone() });
            dst
        }
        _ => {
            panic!("Unsupported expression");
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