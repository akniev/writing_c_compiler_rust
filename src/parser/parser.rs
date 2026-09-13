use crate::lexer::lexer_tokens::{LexerToken, LexerTokenKind};
use crate::parser::ast_tokens::{ASTBinaryOperator, ASTBlock, ASTBlockItem, ASTDeclaration, ASTExpression, ASTFunctionDeclaration, ASTPostfixOperator, ASTProgram, ASTStatement, ASTUnaryOperator, ASTForInit, ASTVariableDeclaration, ASTStorageClass, ASTType, ASTConstant, ASTTypedExpression};
use crate::parser::ast_tokens::ASTExpression::Conditional;
/*
<program> ::= { <declaration> }
<declaration> ::= <variable-declaration> | <function-declaration>
<variable-declaration> ::= { <specifier> }+ <identifier> [ "=" <exp> ] ";"
<function-declaration> ::= { <specifier> }+ <identifier> "(" <param-list> ")" ( <block> | ";" )
<param-list> ::= "void"
               | {<type-specifier>}+ <identifier> { "," {<type-specifier>}+ <identifier> }
<type-specifier> ::= "int" | "long"
<specifier> ::= <type-specifier> | "static" | "extern"
<function> ::= "int" <identifier> "(" "void" ")" <block>
<block> ::= "{" { <block-item> } "}"
<block-item> ::= <statement> | <declaration>
<for-init> ::= <variable-declaration> | [ <exp> ] ";"
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
              | "switch" "(" <exp> ")" <statement>
              | "case" <exp> ":" <statement>
              | "default" ":" <statement>
              | "goto" <identifier> ";"
<exp> ::= <factor>
        | <exp> <binop> <exp>
        | <exp> "?" <exp> ":" <exp>
        | <identifier> "(" [ <argument-list> ] ")"
<argument-list> ::= <exp> { "," <exp> }
<factor> ::= <const> | <identifier>
           | "(" {<type-specifier>}+ ")" <factor>
           | <unop> <factor> | "(" <exp> ")"
           | <identifier> "(" [<argument-list>] ")"
<unop> ::= "-" | "~" | "++" | "--"
<postfix> ::= ++ | --
<binop> ::= "-" | "+" | "*" | "%" | "&" | "|" | "^" | "<<" | ">>" | "&&" | "||" | "==" | "!=" | "<" | "<=" | ">" | ">=" | = | += | -= | *= | /= | %=
<const> ::= <int> | <long>
<identifier> ::= ? An identifier token ?
<int> ::= ? An int token ?
<long> ::= ? A int or long token ?
 */

struct Parser {
    tokens: Vec<LexerToken>,
    cursor: usize,
}

impl Parser {
    fn parse_program(&mut self) -> ASTProgram {
        let mut declarations: Vec<ASTDeclaration> = vec![];
        while self.cursor < self.tokens.len() {
            let declaration = self.parse_declaration();
            declarations.push(declaration);
        }
        ASTProgram { declarations }
    }

    fn parse_declaration(&mut self) -> ASTDeclaration {
        let (decl_type, storage_class) = self.parse_specifiers();
        let LexerToken::Identifier(name) = self.expect(LexerTokenKind::Identifier) else {
            panic!("expected identifier");
        };

        match self.peek() {
            Some(LexerToken::OpenParen) => {
                let function_declaration = self.parse_function_declaration(storage_class, decl_type, name.clone());
                ASTDeclaration::FunctionDeclaration(function_declaration)
            }
            _ => {
                let variable_declaration = self.parse_variable_declaration(storage_class, decl_type, name.clone());
                ASTDeclaration::VariableDeclaration(variable_declaration)
            }
        }
    }

    fn parse_specifiers(&mut self) -> (ASTType, Option<ASTStorageClass>) {
        let mut type_specifiers = vec![];
        let mut storage_specifiers = vec![];
        let allowed_specifiers = vec![
            LexerTokenKind::Int,
            LexerTokenKind::Long,
            LexerTokenKind::Static,
            LexerTokenKind::Extern,
        ];

        while let Some(token) = self.peek() {
            if !allowed_specifiers.contains(&token.kind()) {
                break;
            }

            match token {
                LexerToken::Int | LexerToken::Long => {
                    self.take();
                    type_specifiers.push(token);
                }
                LexerToken::Static | LexerToken::Extern => {
                    self.take();
                    storage_specifiers.push(token);
                }
                _ => unreachable!("Invalid specifier"),
            }
        }

        if type_specifiers.is_empty() || type_specifiers.len() > 2 {
            panic!("Invalid type specifier");
        }

        if storage_specifiers.len() > 1 {
            panic!("Invalid storage class");
        }

        let _type = match type_specifiers.as_slice() {
            [LexerToken::Int] => ASTType::Int,
            [LexerToken::Long] | [LexerToken::Int, LexerToken::Long] | [LexerToken::Long, LexerToken::Int] => ASTType::Long,
            _ => unreachable!("Invalid type specifier"),
        };

        let storage_class = if storage_specifiers.len() == 1 {
            Some(self.parse_storage_class(&storage_specifiers[0]))
        } else {
            None
        };

        (_type, storage_class)
    }

    fn parse_function_declaration(&mut self, storage_class: Option<ASTStorageClass>, return_type: ASTType, name: String) -> ASTFunctionDeclaration {
        self.expect(LexerTokenKind::OpenParen);

        let params: Vec<(ASTType, String)> = self.parse_function_params();

        self.expect(LexerTokenKind::CloseParen);

        let block = if self.peek() == Some(LexerToken::Semicolon) {
            self.take();
            None
        } else {
            Some(self.parse_block())
        };

        let param_names = params.iter().map(|p| p.1.clone()).collect();
        let param_types = params.iter().map(|p| Box::new(p.0.clone())).collect();

        let fun_type = ASTType::FunType(param_types, Box::new(return_type.clone()));

        ASTFunctionDeclaration { name, params: param_names, body: block, _type: fun_type, storage_class }
    }

    fn parse_variable_declaration(&mut self, storage_class: Option<ASTStorageClass>, var_type: ASTType, name: String) -> ASTVariableDeclaration {
        let next_token = self.peek().unwrap();

        if matches!(&next_token, LexerToken::Semicolon) {
            self.expect(LexerTokenKind::Semicolon);
            let declaration = ASTVariableDeclaration { name, init: None, _type: var_type, storage_class };
            return declaration;
        }

        self.expect(LexerTokenKind::EqualSign);

        let exp = self.parse_expression(0);
        self.expect(LexerTokenKind::Semicolon);
        let declaration = ASTVariableDeclaration { name, init: Some(exp), _type: var_type, storage_class };

        declaration
    }

    fn parse_block(&mut self) -> ASTBlock {
        self.expect(LexerTokenKind::OpenBrace);
        let mut block_items: Vec<ASTBlockItem> = vec![];
        while self.peek() != Some(LexerToken::CloseBrace) {
            let block_item = self.parse_block_item();
            block_items.push(block_item);
        }
        self.expect(LexerTokenKind::CloseBrace);
        ASTBlock { items: block_items }
    }

    fn parse_block_item(&mut self) -> ASTBlockItem {
        let next_token = self.peek().unwrap();
        match next_token {
            LexerToken::Int | LexerToken::Long | LexerToken::Static | LexerToken::Extern => { // Block item is a declaration
                let declaration = self.parse_declaration();
                ASTBlockItem::Declaration(declaration)
            }
            _ => { // Block item is a statement
                let statement = self.parse_statement();
                ASTBlockItem::Statement(statement)
            }
        }
    }

    fn parse_storage_class(&mut self, token: &LexerToken) -> ASTStorageClass {
        match token {
            LexerToken::Static => ASTStorageClass::Static,
            LexerToken::Extern => ASTStorageClass::Extern,
            _ => unreachable!("Invalid storage class"),
        }
    }

    fn parse_function_params(&mut self) -> Vec<(ASTType, String)> {
        let mut params: Vec<(ASTType, String)> = vec![];
        if self.peek() == Some(LexerToken::Void) {
            self.take();
            return params;
        }
        let p1_type = self.parse_type();
        let LexerToken::Identifier(p1) = self.expect(LexerTokenKind::Identifier) else {
            panic!("expected identifier");
        };
        params.push((p1_type, p1));
        while self.peek() != Some(LexerToken::CloseParen) {
            self.expect(LexerTokenKind::Comma);
            let pn_type = self.parse_type();
            let LexerToken::Identifier(pn) = self.expect(LexerTokenKind::Identifier) else {
                panic!("expected identifier");
            };
            params.push((pn_type, pn));
        }
        params
    }

    fn parse_type(&mut self) -> ASTType {
        let allowed_specifiers = vec![
            LexerTokenKind::Int,
            LexerTokenKind::Long,
            LexerTokenKind::Static,
            LexerTokenKind::Extern,
        ];

        let mut type_specifiers = vec![];

        while let Some(token) = self.peek() && allowed_specifiers.contains(&token.kind()) {
            type_specifiers.push(token);
            self.take();
        }

        if type_specifiers.is_empty() || type_specifiers.len() > 2 {
            panic!("Invalid type specifier");
        }

        match type_specifiers.as_slice() {
            [LexerToken::Int] => ASTType::Int,
            [LexerToken::Long] | [LexerToken::Int, LexerToken::Long] | [LexerToken::Long, LexerToken::Int] => ASTType::Long,
            _ => unreachable!("Invalid type specifier"),
        }
    }

    fn parse_statement(&mut self) -> ASTStatement {
        let next_token = self.peek().unwrap();
        let next_next_token = self.peek_n(1);
        match next_token {
            LexerToken::Return => {
                self.expect(LexerTokenKind::Return);
                let exp = self.parse_expression(0);
                self.expect(LexerTokenKind::Semicolon);
                ASTStatement::Return(exp)
            }
            LexerToken::Semicolon => {
                self.take();
                ASTStatement::Null
            }
            LexerToken::If => {
                self.take();
                self.expect(LexerTokenKind::OpenParen);
                let condition = self.parse_expression(0);
                self.expect(LexerTokenKind::CloseParen);
                let then_statement = self.parse_statement();
                let else_statement = if self.peek() == Some(LexerToken::Else) {
                    self.take();
                    Some(self.parse_statement())
                } else {
                    None
                };

                ASTStatement::If { condition, then_statement: Box::new(then_statement), else_statement: else_statement.map(Box::new) }
            }
            LexerToken::Break => {
                self.take();
                self.expect(LexerTokenKind::Semicolon);
                ASTStatement::Break { label: "-".to_string() }
            }
            LexerToken::Continue => {
                self.take();
                self.expect(LexerTokenKind::Semicolon);
                ASTStatement::Continue { label: "-".to_string() }
            }
            LexerToken::While => {
                self.take();
                _ = self.expect(LexerTokenKind::OpenParen);
                let condition = self.parse_expression(0);
                _ = self.expect(LexerTokenKind::CloseParen);
                let body = self.parse_statement();
                ASTStatement::While { condition, body: Box::new(body), label: "-".to_string() }
            }
            LexerToken::Do => {
                self.take();
                let body = self.parse_statement();
                self.expect(LexerTokenKind::While);
                self.expect(LexerTokenKind::OpenParen);
                let condition = self.parse_expression(0);
                self.expect(LexerTokenKind::CloseParen);
                self.expect(LexerTokenKind::Semicolon);
                ASTStatement::DoWhile { body: Box::new(body), condition, label: "-".to_string() }
            }
            LexerToken::For => {
                self.take();
                self.expect(LexerTokenKind::OpenParen);
                let for_init = self.parse_for_init();
                let cond = self.parse_optional_expression(0, LexerTokenKind::Semicolon);
                self.expect(LexerTokenKind::Semicolon);
                let post = self.parse_optional_expression(0, LexerTokenKind::CloseParen);
                self.expect(LexerTokenKind::CloseParen);
                let body = self.parse_statement();
                ASTStatement::For { init: for_init, condition: cond, post, body: Box::new(body), label: "-".to_string() }
            }
            LexerToken::Switch => {
                self.take();
                self.expect(LexerTokenKind::OpenParen);
                let exp = self.parse_expression(0);
                self.expect(LexerTokenKind::CloseParen);
                let body = self.parse_statement();
                ASTStatement::Switch { condition: exp, body: Box::new(body), label: "-".to_string() }
            }
            LexerToken::Case => {
                self.take();
                let exp = self.parse_expression(0);
                self.expect(LexerTokenKind::Colon);
                let body = self.parse_statement();
                ASTStatement::Case { value: exp, body: Box::new(body), label: "-".to_string() }
            }
            LexerToken::Default => {
                self.take();
                self.expect(LexerTokenKind::Colon);
                let body = self.parse_statement();
                ASTStatement::Default { body: Box::new(body), label: "-".to_string() }
            }
            LexerToken::OpenBrace => {
                let block = self.parse_block();
                ASTStatement::Compound(block)
            }
            LexerToken::Goto => {
                self.take();
                let label = self.expect(LexerTokenKind::Identifier);
                let label_name = match label {
                    LexerToken::Identifier(name) => name,
                    _ => unreachable!("expected identifier, found {label:?}"),
                };
                self.expect(LexerTokenKind::Semicolon);
                ASTStatement::Goto { label: label_name }
            }
            LexerToken::Identifier(label_name) if next_next_token == Some(LexerToken::Colon) => {
                self.take();
                self.take();
                let statement = self.parse_statement();
                ASTStatement::LabelledStatement { label: label_name, statement: Box::new(statement) }
            }
            _ => {
                let exp = self.parse_expression(0);
                self.expect(LexerTokenKind::Semicolon);
                ASTStatement::Expression(exp)
            }
        }

    }

    fn parse_for_init(&mut self) -> ASTForInit {
        let next_token = self.peek();
        if let Some(next_token) = next_token && (next_token.kind().is_type_token() || next_token.kind().is_storage_class_token()) {
            let decl = self.parse_declaration();
            let ASTDeclaration::VariableDeclaration(var_decl) = decl else {
                panic!("expected declaration");
            };
            ASTForInit::InitDecl(var_decl)
        } else {
            let exp = self.parse_optional_expression(0, LexerTokenKind::Semicolon);
            _ = self.expect(LexerTokenKind::Semicolon);
            ASTForInit::InitExp(exp)
        }
    }

    fn parse_optional_expression(&mut self, min_prec: i32, terminator: LexerTokenKind) -> Option<ASTTypedExpression> {
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

        Some(self.parse_expression(min_prec))
    }

    fn parse_expression(&mut self, min_prec: i32) -> ASTTypedExpression {
        let mut lhs = self.parse_factor();
        let mut next_token = self.peek();
        while Self::is_binary_op(&next_token) && Self::precedence(&next_token) >= min_prec {
            match next_token {
                Some(LexerToken::EqualSign) => { // Assignment
                    self.take();
                    let rhs = self.parse_expression(Self::precedence(&next_token));
                    lhs = ASTExpression::Assignment {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs)
                    }.to_typed_without_type();
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
                    let operator = self.parse_binop();
                    let rhs = self.parse_expression(Self::precedence(&next_token));
                    lhs = ASTExpression::CompoundAssignment {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs), operator
                    }.to_typed_without_type();
                }
                Some(LexerToken::QuestionMark) => {
                    let middle = self.parse_conditional_middle();
                    let rhs = self.parse_expression(Self::precedence(&next_token));
                    lhs = Conditional {
                        condition: Box::new(lhs),
                        if_true: Box::new(middle),
                        if_false: Box::new(rhs)
                    }.to_typed_without_type();
                }
                _ => {
                    let operator = self.parse_binop();
                    let rhs = self.parse_expression(Self::precedence(&next_token) + 1);
                    lhs = ASTExpression::Binary {
                        op: operator,
                        e1: Box::new(lhs),
                        e2: Box::new(rhs)
                    }.to_typed_without_type();
                }
            }

            next_token = self.peek();
        }

        lhs
    }

    fn parse_factor(&mut self) -> ASTTypedExpression {
        let next_token = self.peek().unwrap();
        let mut result: ASTTypedExpression;
        match next_token {
            LexerToken::Const(_) => {
                let const_token = self.expect(LexerTokenKind::Const);
                let value = match const_token {
                    LexerToken::Const(value) => value,
                    _ => unreachable!("expected constant, found {const_token:?}"),
                };
                result = ASTExpression::Constant(ASTConstant::Int(value as i64)).to_typed_without_type();
            }
            LexerToken::LongConst(_) => {
                let const_token = self.expect(LexerTokenKind::LongConst);
                let value = match const_token {
                    LexerToken::LongConst(value) => value,
                    _ => unreachable!("expected constant, found {const_token:?}"),
                };
                result = ASTExpression::Constant(ASTConstant::Long(value)).to_typed_without_type();
            }
            LexerToken::Identifier(_) if self.peek_n(1) == Some(LexerToken::OpenParen) => {
                let function_call = self.parse_function_call();
                result = function_call;
            }
            LexerToken::Identifier(name) => {
                self.take();
                result = ASTExpression::Var(name.to_string()).to_typed_without_type();
            }
            LexerToken::Tilde | LexerToken::Hyphen | LexerToken::ExclamationMark | LexerToken::Increment | LexerToken::Decrement => {
                let operator = self.parse_unop();
                let inner_exp = self.parse_factor();
                result = ASTExpression::Unary { op: operator, exp: Box::new(inner_exp) }.to_typed_without_type();
            }
            LexerToken::OpenParen if self.peek_n(1).unwrap().kind().is_type_token() => {
                self.take();
                let _type = self.parse_type();
                self.expect(LexerTokenKind::CloseParen);
                let factor = self.parse_factor();
                result = ASTExpression::Cast { target_type: _type, exp: Box::new(factor) }.to_typed_without_type();
            }
            LexerToken::OpenParen => {
                self.take();
                let inner_exp = self.parse_expression(0);
                self.expect(LexerTokenKind::CloseParen);
                result = inner_exp;
            }
            _ => {
                panic!("unexpected token");
            }
        }

        if self.peek() == Some(LexerToken::Increment) || self.peek() == Some(LexerToken::Decrement) {
            let postfix_operator = self.parse_postfix();
            result = ASTExpression::Postfix { exp: Box::new(result.clone()), op: postfix_operator }.to_typed_without_type();
        }

        result
    }

    fn parse_function_call(&mut self) -> ASTTypedExpression {
        let LexerToken::Identifier(name) = self.expect(LexerTokenKind::Identifier) else {
            panic!("expected identifier");
        };

        self.expect(LexerTokenKind::OpenParen);

        let mut args: Vec<ASTTypedExpression> = Vec::new();

        if self.peek() != Some(LexerToken::CloseParen) {
            let arg1 = self.parse_expression(0);
            args.push(arg1);

            while self.peek() != Some(LexerToken::CloseParen) {
                self.expect(LexerTokenKind::Comma);
                let argn = self.parse_expression(0);
                args.push(argn);
            }
        }

        self.expect(LexerTokenKind::CloseParen);
        let result = ASTExpression::FunctionCall { name, args };
        let typed_result = ASTTypedExpression { exp: result, exp_type: None };
        typed_result
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

    fn parse_unop(&mut self) -> ASTUnaryOperator {
        let next_token_opt = self.take();
        let next_token = next_token_opt.as_ref().unwrap();
        match next_token {
            LexerToken::Tilde => ASTUnaryOperator::Complement,
            LexerToken::Hyphen => ASTUnaryOperator::Negate,
            LexerToken::ExclamationMark => ASTUnaryOperator::Not,
            LexerToken::Increment => ASTUnaryOperator::Increment,
            LexerToken::Decrement => ASTUnaryOperator::Decrement,
            _ => panic!("unexpected token"),
        }
    }

    fn parse_postfix(&mut self) -> ASTPostfixOperator {
        match self.take().unwrap() {
            LexerToken::Increment => ASTPostfixOperator::Increment,
            LexerToken::Decrement => ASTPostfixOperator::Decrement,
            _ => {
                panic!("unexpected token");
            }
        }
    }

    fn parse_binop(&mut self) -> ASTBinaryOperator {
        let next_token_opt = self.take();
        let next_token = next_token_opt.as_ref().unwrap();
        match next_token {
            LexerToken::Plus => ASTBinaryOperator::Add,
            LexerToken::Hyphen => ASTBinaryOperator::Subtract,
            LexerToken::Asterisk => ASTBinaryOperator::Multiply,
            LexerToken::ForwardSlash => ASTBinaryOperator::Divide,
            LexerToken::Percent => ASTBinaryOperator::Remainder,
            LexerToken::Ampersand => ASTBinaryOperator::BitwiseAnd,
            LexerToken::Pipe => ASTBinaryOperator::BitwiseOr,
            LexerToken::Caret => ASTBinaryOperator::BitwiseXor,
            LexerToken::LeftShift => ASTBinaryOperator::ShiftLeft,
            LexerToken::RightShift => ASTBinaryOperator::ShiftRight,
            LexerToken::TwoAmpersands => ASTBinaryOperator::And,
            LexerToken::TwoPipes => ASTBinaryOperator::Or,
            LexerToken::TwoEqualSigns => ASTBinaryOperator::Equal,
            LexerToken::ExclamationMarkEqualSign => ASTBinaryOperator::NotEqual,
            LexerToken::LessThan => ASTBinaryOperator::LessThan,
            LexerToken::GreaterThan => ASTBinaryOperator::GreaterThan,
            LexerToken::LessThanOrEqual => ASTBinaryOperator::LessThanOrEqual,
            LexerToken::GreaterThanOrEqual => ASTBinaryOperator::GreaterThanOrEqual,
            LexerToken::PlusEqual => ASTBinaryOperator::Add,
            LexerToken::MinusEqual => ASTBinaryOperator::Subtract,
            LexerToken::AsteriskEqual => ASTBinaryOperator::Multiply,
            LexerToken::ForwardSlashEqual => ASTBinaryOperator::Divide,
            LexerToken::PercentEqual => ASTBinaryOperator::Remainder,
            LexerToken::AmpersandEqual => ASTBinaryOperator::BitwiseAnd,
            LexerToken::PipeEqual => ASTBinaryOperator::BitwiseOr,
            LexerToken::CaretEqual => ASTBinaryOperator::BitwiseXor,
            LexerToken::LeftShiftEqual => ASTBinaryOperator::ShiftLeft,
            LexerToken::RightShiftEqual => ASTBinaryOperator::ShiftRight,
            _ => panic!("unexpected token"),
        }
    }

    fn parse_conditional_middle(&mut self) -> ASTTypedExpression {
        self.expect(LexerTokenKind::QuestionMark);
        let middle = self.parse_expression(0);
        self.expect(LexerTokenKind::Colon);
        middle
    }

    fn take(&mut self) -> Option<LexerToken> {
        let token = self.tokens.get(self.cursor)?.clone();
        self.cursor += 1;
        Some(token)
    }

    fn expect(&mut self, expected: LexerTokenKind) -> LexerToken {
        let actual = self.take().unwrap();
        if actual.kind() == expected {
            actual
        } else {
            panic!("expected {expected:?}, found {actual:?}")
        }
    }

    fn peek(&mut self) -> Option<LexerToken> {
        self.tokens.get(self.cursor).cloned()
    }

    fn peek_n(&mut self, n: usize) -> Option<LexerToken> {
        self.tokens.get(self.cursor + n).cloned()
    }
}

pub fn parse(tokens: Vec<LexerToken>) -> ASTProgram {
    let mut parser = Parser { tokens, cursor: 0 };
    let program = parser.parse_program();
    if parser.cursor != parser.tokens.len() {
        panic!("unexpected tokens");
    }
    program
}

impl LexerTokenKind {
    fn is_type_token(self) -> bool {
        match self {
            LexerTokenKind::Int | LexerTokenKind::Long => true,
            _ => false,
        }
    }

    fn is_storage_class_token(self) -> bool {
        match self {
            LexerTokenKind::Static | LexerTokenKind::Extern => true,
            _ => false,
        }
    }
}

impl ASTExpression {
    pub fn to_typed_without_type(self) -> ASTTypedExpression {
        ASTTypedExpression { exp: self, exp_type: None }
    }

    pub fn to_typed(&self, exp_type: Option<ASTType>) -> ASTTypedExpression {
        ASTTypedExpression {
            exp: self.clone(),
            exp_type,
        }
    }
}

impl ASTTypedExpression {
    pub fn exp(&self) -> &ASTExpression {
        &self.exp
    }

    pub fn exp_type(&self) -> Option<&ASTType> {
        self.exp_type.as_ref()
    }

    pub fn with_type(&self, exp_type: Option<ASTType>) -> ASTTypedExpression {
        ASTTypedExpression { exp: self.exp.clone(), exp_type }
    }
}