use std::collections::HashMap;
use crate::assembly::assembly_tokens::{ASMCondCode, ASMInstruction, ASMProgram, ASMRegSize, ASMTopLevel};
use crate::parser::typechecker::{Symbol, SymbolType};

pub fn generate_code(asm: ASMProgram, symbols: &HashMap<String, Symbol>) -> String {
    let mut result = String::new();

    for function in asm.top_level {
        print_top_level(function, symbols, &mut result);
    }

    result.push_str(".section .note.GNU-stack,\"\",@progbits");

    return result;
}

fn print_top_level(top_level: ASMTopLevel, symbols: &HashMap<String, Symbol>, result: &mut String) {
    match top_level {
        ASMTopLevel::Function { name, global, instructions } => {
            result.push_str(&format!("  .globl {}\n", name));
            result.push_str(&format!("{}:\n", name));
            result.push_str("  pushq %rbp\n");
            result.push_str("  movq %rsp, %rbp\n");
            for inst in instructions {
                print_instruction(inst, symbols, result);
            }
        }
        ASMTopLevel::StaticVariable { name, global, init } => {
            panic!("Not implemented yet");
        }
    }

}

fn print_instruction(inst: ASMInstruction, symbols: &HashMap<String, Symbol>, result: &mut String) {
    match inst {
        ASMInstruction::Mov { src, dst } => {
            result.push_str(&format!("  movl {}, {}\n", src.to_string_4b(), dst.to_string_4b()));
        }
        ASMInstruction::Unary { unop, operand } => {
            result.push_str(&format!("  {} {}\n", unop.to_string(), operand.to_string_4b()));
        }
        ASMInstruction::Binary { binop, operand1, operand2 } => {
            result.push_str(&format!("  {} {}, {}\n", binop.to_string(), operand1.to_string_4b(), operand2.to_string_4b()));
        }
        ASMInstruction::Idiv(operand) => {
            result.push_str(&format!("  idiv {}\n", operand.to_string_4b()));
        }
        ASMInstruction::Cdq => {
            result.push_str("  cdq\n");
        }
        ASMInstruction::AllocateStack(size) => {
            result.push_str(&format!("  subq ${}, %rsp\n", size));
        }
        ASMInstruction::DeallocateStack(size) => {
            result.push_str(&format!("  addq ${}, %rsp\n", size))
        }
        ASMInstruction::Push(operand) => {
            result.push_str(&format!("  pushq {}\n", operand.to_string_8b()));
        }
        ASMInstruction::Call(identifier) => {
            let Some(symbol) = symbols.get(&identifier) else {
                panic!("Symbol {} not found", identifier);
            };
            if !matches!(symbol._type, SymbolType::FunType(_)) {
                panic!("{} is not a function", identifier);
            }
            // TODO: Fix
            // if symbol.defined {
            //     result.push_str(&format!("  call {}\n", identifier));
            // } else {
            //     result.push_str(&format!("  call {}@PLT\n", identifier));
            // }
        }
        ASMInstruction::Ret => {
            result.push_str("  movq %rbp, %rsp\n");
            result.push_str("  popq %rbp\n");
            result.push_str("  ret\n");
        }
        ASMInstruction::Cmp { operand1, operand2 } => {
            result.push_str(&format!("  cmpl {}, {}\n", operand1.to_string_4b(), operand2.to_string_4b()));
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