use crate::assembly::assembly_tokens::{ASMBinaryOperator, ASMInstruction, ASMOperand, ASMProgram, ASMRegister, ASMTopLevel, ASMType};

pub fn fix_movs_in_program(program: ASMProgram) -> ASMProgram {
    let mut top_level = vec![];
    for top_level_item in program.top_level {
        let fixed_top_level_item = fix_movs_in_top_level(top_level_item);
        top_level.push(fixed_top_level_item);
    }
    ASMProgram { top_level }
}

fn fix_movs_in_top_level(top_level: ASMTopLevel) -> ASMTopLevel {
    match top_level {
        ASMTopLevel::Function { name, global, instructions } => {
            fixup_function(name, global, instructions)
        }
        ASMTopLevel::StaticVariable { name: _, global: _, alignment: _, init: _ } => {
            top_level
        }
    }
}

fn fixup_function(name: String, global: bool, instructions: Vec<ASMInstruction>) -> ASMTopLevel {
    let mut new_instructions = vec![];

    for instruction in instructions {
        let mut n_instructions = fix_movs_in_instruction(instruction);
        new_instructions.append(&mut n_instructions);
    }

    ASMTopLevel::Function { name, global, instructions: new_instructions }
}

fn fix_movs_in_instruction(instruction: ASMInstruction) -> Vec<ASMInstruction> {
    match instruction.clone() {
        ASMInstruction::Movsx { src, dst } => {
            let mut n_instructions = vec![];
            let mut n_src = src.clone();
            let mut n_dst = dst.clone();
            if matches!(src, ASMOperand::Imm(_)) {
                n_src = ASMOperand::Reg(ASMRegister::R10);
                n_instructions.push(
                    ASMInstruction::Mov { _type: ASMType::Longword, src: src.clone(), dst: n_src.clone() }
                );
            }

            if matches!(dst, ASMOperand::Stack(_)) {
                n_dst = ASMOperand::Reg(ASMRegister::R11);
                n_instructions.push(
                    ASMInstruction::Movsx { src: n_src, dst: n_dst.clone() }
                );
                n_instructions.push(
                    ASMInstruction::Mov { _type: ASMType::Quadword, src: n_dst, dst: dst.clone() }
                );
            } else {
                n_instructions.push(ASMInstruction::Movsx { src: n_src, dst: n_dst.clone() });
            }

            n_instructions
        }
        ASMInstruction::Mov { _type, src, dst } => {
            let mut n_instructions = vec![];
            let mut src = src;

            if let ASMOperand::Imm(val) = &src && matches!(_type, ASMType::Quadword) {
                if *val < i64::from(i32::MIN) || *val > i64::from(i32::MAX) {
                    n_instructions.push(
                        ASMInstruction::Mov {
                            _type: ASMType::Quadword,
                            src,
                            dst: ASMOperand::Reg(ASMRegister::R10)
                        }
                    );
                    src = ASMOperand::Reg(ASMRegister::R10);
                }
            }

            match (src.clone(), dst.clone()) {
                (ASMOperand::Stack(_) | ASMOperand::Data(_), ASMOperand::Stack(_) | ASMOperand::Data(_)) => {
                    n_instructions.extend(vec![
                        ASMInstruction::Mov { _type, src, dst: ASMOperand::Reg(ASMRegister::R10) },
                        ASMInstruction::Mov { _type, src: ASMOperand::Reg(ASMRegister::R10), dst }
                    ]);
                }
                _ => {
                    n_instructions.extend(vec![instruction]);
                }
            }
            n_instructions
        }
        ASMInstruction::Cmp { _type, operand1, operand2 } => {
            match (operand1.clone(), operand2.clone()) {
                (ASMOperand::Stack(_) | ASMOperand::Data(_), ASMOperand::Stack(_) | ASMOperand::Data(_)) => {
                    vec![
                        ASMInstruction::Mov { _type, src: operand1, dst: ASMOperand::Reg(ASMRegister::R10) },
                        ASMInstruction::Cmp { _type, operand1: ASMOperand::Reg(ASMRegister::R10), operand2 }
                    ]
                }
                (operand1, operand2 @ ASMOperand::Imm(_)) => {
                    vec![
                        ASMInstruction::Mov { _type, src: operand2, dst: ASMOperand::Reg(ASMRegister::R11) },
                        ASMInstruction::Cmp { _type, operand1, operand2: ASMOperand::Reg(ASMRegister::R11) }
                    ]
                }
                _ => {
                    vec![instruction]
                }
            }
        }
        ASMInstruction::Binary {
            binop: binop @ (
                ASMBinaryOperator::Add
                | ASMBinaryOperator::Sub
                | ASMBinaryOperator::And
                | ASMBinaryOperator::Or
                | ASMBinaryOperator::Xor
            ),
            _type,
            operand1,
            operand2
        } => {
            match (operand1.clone(), operand2.clone()) {
                (ASMOperand::Stack(_) | ASMOperand::Data(_), ASMOperand::Stack(_) | ASMOperand::Data(_)) => {
                    vec![
                        ASMInstruction::Mov { _type, src: operand1, dst: ASMOperand::Reg(ASMRegister::R10) },
                        ASMInstruction::Binary { binop, _type, operand1: ASMOperand::Reg(ASMRegister::R10), operand2 }
                    ]
                }
                _ => {
                    vec![instruction]
                }
            }
        }
        ASMInstruction::Binary {
            binop: binop @ ASMBinaryOperator::Mult,
            _type,
            operand1,
            operand2
        } => {
            match operand2 {
                ASMOperand::Stack(_) | ASMOperand::Data(_) => {
                    vec![
                        ASMInstruction::Mov { _type, src: operand2.clone(), dst: ASMOperand::Reg(ASMRegister::R11) },
                        ASMInstruction::Binary { binop, _type, operand1, operand2: ASMOperand::Reg(ASMRegister::R11) },
                        ASMInstruction::Mov { _type, src: ASMOperand::Reg(ASMRegister::R11), dst: operand2.clone() }
                    ]
                }
                _ => {
                    vec![instruction]
                }
            }
        }
        ASMInstruction::Binary {
            binop: binop@ (ASMBinaryOperator::ShiftLeft | ASMBinaryOperator::ShiftRight),
            _type,
            operand1,
            operand2
        } => {
            match operand1 {
                ASMOperand::Imm(_) => {
                    vec![instruction]
                }
                _ => {
                    vec![
                        ASMInstruction::Mov { _type, src: operand1.clone(), dst: ASMOperand::Reg(ASMRegister::CX) },
                        ASMInstruction::Binary { binop, _type, operand1: ASMOperand::Reg(ASMRegister::CX), operand2 },
                    ]
                }
            }
        }
        ASMInstruction::Idiv { _type, op } => {
            match op.clone() {
                ASMOperand::Imm(_) => {
                    vec![
                        ASMInstruction::Mov { _type, src: op, dst: ASMOperand::Reg(ASMRegister::R10) },
                        ASMInstruction::Idiv { _type, op: ASMOperand::Reg(ASMRegister::R10) }
                    ]
                }
                _ => {
                    vec![instruction]
                }
            }
        }
        ASMInstruction::Cdq { _type: _ } |
        ASMInstruction::Ret |
        ASMInstruction::Push(_) |
        ASMInstruction::Call(_) |
        ASMInstruction::Jmp { label: _ } |
        ASMInstruction::JmpCC { cond_code: _, label: _ } |
        ASMInstruction::SetCC { cond_code: _, operand: _ } |
        ASMInstruction::Label(_) |
        ASMInstruction::Unary { unop: _, _type: _, operand: _ } => {
            vec![instruction]
        }
    }
}
