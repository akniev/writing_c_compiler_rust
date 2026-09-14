
/*
program = Program(declaration*)
block_item = S(statement) | D(declaration)
block = Block(block_item*)
declaration = FunDecl(function_declaration) | VarDecl(variable_declaration)
variable_declaration = (identifier name, typed_exp? init,
                        type var_type, storage_class?)
function_declaration = (identifier name, identifier* params, block? body,
                        type fun_type, storage_class?)
type = Int | Long | FunType(type* params, type ret)
storage_class = Static | Extern
for_init = InitDecl(variable_declaration) | InitExp(typed_exp?)
statement = Return(typed_exp)
          | Expression(typed_exp)
          | If(typed_exp condition, statement then, statement? else)
          | Compound(block)
          | Break(identifier label)
          | Continue(identifier label)
          | While(typed_exp condition, statement body, identifier label)
          | DoWhile(statement body, typed_exp condition, identifier label)
          | For(for_init init, typed_exp? condition, typed_exp? post, statement body, identifier label)
          | Switch(typed_exp condition, statement body, identifier label)
          | Case(typed_exp value, statement body, identifier label)
          | Default(statement body, identifier label)
          | LabelledStatement(identifier label, statement)
          | Goto(identifier label)
          | Null
typed_exp = TypedExp(type, exp?)
exp = Constant(const)
    | Var(identifier)
    | Cast(type target_type, typed_exp)
    | Unary(unary_operator, typed_exp)
    | Binary(binary_operator, typed_exp, typed_exp)
    | Assignment(typed_exp, typed_exp)
    | CompoundAssignment(typed_exp, typed_exp, binary_operator)
    | Postfix(postfix_operator, typed_exp)
    | Conditional(typed_exp condition, typed_exp, typed_exp)
    | FunctionCall(identifier, typed_exp* args)
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
    pub init: Option<ASTTypedExpression>,
    pub _type: ASTType,
    pub storage_class: Option<ASTStorageClass>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTType {
    Int,
    Long,
    FunType(Vec<Box<ASTType>>, Box<ASTType>),
}

impl ASTType {
    pub fn is_function_type(&self) -> bool {
        match self {
            ASTType::FunType(_, _) => true,
            _ => false,
        }
    }

    pub fn fun_return_type(&self) -> ASTType {
        match self {
            ASTType::FunType(_, ret) => ret.as_ref().clone(),
            _ => panic!("Not a function type!"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTStorageClass {
    Static,
    Extern,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTForInit {
    InitDecl(ASTVariableDeclaration),
    InitExp(Option<ASTTypedExpression>),
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
    Return(ASTTypedExpression),
    Expression(ASTTypedExpression),
    If { condition: ASTTypedExpression, then_statement: Box<ASTStatement>, else_statement: Option<Box<ASTStatement>> },
    Compound(ASTBlock),
    Break { label: String },
    Continue { label: String },
    While { condition: ASTTypedExpression, body: Box<ASTStatement>, label: String },
    DoWhile { body: Box<ASTStatement>, condition: ASTTypedExpression, label: String },
    For {
        init: ASTForInit,
        condition: Option<ASTTypedExpression>,
        post: Option<ASTTypedExpression>,
        body: Box<ASTStatement>,
        label: String
    },
    Switch { condition: ASTTypedExpression, body: Box<ASTStatement>, label: String },
    Case { value: ASTTypedExpression, body: Box<ASTStatement>, label: String },
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
pub struct ASTTypedExpression {
    pub exp: ASTExpression,
    pub exp_type: Option<ASTType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTExpression {
    Constant(ASTConstant),
    Var(String),
    Cast { target_type: ASTType, exp: Box<ASTTypedExpression> },
    Unary { op: ASTUnaryOperator, exp: Box<ASTTypedExpression> },
    Postfix { op: ASTPostfixOperator, exp: Box<ASTTypedExpression> },
    Binary { op: ASTBinaryOperator, e1: Box<ASTTypedExpression>, e2: Box<ASTTypedExpression> },
    Assignment { lhs: Box<ASTTypedExpression>, rhs: Box<ASTTypedExpression> },
    CompoundAssignment { lhs: Box<ASTTypedExpression>, rhs: Box<ASTTypedExpression>, operator: ASTBinaryOperator },
    Conditional { condition: Box<ASTTypedExpression>, if_true: Box<ASTTypedExpression>, if_false: Box<ASTTypedExpression> },
    FunctionCall { name: String, args: Vec<ASTTypedExpression> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTConstant {
    Int(i64),
    Long(i64),
}

impl ASTConstant {
    pub fn typed_constant(i: i64, t: ASTType) -> ASTConstant {
        match t {
            ASTType::Int => ASTConstant::Int(i),
            ASTType::Long => ASTConstant::Long(i),
            ASTType::FunType(_, _) => panic!("Function type is not supported!"),
        }
    }
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