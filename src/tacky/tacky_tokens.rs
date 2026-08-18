/*
program = Program(top_level*)
top_level = Function(identifier, bool global, identifier* params, instruction* body)
          | StaticVariable(identifier, bool global, int init)
instruction = Return(val)
            | Unary(unary_operator, val src, val dst)
            | Binary(binary_operator, val src1, val src2, val dst)
            | Copy(val src, val dst)
            | Jump(identifier target)
            | JumpIfZero(val condition, identifier target)
            | JumpIfNonZero(val condition, identifier target)
            | Label(identifier)
            | FunCall(identifier fun_name, val* args, val dst)
val = Constant(int) | Var(identifier)
unary_operator = Complement | Negate
binary_operator = Add | Subtract | Multiply | Divide | Remainder | BitwiseAnd | BitwiseOr | BitwiseXor | ShiftLeft | ShiftRight | Equal | NotEqual | LessThan | GreaterThan | LessThanOrEqual | GreaterThanOrEqual
 */

#[derive(Debug, Clone, PartialEq)]
pub struct TProgram {
    pub top_level: Vec<TTopLevel>
}

#[derive(Debug, Clone, PartialEq)]
pub enum TTopLevel {
    Function(TFunction),
    StaticVariable(TStaticVariable),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TFunction {
    pub name: String,
    pub global: bool,
    pub params: Vec<String>,
    pub body: Vec<TInstruction>
}

#[derive(Debug, Clone, PartialEq)]
pub struct TStaticVariable {
    pub name: String,
    pub global: bool,
    pub init: i32
}

#[derive(Debug, Clone, PartialEq)]
pub enum TInstruction {
    Return(TValue),
    Unary { op: TUnaryOperator, src: TValue, dst: TValue },
    Binary { op: TBinaryOperator, src1: TValue, src2: TValue, dst: TValue },
    Copy { src: TValue, dst: TValue },
    Jump { label: String },
    JumpIfZero { val: TValue, label: String },
    JumpIfNonZero { val: TValue, label: String },
    Label { label: String },
    FunCall { fun_name: String, args: Vec<TValue>, dst: TValue },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TValue {
    Constant(i32),
    Var(String)
}

#[derive(Debug, Clone, PartialEq)]
pub enum TUnaryOperator {
    Complement,
    Negate,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TBinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}