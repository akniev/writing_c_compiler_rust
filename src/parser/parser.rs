use crate::lexer::lexer_tokens::{LexerToken, LexerTokenKind};
use crate::parser::ast_tokens::{ASTBinaryOperator, ASTBlock, ASTBlockItem, ASTDeclaration, ASTExpression, ASTFunctionDefinition, ASTPostfixOperator, ASTProgram, ASTStatement, ASTUnaryOperator, ForInit};
use crate::parser::ast_tokens::ASTExpression::Conditional;
/*
<program> ::= <function>
<function> ::= "int" <identifier> "(" "void" ")" <block>
<block> ::= "{" { <block-item> } "}"
<block-item> ::= <statement> | <declaration>
<declaration> ::= "int" <identifier> [ "=" <exp> ] ";"
<for-init> ::= <declaration> | [ <exp> ] ";"
<statement> ::= "return" <exp> ";"
              | <exp> ";"
              | "if" "(" <exp> ")" <statement> [ "else" <statement> ]
              | ";"
              | <identifier> ":" <statement>
              | <block>
              | "break" ";"
              | "continue" ";"
              | "while" "(" <exp> ")" <statement>
              | "do" <statement> "while" "(" <exp> ")" ";"
              | for "(" <for-init> [ <exp> ] ";" [ <exp> ] ")" <statement>
              | "goto" <identifier> ";"
<exp> ::= <factor> | <exp> <binop> <exp> | <exp> "?" <exp> ":" <exp>
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

    fn peek_n(&mut self, n: usize) -> Option<LexerToken> {
        self.tokens.get(self.cursor + n).cloned()
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

        let mut block_items: Vec<ASTBlockItem> = vec![];

        while self.peek() != Some(LexerToken::CloseBrace) {
            let block_item = self.parse_block_item()?;
            block_items.push(block_item);
        }

        //let statement = self.parse_statement()?;
        self.expect(LexerTokenKind::CloseBrace)?;
        Ok(ASTFunctionDefinition { name, body: ASTBlock { items: block_items } })
    }

    fn parse_block(&mut self) -> Result<ASTBlock, String> {
        self.expect(LexerTokenKind::OpenBrace)?;
        let mut block_items: Vec<ASTBlockItem> = vec![];
        while self.peek() != Some(LexerToken::CloseBrace) {
            let block_item = self.parse_block_item();
            block_items.push(block_item?);
        }
        self.expect(LexerTokenKind::CloseBrace)?;
        Ok(ASTBlock { items: block_items })
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
        let next_next_token = self.peek_n(1);
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
            LexerToken::If => {
                self.take();
                self.expect(LexerTokenKind::OpenParen)?;
                let condition = self.parse_expression(0)?;
                self.expect(LexerTokenKind::CloseParen)?;
                let then_statement = self.parse_statement()?;
                let else_statement = if self.peek() == Some(LexerToken::Else) {
                    self.take();
                    Some(self.parse_statement()?)
                } else {
                    None
                };

                Ok(ASTStatement::If { condition, then_statement: Box::new(then_statement), else_statement: else_statement.map(Box::new) })
            }
            LexerToken::Break => {
                self.take();
                self.expect(LexerTokenKind::Semicolon)?;
                Ok(ASTStatement::Break { label: "-".to_string() })
            }
            LexerToken::Continue => {
                self.take();
                self.expect(LexerTokenKind::Semicolon)?;
                Ok(ASTStatement::Continue { label: "-".to_string() })
            }
            LexerToken::While => {
                self.take();
                self.expect(LexerTokenKind::OpenParen);
                let condition = self.parse_expression(0)?;
                self.expect(LexerTokenKind::CloseParen);
                let body = self.parse_statement()?;
                Ok(ASTStatement::While { condition, body: Box::new(body), label: "-".to_string() })
            }
            LexerToken::Do => {
                self.take();
                let body = self.parse_statement()?;
                self.expect(LexerTokenKind::While)?;
                self.expect(LexerTokenKind::OpenParen)?;
                let condition = self.parse_expression(0)?;
                self.expect(LexerTokenKind::CloseParen)?;
                self.expect(LexerTokenKind::Semicolon)?;
                Ok(ASTStatement::DoWhile { body: Box::new(body), condition, label: "-".to_string() })
            }
            LexerToken::For => {
                self.take();
                self.expect(LexerTokenKind::OpenParen)?;
                let for_init = self.parse_for_init();
                let cond = self.parse_optional_expression(0, LexerTokenKind::Semicolon);
                self.expect(LexerTokenKind::Semicolon)?;
                let post = self.parse_optional_expression(0, LexerTokenKind::CloseParen);
                self.expect(LexerTokenKind::CloseParen)?;
                let body = self.parse_statement()?;
                Ok(ASTStatement::For { init: for_init, condition: cond, post: post, body: Box::new(body), label: "-".to_string() })
            }
            LexerToken::OpenBrace => {
                let block = self.parse_block()?;
                Ok(ASTStatement::Compound(block))
            }
            LexerToken::Goto => {
                self.take();
                let label = self.expect(LexerTokenKind::Identifier)?;
                let label_name = match label {
                    LexerToken::Identifier(name) => name,
                    _ => unreachable!("expected identifier, found {label:?}"),
                };
                self.expect(LexerTokenKind::Semicolon)?;
                Ok(ASTStatement::Goto { label: label_name })
            }
            LexerToken::Identifier(label_name) if next_next_token == Some(LexerToken::Colon) => {
                self.take();
                self.take();
                let statement = self.parse_statement()?;
                Ok(ASTStatement::LabelledStatement { label: label_name, statement: Box::new(statement) })
            }
            _ => {
                let exp = self.parse_expression(0)?;
                self.expect(LexerTokenKind::Semicolon)?;
                Ok(ASTStatement::Expression(exp))
            }
        }

    }

    fn parse_for_init(&mut self) -> ForInit {
        let next_token = self.peek();
        if next_token != None && next_token.unwrap().kind().is_type_token() {
            let decl = self.parse_declaration().unwrap();
            ForInit::InitDecl(decl)
        } else {
            let exp = self.parse_optional_expression(0, LexerTokenKind::Semicolon);
            _ = self.expect(LexerTokenKind::Semicolon);
            ForInit::InitExp(exp)
        }
    }

    fn parse_optional_expression(&mut self, min_prec: i32, terminator: LexerTokenKind) -> Option<ASTExpression> {
        let next_token = self.peek();
        match next_token {
            Some(token) => {
                if token.kind() == terminator {
                    return None;
                }
            }
            None => {
                return None;
            }
        }

        self.parse_expression(min_prec).ok()
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
                Some(LexerToken::PlusEqual |
                     LexerToken::MinusEqual |
                     LexerToken::AsteriskEqual |
                     LexerToken::ForwardSlashEqual |
                     LexerToken::PercentEqual |
                     LexerToken::AmpersandEqual |
                     LexerToken::PipeEqual |
                     LexerToken::CaretEqual |
                     LexerToken::LeftShiftEqual |
                     LexerToken::RightShiftEqual) => {
                    let operator = self.parse_binop()?;
                    let rhs = self.parse_expression(Self::precedence(&next_token))?;
                    lhs = ASTExpression::CompoundAssignment { lhs: Box::new(lhs), rhs: Box::new(rhs), operator };
                }
                Some(LexerToken::QuestionMark) => {
                    let middle = self.parse_conditional_middle();
                    let rhs = self.parse_expression(Self::precedence(&next_token))?;
                    lhs = Conditional { condition: Box::new(lhs), if_true: Box::new(middle), if_false: Box::new(rhs) };
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
        let mut result: ASTExpression;
        match next_token {
            LexerToken::Const(_) => {
                let const_token = self.expect(LexerTokenKind::Const)?;
                let value = match const_token {
                    LexerToken::Const(value) => value,
                    _ => unreachable!("expected constant, found {const_token:?}"),
                };
                result = ASTExpression::Constant(value);
            }
            LexerToken::Identifier(name) => {
                self.take();
                result = ASTExpression::Var(name.to_string());
            }
            LexerToken::Tilde | LexerToken::Hyphen | LexerToken::ExclamationMark | LexerToken::Increment | LexerToken::Decrement => {
                let operator = self.parse_unop()?;
                let inner_exp = self.parse_factor()?;
                result = ASTExpression::Unary { op: operator, exp: Box::new(inner_exp) };
            }
            LexerToken::OpenParen => {
                self.take();
                let inner_exp = self.parse_expression(0)?;
                self.expect(LexerTokenKind::CloseParen)?;
                result = inner_exp;
            }
            _ => {
                panic!("unexpected token");
            }
        }

        if self.peek() == Some(LexerToken::Increment) || self.peek() == Some(LexerToken::Decrement) {
            let postfix_operator = self.parse_postfix()?;
            result = ASTExpression::Postfix { exp: Box::new(result.clone()), op: postfix_operator };
        }

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
            LexerToken::QuestionMark => 3, // ?
            LexerToken::EqualSign |
            LexerToken::PlusEqual |
            LexerToken::MinusEqual |
            LexerToken::AsteriskEqual |
            LexerToken::ForwardSlashEqual |
            LexerToken::PercentEqual |
            LexerToken::AmpersandEqual |
            LexerToken::PipeEqual |
            LexerToken::CaretEqual |
            LexerToken::LeftShiftEqual |
            LexerToken::RightShiftEqual => 1,
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
            | LexerToken::QuestionMark
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
            | LexerToken::GreaterThanOrEqual
            | LexerToken::PlusEqual
            | LexerToken::MinusEqual
            | LexerToken::AsteriskEqual
            | LexerToken::ForwardSlashEqual
            | LexerToken::PercentEqual
            | LexerToken::AmpersandEqual
            | LexerToken::PipeEqual
            | LexerToken::CaretEqual
            | LexerToken::LeftShiftEqual
            | LexerToken::RightShiftEqual => true,
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
            LexerToken::PlusEqual => Ok(ASTBinaryOperator::Add),
            LexerToken::MinusEqual => Ok(ASTBinaryOperator::Subtract),
            LexerToken::AsteriskEqual => Ok(ASTBinaryOperator::Multiply),
            LexerToken::ForwardSlashEqual => Ok(ASTBinaryOperator::Divide),
            LexerToken::PercentEqual => Ok(ASTBinaryOperator::Remainder),
            LexerToken::AmpersandEqual => Ok(ASTBinaryOperator::BitwiseAnd),
            LexerToken::PipeEqual => Ok(ASTBinaryOperator::BitwiseOr),
            LexerToken::CaretEqual => Ok(ASTBinaryOperator::BitwiseXor),
            LexerToken::LeftShiftEqual => Ok(ASTBinaryOperator::ShiftLeft),
            LexerToken::RightShiftEqual => Ok(ASTBinaryOperator::ShiftRight),
            _ => Err("unexpected token".to_string()),
        }
    }

    fn parse_conditional_middle(&mut self) -> ASTExpression {
        self.expect(LexerTokenKind::QuestionMark).unwrap();
        let middle = self.parse_expression(0).unwrap();
        self.expect(LexerTokenKind::Colon).unwrap();
        middle
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

impl LexerTokenKind {
    fn is_type_token(self) -> bool {
        match self {
            LexerTokenKind::Int => true,
            _ => false,
        }
    }
}