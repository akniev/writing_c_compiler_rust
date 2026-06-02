
/*
program = Program(function_definition)
function_definition = Function(identifier name, statement body)
statement = Return(exp)
exp = Constant(int)
*/

#[derive(Debug, PartialEq)]
pub struct ASTProgramToken {
    pub function_definition: ASTFunctionDefinitionToken,
}

#[derive(Debug, PartialEq)]
pub struct ASTFunctionDefinitionToken {
    pub name: String,
    pub body: ASTStatementToken
}

#[derive(Debug, PartialEq)]
pub enum ASTStatementToken {
    Return(ASTExpressionToken)
}

#[derive(Debug, PartialEq)]
pub enum ASTExpressionToken {
    Constant(i32)
}
