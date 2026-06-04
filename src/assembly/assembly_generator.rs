use crate::assembly::assembly_tokens::{ASMFunctionDefinition, ASMInstruction, ASMOperand, ASMProgram, ASMRegister, ASMUnaryOperator};
use crate::tacky::tacky_tokens::{TInstruction, TProgram, TUnaryOperator, TValue};

pub fn generate_assembly(program: TProgram) -> ASMProgram {
    let function_definition = ASMFunctionDefinition {
        name: program.function_definition.name,
        instructions: asm_instructions(program.function_definition.body)
    };
    ASMProgram { function_definition }
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
                instructions.append(vec![
                    ASMInstruction::Mov { src: src.to_operand(), dst: dst.to_operand() },
                    ASMInstruction::Unary { unop: op.to_operator(), operand: dst.to_operand() },
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
        }
    }
}