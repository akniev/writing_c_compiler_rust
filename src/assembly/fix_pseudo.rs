use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::sync::atomic::{AtomicI32, Ordering};
use crate::assembly::assembly_tokens::{ASMInstruction, ASMOperand, ASMProgram, ASMTopLevel};
use crate::parser::typechecker::{Symbol, SymbolAttrs};

static STACK_OFFSETS: LazyLock<Mutex<HashMap<String, i32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static TEMP_COUNTER: AtomicI32 = AtomicI32::new(0);

pub fn fix_pseudo_in_program(program: ASMProgram, symbols: &HashMap<String, Symbol>) -> ASMProgram {
    let mut top_level = Vec::new();
    for top_level_item in program.top_level {
        let n_function = fix_pseudo_in_top_level(top_level_item, symbols);
        top_level.push(n_function);
    }
    ASMProgram { top_level }
}

fn fix_pseudo_in_top_level(top_level: ASMTopLevel, symbols: &HashMap<String, Symbol>) -> ASMTopLevel {
    match top_level {
        ASMTopLevel::Function { name, global, instructions } => {
            TEMP_COUNTER.store(0, std::sync::atomic::Ordering::Relaxed);
            let mut new_instructions = vec![];
            for instruction in instructions {
                let n_instruction = fix_pseudo_in_instruction(instruction, symbols);
                new_instructions.push(n_instruction);
            }
            new_instructions.insert(0, ASMInstruction::AllocateStack(TEMP_COUNTER.load(Ordering::Relaxed)));
            ASMTopLevel::Function { name, global, instructions: new_instructions }
        }
        ASMTopLevel::StaticVariable { name: _, global: _, init: _ } => {
            top_level
        }
    }
}

fn fix_pseudo_in_instruction(instruction: ASMInstruction, symbols: &HashMap<String, Symbol>) -> ASMInstruction {
    match instruction {
        ASMInstruction::Mov { src, dst } => {
            ASMInstruction::Mov { src: fix_pseudo_in_operands(src, symbols), dst: fix_pseudo_in_operands(dst, symbols) }
        }
        ASMInstruction::Unary { unop, operand } => {
            ASMInstruction::Unary { unop, operand: fix_pseudo_in_operands(operand, symbols) }
        }
        ASMInstruction::Binary { binop, operand1, operand2 } => {
            ASMInstruction::Binary {
                binop,
                operand1: fix_pseudo_in_operands(operand1, symbols),
                operand2: fix_pseudo_in_operands(operand2, symbols)
            }
        }
        ASMInstruction::Cmp { operand1, operand2 } => {
            ASMInstruction::Cmp {
                operand1: fix_pseudo_in_operands(operand1, symbols),
                operand2: fix_pseudo_in_operands(operand2, symbols)
            }
        }
        ASMInstruction::Jmp { label } => {
            ASMInstruction::Jmp { label }
        }
        ASMInstruction::JmpCC { cond_code, label } => {
            ASMInstruction::JmpCC { cond_code, label }
        }
        ASMInstruction::SetCC { cond_code, operand } => {
            ASMInstruction::SetCC { cond_code, operand: fix_pseudo_in_operands(operand, symbols) }
        }
        ASMInstruction::Label(label) => {
            ASMInstruction::Label(label)
        }
        ASMInstruction::Idiv(operand) => {
            ASMInstruction::Idiv(fix_pseudo_in_operands(operand, symbols))
        }
        ASMInstruction::Cdq => {
            ASMInstruction::Cdq
        }
        ASMInstruction::AllocateStack(size) => {
            ASMInstruction::AllocateStack(size)
        }
        ASMInstruction::DeallocateStack(size) => {
            ASMInstruction::DeallocateStack(size)
        }
        ASMInstruction::Push(operand) => {
            ASMInstruction::Push(fix_pseudo_in_operands(operand, symbols))
        }
        ASMInstruction::Call(identifier) => {
            ASMInstruction::Call(identifier)
        }
        ASMInstruction::Ret => {
            ASMInstruction::Ret
        }
    }
}

fn fix_pseudo_in_operands(operand: ASMOperand, symbols: &HashMap<String, Symbol>) -> ASMOperand {
    match operand {
        ASMOperand::Imm(value) => ASMOperand::Imm(value),
        ASMOperand::Reg(reg) => ASMOperand::Reg(reg),
        ASMOperand::Pseudo(identifier) => {
            let mut offsets = STACK_OFFSETS.lock().unwrap();

            if let Some(symbol) = symbols.get(&identifier) && matches!(symbol.identifier_attrs, SymbolAttrs::StaticAttr { init: _, global: _ }) {
                ASMOperand::Data(symbol.name.clone())
            } else if let Some(offset) = offsets.get(&identifier) {
                ASMOperand::Stack(-*offset)
            } else {
                TEMP_COUNTER.fetch_add(4, Ordering::Relaxed);
                let offset = TEMP_COUNTER.load(Ordering::Relaxed);
                offsets.insert(identifier.clone(), offset);
                ASMOperand::Stack(-offset)
            }
        },
        ASMOperand::Stack(offset) => ASMOperand::Stack(offset),
        ASMOperand::Data(identifier) => ASMOperand::Data(identifier),
    }
}