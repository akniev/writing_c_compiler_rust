use crate::assembly::assembly_tokens::{ASMBinaryOperator, ASMCondCode, ASMInstruction, ASMOperand, ASMProgram, ASMRegister, ASMTopLevel, ASMUnaryOperator};
use crate::tacky::tacky_tokens::{TBinaryOperator, TFunction, TInstruction, TProgram, TTopLevel, TUnaryOperator, TValue};

pub fn generate_assembly(program: TProgram) -> ASMProgram {
    let mut top_level = vec![];

    for item in program.top_level {
        let top_level_item = generate_assembly_for_top_level(item);
        top_level.push(top_level_item);
    }

    ASMProgram { top_level }
}

fn generate_assembly_for_top_level(item: TTopLevel) -> ASMTopLevel {
    match item {
        TTopLevel::Function(func) => {
            generate_assembly_for_function(func)
        }
        TTopLevel::StaticVariable(static_variable) => {
            ASMTopLevel::StaticVariable {
                name: static_variable.name,
                global: static_variable.global,
                init: static_variable.init,
            }
        }
    }
}

fn generate_assembly_for_function(function: TFunction) -> ASMTopLevel {
    let arg_registers = vec![
        ASMRegister::DI,
        ASMRegister::SI,
        ASMRegister::DX,
        ASMRegister::CX,
        ASMRegister::R8,
        ASMRegister::R9
    ];

    let mut instructions = vec![];

    let split_at = function.params.len().min(6);
    let register_params = &function.params[..split_at];
    let stack_params = &function.params[split_at..];

    let mut reg_index = 0;
    for reg_param in register_params {
        let r = arg_registers[reg_index];
        instructions.push(
            ASMInstruction::Mov {
                src: ASMOperand::Reg(r),
                dst: ASMOperand::Pseudo(reg_param.clone())
            }
        );
        reg_index += 1;
    }

    let mut stack_index = 0;
    for stack_param in stack_params {
        instructions.push(
            ASMInstruction::Mov {
                src: ASMOperand::Stack(16 + stack_index * 8),
                dst: ASMOperand::Pseudo(stack_param.clone())
            }
        );
        stack_index += 1;
    }

    instructions.extend(asm_instructions(function.body));
    let asm_function = ASMTopLevel::Function {
        name: function.name.clone(),
        global: function.global,
        instructions,
    };
    asm_function
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
            TInstruction::FunCall { fun_name, args, dst } => {
                let arg_registers = vec![
                    ASMRegister::DI,
                    ASMRegister::SI,
                    ASMRegister::DX,
                    ASMRegister::CX,
                    ASMRegister::R8,
                    ASMRegister::R9
                ];

                let split_at = args.len().min(6);
                let register_args = &args[..split_at];
                let stack_args = &args[split_at..];
                let stack_padding = if stack_args.len() % 2 != 0 {
                    8
                } else {
                    0
                };

                if stack_padding != 0 {
                    instructions.push(ASMInstruction::AllocateStack(stack_padding));
                }

                // pass args in registers
                let mut reg_index = 0;
                for tacky_arg in register_args {
                    let r = arg_registers[reg_index];
                    let assembly_arg = tacky_arg.to_operand();
                    instructions.push(ASMInstruction::Mov { src: assembly_arg, dst: ASMOperand::Reg(r) });
                    reg_index += 1;
                }

                // pass args on stack
                for tacky_arg in stack_args.iter().rev() {
                    let assembly_arg = tacky_arg.to_operand();
                    match assembly_arg {
                        ASMOperand::Reg(_) | ASMOperand::Imm(_) => {
                            instructions.push(ASMInstruction::Push(assembly_arg));
                        }
                        ASMOperand::Pseudo(_) | ASMOperand::Stack(_) | ASMOperand::Data(_) => {
                            instructions.push(ASMInstruction::Mov { src: assembly_arg, dst: ASMOperand::Reg(ASMRegister::AX) });
                            instructions.push(ASMInstruction::Push(ASMOperand::Reg(ASMRegister::AX)));
                        }
                    }
                }

                // emit call instruction
                instructions.push(ASMInstruction::Call(fun_name.clone()));

                // adjust stack pointer
                let bytes_to_remove: i32 = 8 * (stack_args.len() as i32) + stack_padding;
                if bytes_to_remove > 0 {
                    instructions.push(ASMInstruction::DeallocateStack(bytes_to_remove));
                }

                // retrieve return value
                let assembly_dst = dst.to_operand();
                instructions.push(ASMInstruction::Mov { src: ASMOperand::Reg(ASMRegister::AX), dst: assembly_dst });
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
