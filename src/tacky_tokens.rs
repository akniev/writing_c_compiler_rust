/*
program = Program(function_definition)
function_definition = Function(identifier, instruction* body)
instruction = Return(val) | Unary(unary_operator, val src, val dst)
val = Constant(int) | Var(identifier)
unary_operator = Complement | Negate
 */

#[derive(Debug, PartialEq)]
pub struct TProgram {
    pub function_definition: TFunctionDefinition
}

#[derive(Debug, PartialEq)]
pub struct TFunctionDefinition {
    pub name: String,
    pub body: Vec<TInstruction>
}

#[derive(Debug, PartialEq)]
pub enum TInstruction {
    Return(TValue),
    Unary { op: TUnaryOperator, src: TValue, dst: TValue }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TValue {
    Constant(i32),
    Var(String)
}

#[derive(Debug, PartialEq)]
pub enum TUnaryOperator {
    Complement,
    Negate
}