use crate::assembly::assembly_tokens::{ASMBinaryOperator, ASMFunctionDefinition, ASMInstruction, ASMOperand, ASMProgram, ASMRegister, ASMUnaryOperator};
use crate::tacky::tacky_tokens::{TBinaryOperator, TInstruction, TProgram, TUnaryOperator, TValue};

pub fn generate_assembly(program: TProgram) -> ASMProgram {
    let function_definition = ASMFunctionDefinition {
        name: program.function_definition.name,
        instructions: asm_instructions(program.function_definition.body)
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
                instructions.append(vec![
                    ASMInstruction::Mov { src: src.to_operand(), dst: dst.to_operand() },
                    ASMInstruction::Unary { unop: op.to_operator(), operand: dst.to_operand() },
                ].as_mut());
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
                    _ => {
                        instructions.append(vec![
                            ASMInstruction::Mov { src: src1.to_operand(), dst: dst.to_operand() },
                            ASMInstruction::Binary { binop: op.to_operator(), operand1: src2.to_operand(), operand2: dst.to_operand() }
                        ].as_mut());
                    }
                }
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
        }
    }
}

impl TBinaryOperator {
    pub fn to_operator(&self) -> ASMBinaryOperator {
        match self {
            TBinaryOperator::Add => ASMBinaryOperator::Add,
            TBinaryOperator::Subtract => ASMBinaryOperator::Sub,
            TBinaryOperator::Multiply => ASMBinaryOperator::Mult,
            _ => panic!("Unsupported binary operator"),
        }
    }
}
