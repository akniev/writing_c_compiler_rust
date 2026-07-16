
/*
program = Program(function_definition)
function_definition = Function(identifier name, statement body)
statement = Return(exp)
exp = Constant(int)
    | Unary(unary_operator, exp)
    | Binary(binary_operator, exp, exp)
unary_operator = Complement | Negate
binary_operator = Add | Subtract | Multiply | Divide | Remainder
 */

#[derive(Debug, PartialEq)]
pub struct ASTProgram {
    pub function_definition: ASTFunctionDefinition,
}

#[derive(Debug, PartialEq)]
pub struct ASTFunctionDefinition {
    pub name: String,
    pub body: ASTStatement
}

#[derive(Debug, PartialEq)]
pub enum ASTStatement {
    Return(ASTExpression)
}

#[derive(Debug, PartialEq)]
pub enum ASTExpression {
    Constant(i32),
    Unary { op: ASTUnaryOperator, exp: Box<ASTExpression> },
    Binary { op: ASTBinaryOperator, lhs: Box<ASTExpression>, rhs: Box<ASTExpression> },
}

#[derive(Debug, PartialEq)]
pub enum ASTUnaryOperator {
    Complement,
    Negate,
}

#[derive(Debug, PartialEq)]
pub enum ASTBinaryOperator {
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
}