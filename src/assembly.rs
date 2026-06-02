use crate::assembly_tokens::{ASMFunctionDefinition, ASMInstruction, ASMOperand, ASMProgram};
use crate::lexer_tokens::LexerToken;
use crate::parser_tokens::{ASTExpression, ASTProgram, ASTStatement};

pub fn generate_assembly(program: ASTProgram) -> ASMProgram {
    let function_definition = ASMFunctionDefinition {
        name: program.function_definition.name,
        instructions: asm_instructions(program.function_definition.body)
    };
    ASMProgram { function_definition }
}

fn asm_instructions(body: ASTStatement) -> Vec<ASMInstruction> {
    match body {
        ASTStatement::Return(exp) => {
            let value = match exp {
                ASTExpression::Constant(value) => value,
                _ => unreachable!("expected constant, found {exp:?}"),
            };
            let src = ASMOperand::Imm(value);
            let dst = ASMOperand::Register;
            vec![ASMInstruction::Mov { src, dst }, ASMInstruction::Ret]
        },
    }
}
