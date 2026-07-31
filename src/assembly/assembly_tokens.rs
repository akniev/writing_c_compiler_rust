/*
 program = Program(function_definition)
 function_definition = Function(identifier name, instruction* instructions)
 instruction = Mov(operand src, operand dst)
             | Unary(unary_operator, operand)
             | Binary(binary_operator, operand, operand)
             | Cmp(operand, operand)
             | Idiv(operand)
             | Cdq
             | Jmp(identifier)
             | JmpCC(cond_code, identifier)
             | SetCC(cond_code, operand)
             | Label(identifier)
             | AllocateStack(int)
             | Ret
 unary_operator = Neg | Not
 binary_operator = Add | Sub | Mult
 operand = Imm(int) | Reg(reg) | Pseudo(identifier) | Stack(int)
 cond_code = E | NE | G | GE | L | LE
 reg = AX | DX | CX | R10 | R11
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
    Cmp { operand1: ASMOperand, operand2: ASMOperand },
    Idiv(ASMOperand),
    Jmp { label: String },
    JmpCC { cond_code: ASMCondCode, label: String },
    SetCC { cond_code: ASMCondCode, operand: ASMOperand },
    Label(String),
    Cdq,
    AllocateStack(i32),
    Ret,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASMCondCode {
    E,
    NE,
    G,
    GE,
    L,
    LE,
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
    And,
    Or,
    Xor,
    ShiftLeft,
    ShiftRight,
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
    CX,
    R10,
    R11,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASMRegSize {
    TwoBytes,
    FourBytes,
}

impl ASMOperand {
    pub fn to_string_4b(&self) -> String {
        self.to_string(ASMRegSize::FourBytes)
    }

    pub fn to_string(&self, size: ASMRegSize) -> String {
        match self {
            ASMOperand::Imm(value) => format!("${}", value),
            ASMOperand::Reg(reg) => reg.to_string(size),
            ASMOperand::Stack(offset) => format!("-{}(%rbp)", offset),
            ASMOperand::Pseudo(_) => {
                panic!("Pseudo-operands shouldn't be used here");
            },
        }
    }
}

impl ASMRegister {
    pub fn to_string(&self, size: ASMRegSize) -> String {
        match size {
            ASMRegSize::TwoBytes => self.to_string2b(),
            ASMRegSize::FourBytes => self.to_string4b(),
        }
    }
    fn to_string2b(&self) -> String {
        match self {
            ASMRegister::AX => "%al".to_string(),
            ASMRegister::DX => "%dl".to_string(),
            ASMRegister::CX => "%cl".to_string(),
            ASMRegister::R10 => "%r10b".to_string(),
            ASMRegister::R11 => "%r11b".to_string(),
        }
    }

    fn to_string4b(&self) -> String {
        match self {
            ASMRegister::AX => "%eax".to_string(),
            ASMRegister::DX => "%edx".to_string(),
            ASMRegister::CX => "%ecx".to_string(),
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
        match self {
            ASMBinaryOperator::Add => "addl".to_string(),
            ASMBinaryOperator::Sub => "subl".to_string(),
            ASMBinaryOperator::Mult => "imull".to_string(),
            ASMBinaryOperator::And => "andl".to_string(),
            ASMBinaryOperator::Or => "orl".to_string(),
            ASMBinaryOperator::Xor => "xorl".to_string(),
            ASMBinaryOperator::ShiftLeft => "sall".to_string(),
            ASMBinaryOperator::ShiftRight => "sarl".to_string(),
        }
    }
}