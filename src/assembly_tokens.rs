/*
 program = Program(function_definition)
 function_definition = Function(identifier name, instruction* instructions)
 instruction = Mov(operand src, operand dst)
             | Unary(unary_operator, operand)
             | AllocateStack(int)
             | Ret
 unary_operator = Neg | Not
 operand = Imm(int) | Reg(reg) | Pseudo(identifier) | Stack(int)
 reg = AX | R10
 */
use crate::tacky_tokens::TValue;

#[derive(Debug, PartialEq)]
pub struct ASMProgram {
    pub function_definition: ASMFunctionDefinition,
}

#[derive(Debug, PartialEq)]
pub struct ASMFunctionDefinition {
    pub name: String,
    pub instructions: Vec<ASMInstruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASMInstruction {
    Mov { src: ASMOperand, dst: ASMOperand },
    Unary { unop: ASMUnaryOperator, operand: ASMOperand },
    AllocateStack(i32),
    Ret,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASMUnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASMOperand {
    Imm(i32),
    Reg(ASMRegister),
    Pseudo(String),
    Stack(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASMRegister {
    AX,
    R10,
}

impl ASMOperand {
    // pub fn to_string(&self) -> String {
    //     match self {
    //         ASMOperand::Imm(value) => format!("${}", value),
    //         ASMOperand::Reg(reg) => reg.to_string(),
    //
    //     }
    // }


}

impl ASMRegister {
    // pub fn to_string(&self) -> String {
    //     match self {
    //         ASMRegister::AX => String::from("%eax"),
    //         ASMRegister::R10 => "R10".to_string(),
    //     }
    // }
}