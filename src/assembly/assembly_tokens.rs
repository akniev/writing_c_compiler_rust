/*
 program = Program(top_level*)
 assembly_type = Longword | Quadword
 top_level = Function(identifier name, bool global, instruction* instructions)
           | StaticVariable(identifier name, bool global, int alignment, int init)
 instruction = Mov(assembly_type, operand src, operand dst)
             | Movsx(operand src, operand dst)
             | Unary(unary_operator, assembly_type, operand)
             | Binary(binary_operator, assembly_type, operand, operand)
             | Cmp(assembly_type, operand, operand)
             | Idiv(assembly_type, operand)
             | Cdq(assembly_type)
             | Jmp(identifier)
             | JmpCC(cond_code, identifier)
             | SetCC(cond_code, operand)
             | Label(identifier)
             | Push(operand)
             | Call(identifier)
             | Ret
 unary_operator = Neg | Not
 binary_operator = Add | Sub | Mult
 operand = Imm(int) | Reg(reg) | Pseudo(identifier) | Stack(int) | Data(identifier)
 cond_code = E | NE | G | GE | L | LE
 reg = AX | CX | DX | DI | SI | R8 | R9 | R10 | R11 | SP
 */
use crate::parser::ast_tokens::ASTType;
use crate::parser::typechecker::StaticInit;

#[derive(Debug, PartialEq)]
pub struct ASMProgram {
    pub top_level: Vec<ASMTopLevel>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ASMType {
    Longword,
    Quadword,
}

impl StaticInit {
    pub fn asm_type(&self) -> ASMType {
        match self {
            StaticInit::IntInit(_) => ASMType::Longword,
            StaticInit::LongInit(_) => ASMType::Quadword,
        }
    }
}

impl ASTType {
    pub fn asm_type(&self) -> ASMType {
        match self {
            ASTType::Int => ASMType::Longword,
            ASTType::Long => ASMType::Quadword,
            ASTType::FunType(_, _) => panic!("Function type is not supported!"),
        }
    }
}

impl ASMType {
    pub fn alignment(&self) -> i32 {
        match self {
            ASMType::Longword => 4,
            ASMType::Quadword => 8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ASMTopLevel {
    Function { name: String, global: bool, instructions: Vec<ASMInstruction> },
    StaticVariable { name: String, global: bool, alignment: i32, init: StaticInit },
}

// #[derive(Debug, PartialEq)]
// pub struct ASMFunctionDefinition {
//     pub name: String,
//     pub instructions: Vec<ASMInstruction>,
// }

#[derive(Debug, Clone, PartialEq)]
pub enum ASMInstruction {
    Mov { _type: ASMType, src: ASMOperand, dst: ASMOperand },
    Movsx { src: ASMOperand, dst: ASMOperand },
    Unary { unop: ASMUnaryOperator, _type: ASMType, operand: ASMOperand },
    Binary { binop: ASMBinaryOperator, _type: ASMType, operand1: ASMOperand, operand2: ASMOperand },
    Cmp { _type: ASMType, operand1: ASMOperand, operand2: ASMOperand },
    Idiv { _type: ASMType, op: ASMOperand },
    Cdq { _type: ASMType },
    Jmp { label: String },
    JmpCC { cond_code: ASMCondCode, label: String },
    SetCC { cond_code: ASMCondCode, operand: ASMOperand },
    Label(String),
    Push(ASMOperand),
    Call(String),
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
    Imm(i64),
    Reg(ASMRegister),
    Pseudo(String),
    Stack(i64),
    Data(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ASMRegister {
    AX,
    CX,
    DX,
    DI,
    SI,
    R8,
    R9,
    R10,
    R11,
    SP,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASMRegSize {
    OneByte,
    FourBytes,
    EightBytes,
}

impl ASMOperand {
    pub fn to_string_4b(&self) -> String {
        self.to_string(ASMRegSize::FourBytes)
    }

    pub fn to_string_8b(&self) -> String {
        self.to_string(ASMRegSize::EightBytes)
    }

    pub fn to_string(&self, size: ASMRegSize) -> String {
        match self {
            ASMOperand::Imm(value) => format!("${}", value),
            ASMOperand::Reg(reg) => reg.to_string(size),
            ASMOperand::Stack(offset) => format!("{}(%rbp)", offset),
            ASMOperand::Data(identifier) => format!("{}(%rip)", identifier),
            ASMOperand::Pseudo(_) => {
                panic!("Pseudo- and data- operands shouldn't be used here");
            },
        }
    }
}

impl ASMRegister {
    pub fn to_string(&self, size: ASMRegSize) -> String {
        match size {
            ASMRegSize::OneByte => self.to_string1b(),
            ASMRegSize::FourBytes => self.to_string4b(),
            ASMRegSize::EightBytes => self.to_string8b(),
        }
    }
    fn to_string1b(&self) -> String {
        match self {
            ASMRegister::AX => "%al".to_string(),
            ASMRegister::CX => "%cl".to_string(),
            ASMRegister::DX => "%dl".to_string(),
            ASMRegister::DI => "%dil".to_string(),
            ASMRegister::SI => "%sil".to_string(),
            ASMRegister::R8 => "%r8b".to_string(),
            ASMRegister::R9 => "%r9b".to_string(),
            ASMRegister::R10 => "%r10b".to_string(),
            ASMRegister::R11 => "%r11b".to_string(),
            ASMRegister::SP => panic!("SP is not a 1-byte register"),
        }
    }

    fn to_string4b(&self) -> String {
        match self {
            ASMRegister::AX => "%eax".to_string(),
            ASMRegister::CX => "%ecx".to_string(),
            ASMRegister::DX => "%edx".to_string(),
            ASMRegister::DI => "%edi".to_string(),
            ASMRegister::SI => "%esi".to_string(),
            ASMRegister::R8 => "%r8d".to_string(),
            ASMRegister::R9 => "%r9d".to_string(),
            ASMRegister::R10 => "%r10d".to_string(),
            ASMRegister::R11 => "%r11d".to_string(),
            ASMRegister::SP => panic!("SP is not a 4-byte register"),
        }
    }

    fn to_string8b(&self) -> String {
        match self {
            ASMRegister::AX => "%rax".to_string(),
            ASMRegister::CX => "%rcx".to_string(),
            ASMRegister::DX => "%rdx".to_string(),
            ASMRegister::DI => "%rdi".to_string(),
            ASMRegister::SI => "%rsi".to_string(),
            ASMRegister::R8 => "%r8".to_string(),
            ASMRegister::R9 => "%r9".to_string(),
            ASMRegister::R10 => "%r10".to_string(),
            ASMRegister::R11 => "%r11".to_string(),
            ASMRegister::SP => "%rsp".to_string(),
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

impl ASMInstruction {
    pub fn allocate_stack(bytes: i64) -> ASMInstruction {
        ASMInstruction::Binary {
            binop: ASMBinaryOperator::Sub,
            _type: ASMType::Quadword,
            operand1: ASMOperand::Imm(bytes),
            operand2: ASMOperand::Reg(ASMRegister::SP),
        }
    }

    pub fn deallocate_stack(bytes: i64) -> ASMInstruction {
        ASMInstruction::Binary {
            binop: ASMBinaryOperator::Add,
            _type: ASMType::Quadword,
            operand1: ASMOperand::Reg(ASMRegister::SP),
            operand2: ASMOperand::Imm(bytes),
        }
    }
}