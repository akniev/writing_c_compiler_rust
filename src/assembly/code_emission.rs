use std::collections::HashMap;
use crate::assembly::assembly_tokens::{ASMCondCode, ASMInstruction, ASMProgram, ASMRegSize, ASMTopLevel, ASMType};
use crate::parser::ast_tokens::ASTType;
use crate::parser::typechecker::{StaticInit, Symbol};

pub fn generate_code(asm: ASMProgram, symbols: &HashMap<String, Symbol>) -> String {
    let mut result = String::new();

    for function in asm.top_level {
        print_top_level(function, symbols, &mut result);
    }

    result.push_str(".section .note.GNU-stack,\"\",@progbits");

    result
}

fn print_top_level(top_level: ASMTopLevel, symbols: &HashMap<String, Symbol>, result: &mut String) {
    match top_level {
        ASMTopLevel::Function { name, global, instructions } => {
            if global {
                result.push_str(&format!("  .globl {}\n", name));
            }
            result.push_str("  .text\n");
            result.push_str(&format!("{}:\n", name));
            result.push_str("  pushq %rbp\n");
            result.push_str("  movq %rsp, %rbp\n");
            for inst in instructions {
                print_instruction(inst, symbols, result);
            }
        }
        ASMTopLevel::StaticVariable { name, global, alignment, init } => {
            if global {
                result.push_str(&format!("  .globl {}\n", name))
            }
            if init.value() == 0 {
                result.push_str("  .bss\n");
                result.push_str(&format!("  .align {}\n", alignment));
                result.push_str(&format!("{}:\n", name));
                result.push_str(&format!("  .zero {}\n", init.asm_type().size()));
            } else {
                result.push_str("  .data\n");
                result.push_str(&format!("  .align {}\n", alignment));
                result.push_str(&format!("{}:\n", name));
                match init {
                    StaticInit::IntInit(val) => {
                        result.push_str(&format!("  .long {}\n", val));
                    }
                    StaticInit::LongInit(val) => {
                        result.push_str(&format!("  .quad {}\n", val));
                    }
                }

            }

        }
    }

}

fn print_instruction(inst: ASMInstruction, symbols: &HashMap<String, Symbol>, result: &mut String) {
    match inst {
        ASMInstruction::Mov { _type, src, dst } => {
            let suffix = _type.standard_suffix();
            let reg_size = _type.reg_size();
            result.push_str(&format!("  mov{} {}, {}\n", suffix, src.to_string(reg_size.clone()), dst.to_string(reg_size)));
        }
        ASMInstruction::Movsx { src, dst } => {
            let reg_size = ASMRegSize::EightBytes;
            result.push_str(&format!("  movslq {}, {}\n", src.to_string(ASMRegSize::FourBytes), dst.to_string(reg_size)));
        }
        ASMInstruction::Unary { unop, _type, operand } => {
            let suffix = _type.standard_suffix();
            let reg_size = _type.reg_size();
            result.push_str(&format!("  {}{} {}\n", unop.to_string(), suffix, operand.to_string(reg_size)));
        }
        ASMInstruction::Binary { binop, _type, operand1, operand2 } => {
            let suffix = _type.standard_suffix();
            let reg_size = _type.reg_size();
            result.push_str(&format!("  {}{} {}, {}\n", binop.to_string(), suffix, operand1.to_string(reg_size.clone()), operand2.to_string(reg_size)));
        }
        ASMInstruction::Idiv { _type, op } => {
            let suffix = _type.standard_suffix();
            let reg_size = _type.reg_size();
            result.push_str(&format!("  idiv{} {}\n", suffix, op.to_string(reg_size)));
        }
        ASMInstruction::Cdq { _type } => {
            match _type {
                ASMType::Longword => {
                    result.push_str("  cdq\n");
                }
                ASMType::Quadword => {
                    result.push_str("  cqo\n");
                }
            }
        }
        ASMInstruction::Push(operand) => {
            result.push_str(&format!("  pushq {}\n", operand.to_string_8b()));
        }
        ASMInstruction::Call(identifier) => {
            let Some(symbol) = symbols.get(&identifier) else {
                panic!("Symbol {} not found", identifier);
            };
            if !matches!(symbol._type, ASTType::FunType(_, _)) {
                panic!("{} is not a function", identifier);
            }
            if symbol.identifier_attrs.is_defined() {
                result.push_str(&format!("  call {}\n", identifier));
            } else {
                result.push_str(&format!("  call {}@PLT\n", identifier));
            }
        }
        ASMInstruction::Ret => {
            result.push_str("  movq %rbp, %rsp\n");
            result.push_str("  popq %rbp\n");
            result.push_str("  ret\n");
        }
        ASMInstruction::Cmp { _type, operand1, operand2 } => {
            let suffix = _type.standard_suffix();
            let reg_size = _type.reg_size();
            result.push_str(&format!("  cmp{} {}, {}\n", suffix, operand1.to_string(reg_size.clone()), operand2.to_string(reg_size)));
        }
        ASMInstruction::Jmp { label } => {
            result.push_str(&format!("  jmp .L{}\n", label));
        }
        ASMInstruction::JmpCC { cond_code, label } => {
            result.push_str(&format!("  j{} .L{}\n", cond_code.to_string(), label));
        }
        ASMInstruction::SetCC { cond_code, operand } => {
            result.push_str(&format!("  set{} {}\n", cond_code.to_string(), operand.to_string(ASMRegSize::OneByte)));
        }
        ASMInstruction::Label(label) => {
            result.push_str(&format!(".L{}:\n", label));
        }
    }
}

impl ASMCondCode {
    pub fn to_string(&self) -> String {
        match self {
            ASMCondCode::E => "e".to_string(),
            ASMCondCode::NE => "ne".to_string(),
            ASMCondCode::G => "g".to_string(),
            ASMCondCode::GE => "ge".to_string(),
            ASMCondCode::L => "l".to_string(),
            ASMCondCode::LE => "le".to_string(),
        }
    }
}