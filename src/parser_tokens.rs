
/*
program = Program(function_definition)
function_definition = Function(identifier name, statement body)
statement = Return(exp)
exp = Constant(int)
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
    Constant(i32)
}
