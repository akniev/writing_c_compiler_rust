use crate::assembly::assembly_tokens::{ASMFunctionDefinition, ASMInstruction, ASMProgram};

pub fn generate_code(asm: ASMProgram) -> String {
    let mut result = String::new();

    print_function(asm.function_definition, &mut result);

    result.push_str(".section .note.GNU-stack,\"\",@progbits");

    return result;
}

fn print_function(function: ASMFunctionDefinition, result: &mut String) {
    result.push_str(&format!("  .globl {}\n", function.name));
    result.push_str(&format!("{}:\n", function.name));
    result.push_str("  pushq %rbp\n");
    result.push_str("  movq %rsp, %rbp\n");
    for inst in function.instructions {
        print_instruction(inst, result);
    }
}

fn print_instruction(inst: ASMInstruction, result: &mut String) {
    match inst {
        ASMInstruction::Mov { src, dst } => {
            result.push_str(&format!("  movl {}, {}\n", src.to_string(), dst.to_string()));
        }
        ASMInstruction::Unary { unop, operand } => {
            result.push_str(&format!("  {} {}\n", unop.to_string(), operand.to_string()));
        }
        ASMInstruction::AllocateStack(size) => {
            result.push_str(&format!("  subq ${}, %rsp\n", size));
        }
        ASMInstruction::Ret => {
            result.push_str("  movq %rbp, %rsp\n");
            result.push_str("  popq %rbp\n");
            result.push_str("  ret\n");
        }
    }
}