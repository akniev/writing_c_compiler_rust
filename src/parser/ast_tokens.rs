
/*
program = Program(declaration*)
block_item = S(statement) | D(declaration)
block = Block(block_item*)
declaration = FunDecl(function_declaration) | VarDecl(variable_declaration)
variable_declaration = (identifier name, exp? init,
                        type var_type, storage_class?)
function_declaration = (identifier name, identifier* params, block? body,
                        type fun_type, storage_class?)
type = Int | Long | FunType(type* params, type ret)
storage_class = Static | Extern
for_init = InitDecl(variable_declaration) | InitExp(exp?)
statement = Return(exp)
          | Expression(exp)
          | If(exp condition, statement then, statement? else)
          | Compound(block)
          | Break(identifier label)
          | Continue(identifier label)
          | While(exp condition, statement body, identifier label)
          | DoWhile(statement body, exp condition, identifier label)
          | For(for_init init, exp? condition, exp? post, statement body, identifier label)
          | Switch(exp condition, statement body, identifier label)
          | Case(exp value, statement body, identifier label)
          | Default(statement body, identifier label)
          | LabelledStatement(identifier label, statement)
          | Goto(identifier label)
          | Null
exp = Constant(const)
    | Var(identifier)
    | Cast(type target_type, exp)
    | Unary(unary_operator, exp)
    | Binary(binary_operator, exp, exp)
    | Assignment(exp, exp)
    | CompoundAssignment(exp, exp, binary_operator)
    | Postfix(postfix_operator, exp)
    | Conditional(exp condition, exp, exp)
    | FunctionCall(identifier, exp* args)
unary_operator = Complement | Negate | Not
binary_operator = Add | Subtract | Multiply | Divide | Remainder
                | BitwiseAnd | BitwiseOr | BitwiseXor | ShiftLeft
                | ShiftRight | And | Or | Equal | NotEqual | LessThan
                | GreaterThan | LessThanOrEqual | GreaterThanOrEqual
const = ConstInt(int) | ConstLong(int)
 */

#[derive(Debug, Clone, PartialEq)]
pub struct ASTProgram {
    pub declarations: Vec<ASTDeclaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ASTFunctionDeclaration {
    pub name: String,
    pub params: Vec<String>,
    pub body: Option<ASTBlock>,
    pub _type: ASTType,
    pub storage_class: Option<ASTStorageClass>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ASTVariableDeclaration {
    pub name: String,
    pub init: Option<ASTExpression>,
    pub _type: ASTType,
    pub storage_class: Option<ASTStorageClass>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTType {
    Int,
    Long,
    FunType(Vec<Box<ASTType>>, Box<ASTType>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTStorageClass {
    Static,
    Extern,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTForInit {
    InitDecl(ASTVariableDeclaration),
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
        init: ASTForInit,
        condition: Option<ASTExpression>,
        post: Option<ASTExpression>,
        body: Box<ASTStatement>,
        label: String
    },
    Switch { condition: ASTExpression, body: Box<ASTStatement>, label: String },
    Case { value: ASTExpression, body: Box<ASTStatement>, label: String },
    Default { body: Box<ASTStatement>, label: String },
    LabelledStatement { label: String, statement: Box<ASTStatement> },
    Goto { label: String },
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTDeclaration {
    VariableDeclaration(ASTVariableDeclaration),
    FunctionDeclaration(ASTFunctionDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTExpression {
    Constant(ASTConstant),
    Var(String),
    Cast { target_type: ASTType, exp: Box<ASTExpression> },
    Unary { op: ASTUnaryOperator, exp: Box<ASTExpression> },
    Postfix { op: ASTPostfixOperator, exp: Box<ASTExpression> },
    Binary { op: ASTBinaryOperator, lhs: Box<ASTExpression>, rhs: Box<ASTExpression> },
    Assignment { lhs: Box<ASTExpression>, rhs: Box<ASTExpression> },
    CompoundAssignment { lhs: Box<ASTExpression>, rhs: Box<ASTExpression>, operator: ASTBinaryOperator },
    Conditional { condition: Box<ASTExpression>, if_true: Box<ASTExpression>, if_false: Box<ASTExpression> },
    FunctionCall { name: String, args: Vec<ASTExpression> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTConstant {
    Int(i64),
    Long(i64),
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