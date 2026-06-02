/*
 program = Program(function_definition)
 function_definition = Function(identifier name, instruction* instructions)
 instruction = Mov(operand src, operand dst) | Ret
 operand = Imm(int) | Register
 */

#[derive(Debug, PartialEq)]
pub struct ASMProgram {
    pub function_definition: ASMFunctionDefinition,
}

#[derive(Debug, PartialEq)]
pub struct ASMFunctionDefinition {
    pub name: String,
    pub instructions: Vec<ASMInstruction>,
}

#[derive(Debug, PartialEq)]
pub enum ASMInstruction {
    Mov { src: ASMOperand, dst: ASMOperand },
    Ret,
}

#[derive(Debug, PartialEq)]
pub enum ASMOperand {
    Imm(i32),
    Register,
}

impl ASMOperand {
    pub fn to_string(&self) -> String {
        match self {
            ASMOperand::Imm(value) => format!("${}", value),
            ASMOperand::Register => String::from("%eax"),
        }
    }
}