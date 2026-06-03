use crate::lexer_tokens::{LexerToken, LexerTokenKind};
use crate::parser_tokens::{ASTExpression, ASTFunctionDefinition, ASTProgram, ASTStatement, ASTUnaryOperator};

/*
<program> ::= <function>
<function> ::= "int" <identifier> "(" "void" ")" "{" <statement> "}"
<statement> ::= "return" <exp> ";"
<exp> ::= <int> | <unop> <exp> | "(" <exp> ")"
<unop> ::= "-" | "~"
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

    fn peek(&mut self) -> Option<LexerToken> {
        self.tokens.get(self.cursor).cloned()
    }

    fn parse_function(&mut self) -> Result<ASTFunctionDefinition, String> {
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
        Ok(ASTFunctionDefinition { name, body: statement })
    }

    fn parse_statement(&mut self) -> Result<ASTStatement, String> {
        self.expect(LexerTokenKind::Return)?;
        let exp = self.parse_expression()?;
        self.expect(LexerTokenKind::Semicolon)?;
        Ok(ASTStatement::Return(exp))
    }

    fn parse_expression(&mut self) -> Result<ASTExpression, String> {
        let next_token = self.peek().ok_or("unexpected end of input")?;
        match next_token {
            LexerToken::Const(_) => {
                let const_token = self.expect(LexerTokenKind::Const)?;
                let value = match const_token {
                    LexerToken::Const(value) => value,
                    _ => unreachable!("expected constant, found {const_token:?}"),
                };
                Ok(ASTExpression::Constant(value))
            }
            LexerToken::Tilde | LexerToken::Hyphen => {
                let operator = self.parse_unop()?;
                let inner_exp = self.parse_expression()?;
                Ok(ASTExpression::Unary { op: operator, exp: Box::new(inner_exp) })
            }
            LexerToken::OpenParen => {
                self.expect(LexerTokenKind::OpenParen)?;
                let exp = self.parse_expression()?;
                self.expect(LexerTokenKind::CloseParen)?;
                Ok(exp)
            }
            _ => {
                Err("unexpected token".to_string())
            }
        }
    }

    fn parse_unop(&mut self) -> Result<ASTUnaryOperator, String> {
        let next_token = self.take();
        match next_token {
            Some(LexerToken::Tilde) => Ok(ASTUnaryOperator::Complement),
            Some(LexerToken::Hyphen) => Ok(ASTUnaryOperator::Negate),
            _ => Err("unexpected token".to_string()),
        }
    }
}

pub fn parse(tokens: Vec<LexerToken>) -> Result<ASTProgram, String> {
    let mut parser = Parser { tokens, cursor: 0 };
    let function_definition = parser.parse_function()?;
    if parser.cursor != parser.tokens.len() {
        return Err("unexpected tokens".to_string());
    }
    Ok(ASTProgram { function_definition })
}
