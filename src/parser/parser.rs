use crate::lexer::lexer_tokens::{LexerToken, LexerTokenKind};
use crate::parser::ast_tokens::{ASTBinaryOperator, ASTBlockItem, ASTDeclaration, ASTExpression, ASTFunctionDefinition, ASTPostfixOperator, ASTProgram, ASTStatement, ASTUnaryOperator};

/*
<program> ::= <function>
<function> ::= "int" <identifier> "(" "void" ")" "{" { <block-item> } "}"
<block-item> ::= <statement> | <declaration>
<declaration> ::= "int" <identifier> [ "=" <exp> ] ";"
<statement> ::= "return" <exp> ";" | <exp> ";" | ";"
<exp> ::= <factor> | <exp> <binop> <exp>
<factor> ::= <int> | <identifier> | <unop> <factor> | "(" <exp> ")" | <factor> <postfix>
<unop> ::= "-" | "~" | "++" | "--"
<postfix> ::= ++ | --
<binop> ::= "-" | "+" | "*" | "%" | "&" | "|" | "^" | "<<" | ">>" | "&&" | "||" | "==" | "!=" | "<" | "<=" | ">" | ">=" | = | += | -= | *= | /= | %=
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

        let mut function_body: Vec<ASTBlockItem> = vec![];

        while self.peek() != Some(LexerToken::CloseBrace) {
            let block_item = self.parse_block_item()?;
            function_body.push(block_item);
        }

        //let statement = self.parse_statement()?;
        self.expect(LexerTokenKind::CloseBrace)?;
        Ok(ASTFunctionDefinition { name, body: function_body })
    }

    fn parse_block_item(&mut self) -> Result<ASTBlockItem, String> {
        let next_token = self.peek().ok_or("unexpected end of input")?;
        match next_token {
            LexerToken::Int => { // Block item is a declaration
                let declaration = self.parse_declaration();
                Ok(ASTBlockItem::Declaration(declaration?))
            }
            _ => { // Block item is a statement
                let statement = self.parse_statement();
                Ok(ASTBlockItem::Statement(statement?))
            }
        }
    }

    fn parse_declaration(&mut self) -> Result<ASTDeclaration, String> {
        let Ok(LexerToken::Int) =
            self.expect(LexerTokenKind::Int)
        else {
            return Err("expected identifier".to_string());
        };

        let Ok(LexerToken::Identifier(var_name)) =
            self.expect(LexerTokenKind::Identifier)
        else {
            return Err("expected identifier".to_string());
        };

        let next_token = self.peek().ok_or("unexpected end of input")?;

        if matches!(&next_token, LexerToken::Semicolon) {
            self.expect(LexerTokenKind::Semicolon)?;
            let declaration = ASTDeclaration::VariableDeclaration { name: var_name, init: None };
            return Ok(declaration);
        }

        self.expect(LexerTokenKind::EqualSign)?;

        let exp = self.parse_expression(0)?;
        self.expect(LexerTokenKind::Semicolon)?;
        let declaration = ASTDeclaration::VariableDeclaration { name: var_name, init: Some(exp) };

        Ok(declaration)
    }

    fn parse_statement(&mut self) -> Result<ASTStatement, String> {
        let next_token = self.peek().ok_or("unexpected end of input")?;
        match next_token {
            LexerToken::Return => {
                self.expect(LexerTokenKind::Return)?;
                let exp = self.parse_expression(0)?;
                self.expect(LexerTokenKind::Semicolon)?;
                Ok(ASTStatement::Return(exp))
            }
            LexerToken::Semicolon => {
                self.take();
                Ok(ASTStatement::Null)
            }
            _ => {
                let exp = self.parse_expression(0)?;
                self.expect(LexerTokenKind::Semicolon)?;
                Ok(ASTStatement::Expression(exp))
            }
        }

    }

    fn parse_expression(&mut self, min_prec: i32) -> Result<ASTExpression, String> {
        let mut lhs = self.parse_factor()?;
        let mut next_token = self.peek();
        while Self::is_binary_op(&next_token) && Self::precedence(&next_token) >= min_prec {
            match next_token {
                Some(LexerToken::EqualSign) => { // Assignment
                    self.take();
                    let rhs = self.parse_expression(Self::precedence(&next_token))?;
                    lhs = ASTExpression::Assignment { lhs: Box::new(lhs), rhs: Box::new(rhs) };
                }
                _ => {
                    let operator = self.parse_binop()?;
                    let rhs = self.parse_expression(Self::precedence(&next_token) + 1)?;
                    lhs = ASTExpression::Binary { op: operator, lhs: Box::new(lhs), rhs: Box::new(rhs) };
                }
            }

            next_token = self.peek();
        }

        Ok(lhs)
    }

    fn parse_factor(&mut self) -> Result<ASTExpression, String> {
        let next_token = self.peek().ok_or("unexpected end of input")?;
        let mut result_opt: Option<ASTExpression> = None;
        match next_token {
            LexerToken::Const(_) => {
                let const_token = self.expect(LexerTokenKind::Const)?;
                let value = match const_token {
                    LexerToken::Const(value) => value,
                    _ => unreachable!("expected constant, found {const_token:?}"),
                };
                result_opt = Some(ASTExpression::Constant(value));
                // Ok(ASTExpression::Constant(value))
            }
            LexerToken::Identifier(name) => {
                self.take();
                result_opt = Some(ASTExpression::Var(name.to_string()));
                // Ok(ASTExpression::Var(name.to_string()))
            }
            LexerToken::Tilde | LexerToken::Hyphen | LexerToken::ExclamationMark | LexerToken::Increment | LexerToken::Decrement => {
                let operator = self.parse_unop()?;
                let inner_exp = self.parse_factor()?;
                result_opt = Some(ASTExpression::Unary { op: operator, exp: Box::new(inner_exp) });
                // Ok(ASTExpression::Unary { op: operator, exp: Box::new(inner_exp) })
            }
            LexerToken::OpenParen => {
                self.take();
                let inner_exp = self.parse_expression(0)?;
                self.expect(LexerTokenKind::CloseParen)?;
                result_opt = Some(inner_exp);
                // Ok(inner_exp)
            }
            _ => {
                panic!("unexpected token");
                //Err("unexpected token".to_string())
            }
        }

        if self.peek() == Some(LexerToken::Increment) || self.peek() == Some(LexerToken::Decrement) {
            let postfix_operator = self.parse_postfix()?;
            result_opt = Some(ASTExpression::Postfix { exp: Box::new(result_opt.unwrap()), op: postfix_operator });
        }

        let result = result_opt.unwrap();

        Ok(result)
    }

    fn precedence(token: &Option<LexerToken>) -> i32 {
        let token = token.as_ref().unwrap();
        match token {
            LexerToken::Asterisk | LexerToken::ForwardSlash | LexerToken::Percent => 50, // *, /, %
            LexerToken::Plus | LexerToken::Hyphen => 45, // +, -
            LexerToken::LeftShift | LexerToken::RightShift => 40, // <<, >>
            LexerToken::LessThan
            | LexerToken::LessThanOrEqual
            | LexerToken::GreaterThan
            | LexerToken::GreaterThanOrEqual => 35, // <, >, <=, >=
            LexerToken::TwoEqualSigns | LexerToken::ExclamationMarkEqualSign => 30, // ==, !=
            LexerToken::Ampersand => 25, // &
            LexerToken::Caret => 20, // ^
            LexerToken::Pipe => 15, // |
            LexerToken::TwoAmpersands => 10, // &&
            LexerToken::TwoPipes => 5, // ||
            LexerToken::EqualSign => 1,
            _ => {
                panic!("unexpected token");
            },
        }
    }

    fn is_binary_op(token: &Option<LexerToken>) -> bool {
        let token = token.as_ref().unwrap();
        match token {
            LexerToken::EqualSign
            | LexerToken::Plus
            | LexerToken::Hyphen
            | LexerToken::Asterisk
            | LexerToken::ForwardSlash
            | LexerToken::Percent
            | LexerToken::Ampersand
            | LexerToken::Pipe
            | LexerToken::Caret
            | LexerToken::LeftShift
            | LexerToken::RightShift
            | LexerToken::TwoAmpersands
            | LexerToken::TwoPipes
            | LexerToken::TwoEqualSigns
            | LexerToken::ExclamationMarkEqualSign
            | LexerToken::LessThan
            | LexerToken::GreaterThan
            | LexerToken::LessThanOrEqual
            | LexerToken::GreaterThanOrEqual => true,
            _ => false,
        }
    }

    fn parse_unop(&mut self) -> Result<ASTUnaryOperator, String> {
        let next_token_opt = self.take();
        let next_token = next_token_opt.as_ref().unwrap();
        match next_token {
            LexerToken::Tilde => Ok(ASTUnaryOperator::Complement),
            LexerToken::Hyphen => Ok(ASTUnaryOperator::Negate),
            LexerToken::ExclamationMark => Ok(ASTUnaryOperator::Not),
            LexerToken::Increment => Ok(ASTUnaryOperator::Increment),
            LexerToken::Decrement => Ok(ASTUnaryOperator::Decrement),
            _ => Err("unexpected token".to_string()),
        }
    }

    fn parse_postfix(&mut self) -> Result<ASTPostfixOperator, String> {
        match self.take().unwrap() {
            LexerToken::Increment => Ok(ASTPostfixOperator::Increment),
            LexerToken::Decrement => Ok(ASTPostfixOperator::Decrement),
            _ => {
                panic!("unexpected token");
            }
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
            LexerToken::Ampersand => Ok(ASTBinaryOperator::BitwiseAnd),
            LexerToken::Pipe => Ok(ASTBinaryOperator::BitwiseOr),
            LexerToken::Caret => Ok(ASTBinaryOperator::BitwiseXor),
            LexerToken::LeftShift => Ok(ASTBinaryOperator::ShiftLeft),
            LexerToken::RightShift => Ok(ASTBinaryOperator::ShiftRight),
            LexerToken::TwoAmpersands => Ok(ASTBinaryOperator::And),
            LexerToken::TwoPipes => Ok(ASTBinaryOperator::Or),
            LexerToken::TwoEqualSigns => Ok(ASTBinaryOperator::Equal),
            LexerToken::ExclamationMarkEqualSign => Ok(ASTBinaryOperator::NotEqual),
            LexerToken::LessThan => Ok(ASTBinaryOperator::LessThan),
            LexerToken::GreaterThan => Ok(ASTBinaryOperator::GreaterThan),
            LexerToken::LessThanOrEqual => Ok(ASTBinaryOperator::LessThanOrEqual),
            LexerToken::GreaterThanOrEqual => Ok(ASTBinaryOperator::GreaterThanOrEqual),
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
