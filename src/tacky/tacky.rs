use crate::parser::ast_tokens::{ASTBinaryOperator, ASTExpression, ASTProgram, ASTStatement, ASTUnaryOperator};
use crate::tacky::tacky_tokens::{TBinaryOperator, TFunctionDefinition, TInstruction, TProgram, TUnaryOperator, TValue};

use std::sync::atomic::{AtomicUsize, Ordering};
use crate::parser::ast_tokens::ASTExpression::Binary;

static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn parse_tacky(program: ASTProgram) -> Result<TProgram, String> {
    let ast_function = program.function_definition;
    let function_body = parse_function_body(ast_function.body);
    let t_function = TFunctionDefinition { name: ast_function.name, body: function_body };
    let t_program = TProgram { function_definition: t_function };
    Ok(t_program)
}

fn parse_function_body(body: ASTStatement) -> Vec<TInstruction> {
    let mut instructions = Vec::new();

    match body {
        ASTStatement::Return(expr) => {
            let val = emit_tacky(expr, &mut instructions);
            instructions.push(TInstruction::Return(val));
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
        ASTExpression::Binary { op, lhs: e1, rhs: e2 } => {
            let v1 = emit_tacky(*e1, instructions);
            let v2 = emit_tacky(*e2, instructions);
            let dst_name = make_temporary();
            let dst = TValue::Var(dst_name);
            let tacky_op = convert_binop(op);
            instructions.push(TInstruction::Binary { op: tacky_op, src1: v1, src2: v2, dst: dst.clone() });
            dst
        }
    }
}

fn make_temporary() -> String {
    let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("tmp.{}", id)
}

fn convert_unop(op: ASTUnaryOperator) -> TUnaryOperator {
    match op {
        ASTUnaryOperator::Negate => TUnaryOperator::Negate,
        ASTUnaryOperator::Complement => TUnaryOperator::Complement,
    }
}

fn convert_binop(op: ASTBinaryOperator) -> TBinaryOperator {
    match op {
        ASTBinaryOperator::Add => TBinaryOperator::Add,
        ASTBinaryOperator::Subtract => TBinaryOperator::Subtract,
        ASTBinaryOperator::Multiply => TBinaryOperator::Multiply,
        ASTBinaryOperator::Divide => TBinaryOperator::Divide,
        ASTBinaryOperator::Remainder => TBinaryOperator::Remainder,
    }
}