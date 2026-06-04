use crate::assembly_tokens::{ASMFunctionDefinition, ASMInstruction, ASMOperand, ASMProgram, ASMRegister};

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
                (ASMOperand::Stack(offset1), ASMOperand::Stack(offset2)) => {
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
        ASMInstruction::Unary { unop, operand } => {
            vec![instruction]
        }
        ASMInstruction::AllocateStack(size) => {
            vec![instruction]
        }
        ASMInstruction::Ret => {
            vec![instruction]
        }
    }
}
