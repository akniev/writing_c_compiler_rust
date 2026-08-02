
/*
program = Program(function_definition)
function_definition = Function(identifier name, block_item* body)
block_item = S(statement) | D(declaration)
statement = Return(exp) | Expression(exp) | Null
declaration = Declaration(identifier name, exp? init)
exp = Constant(int)
    | Var(identifier)
    | Unary(unary_operator, exp)
    | Binary(binary_operator, exp, exp)
    | Assignment(exp, exp)
unary_operator = Complement | Negate | Not
binary_operator = Add | Subtract | Multiply | Divide | Remainder | BitwiseAnd | BitwiseOr | BitwiseXor | ShiftLeft | ShiftRight | And | Or | Equal | NotEqual | LessThan | GreaterThan | LessThanOrEqual | GreaterThanOrEqual
 */

#[derive(Debug, PartialEq)]
pub struct ASTProgram {
    pub function_definition: ASTFunctionDefinition,
}

#[derive(Debug, PartialEq)]
pub struct ASTFunctionDefinition {
    pub name: String,
    pub body: Vec<ASTBlockItem>
}

#[derive(Debug, PartialEq)]
pub enum ASTBlockItem {
    Statement(ASTStatement),
    Declaration(ASTDeclaration),
}

#[derive(Debug, PartialEq)]
pub enum ASTStatement {
    Return(ASTExpression),
    Expression(ASTExpression),
    Null,
}

#[derive(Debug, PartialEq)]
pub enum ASTDeclaration {
    VariableDeclaration { name: String, init: Option<ASTExpression> }
}

#[derive(Debug, PartialEq)]
pub enum ASTExpression {
    Var(String),
    Constant(i32),
    Unary { op: ASTUnaryOperator, exp: Box<ASTExpression> },
    Binary { op: ASTBinaryOperator, lhs: Box<ASTExpression>, rhs: Box<ASTExpression> },
    Assignment { lhs: Box<ASTExpression>, rhs: Box<ASTExpression> },
}

#[derive(Debug, PartialEq)]
pub enum ASTUnaryOperator {
    Complement,
    Negate,
    Not,
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
    And,
    Or,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}