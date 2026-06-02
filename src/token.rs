use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Const(i64),
    Int,
    Void,
    Return,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,
    // We need them to be able to remove them from code because we don't need them yet
    Comment,
    CompilerDirective,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Identifier,
    Const,
    Int,
    Void,
    Return,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon,
    // We need them to be able to remove them from code because we don't need them yet
    Comment,
    CompilerDirective,
}

impl TokenKind {
    pub fn pattern(&self) -> Regex {
        Regex::new(match self {
            Self::Identifier => r"^[a-zA-Z_]\w*\b",
            Self::Const => r"^[0-9]+\b",
            Self::Int => r"^int\b",
            Self::Void => r"^void\b",
            Self::Return => r"^return\b",
            Self::OpenParen => r"^\(",
            Self::CloseParen => r"^\)",
            Self::OpenBrace => r"^\{",
            Self::CloseBrace => r"^\}",
            Self::Semicolon => r"^;",
            Self::Comment => r"^(?://[^\r\n]*|/\*[\s\S]*?\*/)",
            Self::CompilerDirective => r"(?m)^#[^\r\n]*",
        }).expect("Invalid regex")
    }

    pub fn token(&self, text: &str) -> Token {
        match self {
            TokenKind::Identifier => Token::Identifier(text.to_string()),
            TokenKind::Const => Token::Const(text.parse().unwrap()),
            TokenKind::Int => Token::Int,
            TokenKind::Void => Token::Void,
            TokenKind::Return => Token::Return,
            TokenKind::OpenParen => Token::OpenParen,
            TokenKind::CloseParen => Token::CloseParen,
            TokenKind::OpenBrace => Token::OpenBrace,
            TokenKind::CloseBrace => Token::CloseBrace,
            TokenKind::Semicolon => Token::Semicolon,
            TokenKind::Comment => Token::Comment,
            TokenKind::CompilerDirective => Token::CompilerDirective,
        }
    }
}

pub const TOKEN_KINDS: &[TokenKind] = &[
    TokenKind::Identifier,
    TokenKind::Const,
    TokenKind::Int,
    TokenKind::Void,
    TokenKind::Return,
    TokenKind::OpenParen,
    TokenKind::CloseParen,
    TokenKind::OpenBrace,
    TokenKind::CloseBrace,
    TokenKind::Semicolon,
    TokenKind::Comment,
    TokenKind::CompilerDirective,
];

