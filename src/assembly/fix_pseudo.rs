use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::sync::atomic::{AtomicI32, Ordering};
use crate::assembly::assembly_tokens::{ASMFunctionDefinition, ASMInstruction, ASMOperand, ASMProgram};

static STACK_OFFSETS: LazyLock<Mutex<HashMap<String, i32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static TEMP_COUNTER: AtomicI32 = AtomicI32::new(0);

pub fn fix_pseudo_in_program(program: ASMProgram) -> ASMProgram {
    let n_function = fix_pseudo_in_function(program.function_definition);
    ASMProgram { function_definition: n_function }
}

fn fix_pseudo_in_function(func: ASMFunctionDefinition) -> ASMFunctionDefinition {
    TEMP_COUNTER.store(0, std::sync::atomic::Ordering::Relaxed);
    let mut new_instructions = vec![];
    for instruction in func.instructions {
        let n_instruction = fix_pseudo_in_instruction(instruction);
        new_instructions.push(n_instruction);
    }
    new_instructions.insert(0, ASMInstruction::AllocateStack(TEMP_COUNTER.load(Ordering::Relaxed)));
    ASMFunctionDefinition { name: func.name, instructions: new_instructions }
}

fn fix_pseudo_in_instruction(instruction: ASMInstruction) -> ASMInstruction {
    match instruction {
        ASMInstruction::Mov { src, dst } => {
            ASMInstruction::Mov { src: fix_pseudo_in_operands(src), dst: fix_pseudo_in_operands(dst) }
        }
        ASMInstruction::Unary { unop, operand } => {
            ASMInstruction::Unary { unop, operand: fix_pseudo_in_operands(operand) }
        }
        ASMInstruction::AllocateStack(size) => {
            ASMInstruction::AllocateStack(size)
        }
        ASMInstruction::Ret => {
            ASMInstruction::Ret
        }
    }
}

fn fix_pseudo_in_operands(operand: ASMOperand) -> ASMOperand {
    match operand {
        ASMOperand::Imm(value) => ASMOperand::Imm(value),
        ASMOperand::Reg(reg) => ASMOperand::Reg(reg),
        ASMOperand::Pseudo(identifier) => {
            let mut offsets = STACK_OFFSETS.lock().unwrap();

            if let Some(offset) = offsets.get(&identifier) {
                return ASMOperand::Stack(*offset);
            } else {
                TEMP_COUNTER.fetch_add(4, Ordering::Relaxed);
                let offset = TEMP_COUNTER.load(Ordering::Relaxed);
                offsets.insert(identifier.clone(), offset);
                return ASMOperand::Stack(offset);
            }
        },
        ASMOperand::Stack(offset) => ASMOperand::Stack(offset),
    }
}