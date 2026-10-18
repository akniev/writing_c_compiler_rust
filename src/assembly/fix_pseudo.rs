use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::sync::atomic::{AtomicI64, Ordering};
use crate::assembly::assembly_tokens::{ASMInstruction, ASMOperand, ASMProgram, ASMTopLevel, ASMType};
use crate::assembly::backend_symbol_table::ASMSymtabEntry;

static STACK_OFFSETS: LazyLock<Mutex<HashMap<String, i64>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static TEMP_COUNTER: AtomicI64 = AtomicI64::new(0);

pub fn fix_pseudo_in_program(program: ASMProgram, symbols: &HashMap<String, ASMSymtabEntry>) -> ASMProgram {
    let mut top_level = Vec::new();
    for top_level_item in program.top_level {
        let n_function = fix_pseudo_in_top_level(top_level_item, symbols);
        top_level.push(n_function);
    }
    ASMProgram { top_level }
}

fn fix_pseudo_in_top_level(top_level: ASMTopLevel, symbols: &HashMap<String, ASMSymtabEntry>) -> ASMTopLevel {
    match top_level {
        ASMTopLevel::Function { name, global, instructions } => {
            TEMP_COUNTER.store(0, Ordering::Relaxed);
            let mut new_instructions = vec![];
            for instruction in instructions {
                let n_instruction = fix_pseudo_in_instruction(instruction, symbols);
                new_instructions.push(n_instruction);
            }
            new_instructions.insert(
                0,
                ASMInstruction::allocate_stack(TEMP_COUNTER.load(Ordering::Relaxed))
            );
            ASMTopLevel::Function { name, global, instructions: new_instructions }
        }
        ASMTopLevel::StaticVariable { name: _, global: _, alignment: _, init: _ } => {
            top_level
        }
    }
}

fn fix_pseudo_in_instruction(instruction: ASMInstruction, symbols: &HashMap<String, ASMSymtabEntry>) -> ASMInstruction {
    match instruction {
        ASMInstruction::Mov { _type, src, dst } => {
            ASMInstruction::Mov {
                _type,
                src: fix_pseudo_in_operands(src, symbols),
                dst: fix_pseudo_in_operands(dst, symbols)
            }
        }
        ASMInstruction::Unary { _type, unop, operand } => {
            ASMInstruction::Unary {
                _type,
                unop,
                operand: fix_pseudo_in_operands(operand, symbols)
            }
        }
        ASMInstruction::Binary { _type, binop, operand1, operand2 } => {
            ASMInstruction::Binary {
                _type,
                binop,
                operand1: fix_pseudo_in_operands(operand1, symbols),
                operand2: fix_pseudo_in_operands(operand2, symbols)
            }
        }
        ASMInstruction::Cmp { _type, operand1, operand2 } => {
            ASMInstruction::Cmp {
                _type,
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
        ASMInstruction::Idiv { _type, op: operand } => {
            ASMInstruction::Idiv {
                _type,
                op: fix_pseudo_in_operands(operand, symbols)
            }
        }
        ASMInstruction::Cdq { _type} => {
            ASMInstruction::Cdq { _type }
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
        ASMInstruction::Movsx { src, dst } => {
            ASMInstruction::Movsx {
                src: fix_pseudo_in_operands(src, symbols),
                dst: fix_pseudo_in_operands(dst, symbols)
            }
        }
    }
}

fn fix_pseudo_in_operands(operand: ASMOperand, symbols: &HashMap<String, ASMSymtabEntry>) -> ASMOperand {
    match operand {
        ASMOperand::Imm(value) => ASMOperand::Imm(value),
        ASMOperand::Reg(reg) => ASMOperand::Reg(reg),
        ASMOperand::Pseudo(identifier) => {
            let mut offsets = STACK_OFFSETS.lock().unwrap();
            if let Some(symbol) = symbols.get(&identifier) && symbol.is_static() {
                ASMOperand::Data(identifier.clone())
            } else if let Some(offset) = offsets.get(&identifier) {
                ASMOperand::Stack(-*offset)
            } else {
                let Some(symbol) = symbols.get(&identifier) else {
                    panic!("symbol not found: {}", identifier);
                };
                let _type = symbol.get_type();
                let offset = match _type {
                    ASMType::Longword => 4,
                    ASMType::Quadword => 8,
                };
                let alignment = _type.alignment() as i64;
                let offset = align_up(offset, alignment);
                TEMP_COUNTER.fetch_add(offset, Ordering::Relaxed);
                let offset = TEMP_COUNTER.load(Ordering::Relaxed);
                offsets.insert(identifier.clone(), offset);
                ASMOperand::Stack(-offset)
            }
        },
        ASMOperand::Stack(offset) => ASMOperand::Stack(offset),
        ASMOperand::Data(identifier) => ASMOperand::Data(identifier),
    }
}

fn align_up(value: i64, alignment: i64) -> i64 {
    ((value + alignment - 1) / alignment) * alignment
}