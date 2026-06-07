use crate::assembly::assembly_tokens::{ASMBinaryOperator, ASMFunctionDefinition, ASMInstruction, ASMOperand, ASMProgram, ASMRegister};

pub fn fix_movs_in_program(program: ASMProgram) -> ASMProgram {
    let n_function = fix_movs_in_function(program.function_definition);
    ASMProgram { function_definition: n_function }
}

fn fix_movs_in_function(func: ASMFunctionDefinition) -> ASMFunctionDefinition {
    let mut new_instructions = vec![];
    for instruction in func.instructions {
        let mut n_instructions = fix_movs_in_instruction(instruction);
        new_instructions.append(&mut n_instructions);
    }
    ASMFunctionDefinition { name: func.name, instructions: new_instructions }
}

fn fix_movs_in_instruction(instruction: ASMInstruction) -> Vec<ASMInstruction> {
    match instruction.clone() {
        ASMInstruction::Mov { src, dst } => {
            match (src.clone(), dst.clone()) {
                (ASMOperand::Stack(_), ASMOperand::Stack(_)) => {
                    vec![
                        ASMInstruction::Mov { src, dst: ASMOperand::Reg(ASMRegister::R10) },
                        ASMInstruction::Mov { src: ASMOperand::Reg(ASMRegister::R10), dst }
                    ]
                }
                _ => {
                    vec![instruction]
                }
            }
        }
        ASMInstruction::Unary { unop: _, operand: _ } => {
            vec![instruction]
        }
        ASMInstruction::Binary { binop: binop @ (ASMBinaryOperator::Add | ASMBinaryOperator::Sub), operand1, operand2 } => {
            match (operand1.clone(), operand2.clone()) {
                (ASMOperand::Stack(_), ASMOperand::Stack(_)) => {
                    vec![
                        ASMInstruction::Mov { src: operand1, dst: ASMOperand::Reg(ASMRegister::R10) },
                        ASMInstruction::Binary { binop, operand1: ASMOperand::Reg(ASMRegister::R10), operand2 }
                    ]
                }
                _ => {
                    vec![instruction]
                }
            }
        }
        ASMInstruction::Binary { binop: binop @ ASMBinaryOperator::Mult, operand1, operand2 } => {
            match operand2 {
                ASMOperand::Stack(_) => {
                    vec![
                        ASMInstruction::Mov { src: operand2.clone(), dst: ASMOperand::Reg(ASMRegister::R11) },
                        ASMInstruction::Binary { binop, operand1, operand2: ASMOperand::Reg(ASMRegister::R11) },
                        ASMInstruction::Mov { src: ASMOperand::Reg(ASMRegister::R11), dst: operand2.clone() }
                    ]
                }
                _ => {
                    vec![instruction]
                }
            }
        }
        ASMInstruction::Idiv(operand) => {
            match operand.clone() {
                ASMOperand::Imm(_) => {
                    vec![
                        ASMInstruction::Mov { src: operand, dst: ASMOperand::Reg(ASMRegister::R10) },
                        ASMInstruction::Idiv(ASMOperand::Reg(ASMRegister::R10))
                    ]
                }
                _ => {
                    vec![instruction]
                }
            }
        }
        ASMInstruction::Cdq => {
            vec![instruction]
        }
        ASMInstruction::AllocateStack(_) => {
            vec![instruction]
        }
        ASMInstruction::Ret => {
            vec![instruction]
        }
    }
}
