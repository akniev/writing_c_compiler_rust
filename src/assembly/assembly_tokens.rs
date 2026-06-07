/*
 program = Program(function_definition)
 function_definition = Function(identifier name, instruction* instructions)
 instruction = Mov(operand src, operand dst)
             | Unary(unary_operator, operand)
             | Binary(binary_operator, operand, operand)
             | Idiv(operand)
             | Cdq
             | AllocateStack(int)
             | Ret
 unary_operator = Neg | Not
 binary_operator = Add | Sub | Mult
 operand = Imm(int) | Reg(reg) | Pseudo(identifier) | Stack(int)
 reg = AX | DX | R10 | R11
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

#[derive(Debug, Clone, PartialEq)]
pub enum ASMInstruction {
    Mov { src: ASMOperand, dst: ASMOperand },
    Unary { unop: ASMUnaryOperator, operand: ASMOperand },
    Binary { binop: ASMBinaryOperator, operand1: ASMOperand, operand2: ASMOperand },
    Idiv(ASMOperand),
    Cdq,
    AllocateStack(i32),
    Ret,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASMUnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASMBinaryOperator {
    Add,
    Sub,
    Mult,
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
    DX,
    R10,
    R11,
}

impl ASMOperand {
    pub fn to_string(&self) -> String {
        match self {
            ASMOperand::Imm(value) => format!("${}", value),
            ASMOperand::Reg(reg) => reg.to_string(),
            ASMOperand::Stack(offset) => format!("-{}(%rbp)", offset),
            ASMOperand::Pseudo(_) => {
                panic!("Pseudo-operands shouldn't be used here");
            },
        }
    }
}

impl ASMRegister {
    pub fn to_string(&self) -> String {
        match self {
            ASMRegister::AX => String::from("%eax"),
            ASMRegister::DX => String::from("%edx"),
            ASMRegister::R10 => "%r10d".to_string(),
            ASMRegister::R11 => "%r11d".to_string(),
        }
    }
}

impl ASMUnaryOperator {
    pub fn to_string(&self) -> String {
        match self {
            ASMUnaryOperator::Neg => "negl".to_string(),
            ASMUnaryOperator::Not => "notl".to_string(),
        }
    }
}

impl ASMBinaryOperator {
    pub fn to_string(&self) -> String {
        todo!()
        // match self {
        //     ASMBinaryOperator::Add => "addl".to_string(),
        //     ASMBinaryOperator::Sub => "subl".to_string(),
        //     ASMBinaryOperator::Mult => "imull".to_string(),
        // }
    }
}