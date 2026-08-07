
/*
program = Program(function_definition)
function_definition = Function(identifier name, block body)
block_item = S(statement) | D(declaration)
block = Block(block_item*)
declaration = Declaration(identifier name, exp? init)
for_init = InitDecl(declaration) | InitExp(exp?)
statement = Return(exp)
          | Expression(exp)
          | If(exp condition, statement then, statement? else)
          | Compound(block)
          | Break(identifier label)
          | Continue(identifier label)
          | While(exp condition, statement body, identifier label)
          | DoWhile(statement body, exp condition, identifier label)
          | For(for_init init, exp? condition, exp? post, statement body, identifier label)
          | LabelledStatement(identifier label, statement)
          | Goto(identifier label)
          | Null
exp = Constant(int)
    | Var(identifier)
    | Unary(unary_operator, exp)
    | Binary(binary_operator, exp, exp)
    | Assignment(exp, exp)
    | CompoundAssignment(exp, exp, binary_operator)
    | Postfix(postfix_operator, exp)
    | Conditional(exp condition, exp, exp)
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
    pub body: ASTBlock,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
    InitDecl(ASTDeclaration),
    InitExp(Option<ASTExpression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ASTBlock {
    pub items: Vec<ASTBlockItem>,
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
    If { condition: ASTExpression, then_statement: Box<ASTStatement>, else_statement: Option<Box<ASTStatement>> },
    Compound(ASTBlock),
    Break { label: String },
    Continue { label: String },
    While { condition: ASTExpression, body: Box<ASTStatement>, label: String },
    DoWhile { body: Box<ASTStatement>, condition: ASTExpression, label: String },
    For {
        init: ForInit,
        condition: Option<ASTExpression>,
        post: Option<ASTExpression>,
        body: Box<ASTStatement>,
        label: String
    },
    LabelledStatement { label: String, statement: Box<ASTStatement> },
    Goto { label: String },
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
    CompoundAssignment { lhs: Box<ASTExpression>, rhs: Box<ASTExpression>, operator: ASTBinaryOperator },
    Conditional { condition: Box<ASTExpression>, if_true: Box<ASTExpression>, if_false: Box<ASTExpression> }
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