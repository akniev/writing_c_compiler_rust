use crate::lexer_tokens::{LexerToken, LexerTokenKind};
use crate::parser_tokens::{ASTExpressionToken, ASTFunctionDefinitionToken, ASTProgramToken, ASTStatementToken};

/*
<program> ::= <function>
<function> ::= "int" <identifier> "(" "void" ")" "{" <statement> "}"
<statement> ::= "return" <exp> ";"
<exp> ::= <int>
<identifier> ::= ? An identifier token ?
<int> ::= ? A constant token ?
*/

struct Parser {
    tokens: Vec<LexerToken>,
    cursor: usize,
}

impl Parser {
    fn take(&mut self) -> Option<LexerToken> {
        let token = self.tokens.get(self.cursor)?.clone();
        self.cursor += 1;
        Some(token)
    }

    fn expect(&mut self, expected: LexerTokenKind) -> Result<LexerToken, String> {
        let actual = self.take().ok_or("unexpected end of input")?;
        if actual.kind() == expected {
            Ok(actual)
        } else {
            Err(format!("expected {expected:?}, found {actual:?}"))
        }
    }

    // fn peek(&mut self) -> Option<LexerToken> {
    //     self.tokens.get(self.cursor).cloned()
    // }

    fn parse_function(&mut self) -> Result<ASTFunctionDefinitionToken, String> {
        self.expect(LexerTokenKind::Int)?;
        let name_token = self.expect(LexerTokenKind::Identifier)?;
        let name = match name_token {
            LexerToken::Identifier(name) => name,
            _ => unreachable!("expected identifier, found {name_token:?}"),
        };
        self.expect(LexerTokenKind::OpenParen)?;
        self.expect(LexerTokenKind::Void)?;
        self.expect(LexerTokenKind::CloseParen)?;
        self.expect(LexerTokenKind::OpenBrace)?;
        let statement = self.parse_statement()?;
        self.expect(LexerTokenKind::CloseBrace)?;
        Ok(ASTFunctionDefinitionToken { name, body: statement })
    }

    fn parse_statement(&mut self) -> Result<ASTStatementToken, String> {
        self.expect(LexerTokenKind::Return)?;
        let exp = self.parse_expression()?;
        self.expect(LexerTokenKind::Semicolon)?;
        Ok(ASTStatementToken::Return(exp))
    }

    fn parse_expression(&mut self) -> Result<ASTExpressionToken, String> {
        let const_token = self.expect(LexerTokenKind::Const)?;
        let value = match const_token {
            LexerToken::Const(value) => value,
            _ => unreachable!("expected constant, found {const_token:?}"),
        };
        Ok(ASTExpressionToken::Constant(value))
    }
}

pub fn parse(tokens: Vec<LexerToken>) -> Result<ASTProgramToken, String> {
    let mut parser = Parser { tokens, cursor: 0 };
    let function_definition = parser.parse_function()?;
    if parser.cursor != parser.tokens.len() {
        return Err("unexpected tokens".to_string());
    }
    Ok(ASTProgramToken { function_definition })
}
