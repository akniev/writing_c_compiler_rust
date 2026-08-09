use crate::assembly::assembly_tokens::{ASMBinaryOperator, ASMCondCode, ASMFunctionDefinition, ASMInstruction, ASMOperand, ASMProgram, ASMRegister, ASMUnaryOperator};
use crate::tacky::tacky_tokens::{TBinaryOperator, TInstruction, TProgram, TUnaryOperator, TValue};

pub fn generate_assembly(program: TProgram) -> ASMProgram {
    let function_definition = ASMFunctionDefinition {
        name: program.functions[0].name.clone(),
        instructions: asm_instructions(program.functions[0].body.clone())
    };
    ASMProgram { function_definition }
}

fn asm_instructions(body: Vec<TInstruction>) -> Vec<ASMInstruction> {
    let mut instructions = vec![];

    for instruction in body {
        match instruction {
            TInstruction::Return(val) => {
                instructions.append(vec![
                    ASMInstruction::Mov { src: val.to_operand(), dst: ASMOperand::Reg(ASMRegister::AX) },
                    ASMInstruction::Ret,
                ].as_mut());
            },
            TInstruction::Unary { op, src, dst } => {
                match op {
                    TUnaryOperator::Not => {
                        instructions.append(vec![
                            ASMInstruction::Cmp { operand1: ASMOperand::Imm(0), operand2: src.to_operand() },
                            ASMInstruction::Mov { src: ASMOperand::Imm(0), dst: dst.to_operand() },
                            ASMInstruction::SetCC { cond_code: ASMCondCode::E, operand: dst.to_operand() },
                        ].as_mut());
                    }
                    _ => {
                        instructions.append(vec![
                            ASMInstruction::Mov { src: src.to_operand(), dst: dst.to_operand() },
                            ASMInstruction::Unary { unop: op.to_operator(), operand: dst.to_operand() },
                        ].as_mut());
                    }
                }
            }
            TInstruction::Binary { op, src1, src2, dst } => {
                match op {
                    TBinaryOperator::Divide => {
                        instructions.append(vec![
                            ASMInstruction::Mov { src: src1.to_operand(), dst: ASMOperand::Reg(ASMRegister::AX) },
                            ASMInstruction::Cdq,
                            ASMInstruction::Idiv(src2.to_operand()),
                            ASMInstruction::Mov { src: ASMOperand::Reg(ASMRegister::AX), dst: dst.to_operand()}
                        ].as_mut());
                    }
                    TBinaryOperator::Remainder => {
                        instructions.append(vec![
                            ASMInstruction::Mov { src: src1.to_operand(), dst: ASMOperand::Reg(ASMRegister::AX) },
                            ASMInstruction::Cdq,
                            ASMInstruction::Idiv(src2.to_operand()),
                            ASMInstruction::Mov { src: ASMOperand::Reg(ASMRegister::DX), dst: dst.to_operand()}
                        ].as_mut());
                    }
                    TBinaryOperator::GreaterThan |
                    TBinaryOperator::GreaterThanOrEqual |
                    TBinaryOperator::LessThan |
                    TBinaryOperator::LessThanOrEqual |
                    TBinaryOperator::Equal |
                    TBinaryOperator::NotEqual => {
                        instructions.append(vec![
                            ASMInstruction::Cmp { operand1: src2.to_operand(), operand2: src1.to_operand() },
                            ASMInstruction::Mov { src: ASMOperand::Imm(0), dst: dst.to_operand() },
                            ASMInstruction::SetCC { cond_code: op.to_cond_code(), operand: dst.to_operand() },
                        ].as_mut())
                    }
                    _ => {
                        instructions.append(vec![
                            ASMInstruction::Mov { src: src1.to_operand(), dst: dst.to_operand() },
                            ASMInstruction::Binary { binop: op.to_operator(), operand1: src2.to_operand(), operand2: dst.to_operand() }
                        ].as_mut());
                    }
                }
            }
            TInstruction::JumpIfZero { val, label } => {
                instructions.append(vec![
                    ASMInstruction::Cmp { operand1: ASMOperand::Imm(0), operand2: val.to_operand() },
                    ASMInstruction::JmpCC { cond_code: ASMCondCode::E, label },
                ].as_mut());
            }
            TInstruction::JumpIfNonZero { val, label } => {
                instructions.append(vec![
                    ASMInstruction::Cmp { operand1: ASMOperand::Imm(0), operand2: val.to_operand() },
                    ASMInstruction::JmpCC { cond_code: ASMCondCode::NE, label },
                ].as_mut())
            }
            TInstruction::Jump { label } => {
                instructions.append(vec![
                    ASMInstruction::Jmp { label },
                ].as_mut());
            }
            TInstruction::Label { label } => {
                instructions.append(vec![
                    ASMInstruction::Label(label),
                ].as_mut());
            }
            TInstruction::Copy { src, dst } => {
                instructions.append(vec![
                    ASMInstruction::Mov { src: src.to_operand(), dst: dst.to_operand() },
                ].as_mut());
            }
            _ => {
                panic!("Unsupported instruction");
            }
        }
    }

    instructions
}

impl TValue {
    pub fn to_operand(&self) -> ASMOperand {
        match self {
            TValue::Constant(value) => ASMOperand::Imm(value.clone()),
            TValue::Var(identifier) => ASMOperand::Pseudo(identifier.clone()),
        }
    }
}

impl TUnaryOperator {
    pub fn to_operator(&self) -> ASMUnaryOperator {
        match self {
            TUnaryOperator::Complement => ASMUnaryOperator::Not,
            TUnaryOperator::Negate => ASMUnaryOperator::Neg,
            _ => panic!("Unsupported unary operator"),
        }
    }
}

impl TBinaryOperator {
    pub fn to_operator(&self) -> ASMBinaryOperator {
        match self {
            TBinaryOperator::Add => ASMBinaryOperator::Add,
            TBinaryOperator::Subtract => ASMBinaryOperator::Sub,
            TBinaryOperator::Multiply => ASMBinaryOperator::Mult,
            TBinaryOperator::BitwiseAnd => ASMBinaryOperator::And,
            TBinaryOperator::BitwiseOr => ASMBinaryOperator::Or,
            TBinaryOperator::BitwiseXor => ASMBinaryOperator::Xor,
            TBinaryOperator::ShiftLeft => ASMBinaryOperator::ShiftLeft,
            TBinaryOperator::ShiftRight => ASMBinaryOperator::ShiftRight,
            _ => panic!("Unsupported binary operator"),
        }
    }

    pub fn to_cond_code(&self) -> ASMCondCode {
        match self {
            TBinaryOperator::GreaterThan => ASMCondCode::G,
            TBinaryOperator::GreaterThanOrEqual => ASMCondCode::GE,
            TBinaryOperator::LessThan => ASMCondCode::L,
            TBinaryOperator::LessThanOrEqual => ASMCondCode::LE,
            TBinaryOperator::Equal => ASMCondCode::E,
            TBinaryOperator::NotEqual => ASMCondCode::NE,
            _ => panic!("Unsupported binary operator"),
        }
    }
}
