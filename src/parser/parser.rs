use crate::lexer::lexer_tokens::{LexerToken, LexerTokenKind};
use crate::parser::ast_tokens::{ASTBinaryOperator, ASTExpression, ASTFunctionDefinition, ASTProgram, ASTStatement, ASTUnaryOperator};

/*
<program> ::= <function>
<function> ::= "int" <identifier> "(" "void" ")" "{" <statement> "}"
<statement> ::= "return" <exp> ";"
<exp> ::= <factor> | <exp> <binop> <exp>
<factor> ::= <int> | <unop> <factor> | "(" <exp> ")"
<unop> ::= "-" | "~"
<binop> ::= "-" | "+" | "*" | "%"
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
        let exp = self.parse_expression(0)?;
        self.expect(LexerTokenKind::Semicolon)?;
        Ok(ASTStatement::Return(exp))
    }

    fn parse_expression(&mut self, min_prec: i32) -> Result<ASTExpression, String> {
        let mut lhs = self.parse_factor()?;
        let mut next_token = self.peek();
        while Self::is_binary_op(&next_token) && Self::precedence(&next_token) >= min_prec {
            let operator = self.parse_binop()?;
            let rhs = self.parse_expression(Self::precedence(&next_token) + 1)?;
            lhs = ASTExpression::Binary { op: operator, lhs: Box::new(lhs), rhs: Box::new(rhs) };
            next_token = self.peek();
        }
        Ok(lhs)
    }

    fn parse_factor(&mut self) -> Result<ASTExpression, String> {
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
                let inner_exp = self.parse_factor()?;
                Ok(ASTExpression::Unary { op: operator, exp: Box::new(inner_exp) })
            }
            LexerToken::OpenParen => {
                self.take();
                let inner_exp = self.parse_expression(0)?;
                self.expect(LexerTokenKind::CloseParen)?;
                Ok(inner_exp)
            }
            _ => {
                Err("unexpected token".to_string())
            }
        }
    }

    fn precedence(token: &Option<LexerToken>) -> i32 {
        let token = token.as_ref().unwrap();
        match token {
            LexerToken::Plus | LexerToken::Hyphen => 45,
            LexerToken::Asterisk | LexerToken::ForwardSlash | LexerToken::Percent => 50,
            _ => 0,
        }
    }

    fn is_binary_op(token: &Option<LexerToken>) -> bool {
        let token = token.as_ref().unwrap();
        match token {
            LexerToken::Plus | LexerToken::Hyphen | LexerToken::Asterisk | LexerToken::ForwardSlash | LexerToken::Percent => true,
            _ => false,
        }
    }

    fn parse_unop(&mut self) -> Result<ASTUnaryOperator, String> {
        let next_token_opt = self.take();
        let next_token = next_token_opt.as_ref().unwrap();
        match next_token {
            LexerToken::Tilde => Ok(ASTUnaryOperator::Complement),
            LexerToken::Hyphen => Ok(ASTUnaryOperator::Negate),
            _ => Err("unexpected token".to_string()),
        }
    }

    fn parse_binop(&mut self) -> Result<ASTBinaryOperator, String> {
        let next_token_opt = self.take();
        let next_token = next_token_opt.as_ref().unwrap();
        match next_token {
            LexerToken::Plus => Ok(ASTBinaryOperator::Add),
            LexerToken::Hyphen => Ok(ASTBinaryOperator::Subtract),
            LexerToken::Asterisk => Ok(ASTBinaryOperator::Multiply),
            LexerToken::ForwardSlash => Ok(ASTBinaryOperator::Divide),
            LexerToken::Percent => Ok(ASTBinaryOperator::Remainder),
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
