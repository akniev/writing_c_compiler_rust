
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

#[derive(Debug, Clone, PartialEq)]
pub struct ASTProgram {
    pub function_definition: ASTFunctionDefinition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ASTFunctionDefinition {
    pub name: String,
    pub body: Vec<ASTBlockItem>
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTBlockItem {
    Statement(ASTStatement),
    Declaration(ASTDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTStatement {
    Return(ASTExpression),
    Expression(ASTExpression),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTDeclaration {
    VariableDeclaration { name: String, init: Option<ASTExpression> }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTExpression {
    Var(String),
    Constant(i32),
    Unary { op: ASTUnaryOperator, exp: Box<ASTExpression> },
    Postfix { op: ASTPostfixOperator, exp: Box<ASTExpression> },
    Binary { op: ASTBinaryOperator, lhs: Box<ASTExpression>, rhs: Box<ASTExpression> },
    Assignment { lhs: Box<ASTExpression>, rhs: Box<ASTExpression> },
    CompoundAssignment { lhs: Box<ASTExpression>, rhs: Box<ASTExpression>, operator: ASTBinaryOperator }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTUnaryOperator {
    Complement,
    Negate,
    Not,
    Increment,
    Decrement,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTPostfixOperator {
    Increment,
    Decrement,
}

#[derive(Debug, Clone, PartialEq)]
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