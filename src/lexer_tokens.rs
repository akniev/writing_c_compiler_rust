use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum LexerToken {
    Identifier(String),
    Const(i32),
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
pub enum LexerTokenKind {
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

impl LexerToken {
    pub fn kind(&self) -> LexerTokenKind {
        match self {
            LexerToken::Identifier(_) => LexerTokenKind::Identifier,
            LexerToken::Const(_) => LexerTokenKind::Const,
            LexerToken::Int => LexerTokenKind::Int,
            LexerToken::Void => LexerTokenKind::Void,
            LexerToken::Return => LexerTokenKind::Return,
            LexerToken::OpenParen => LexerTokenKind::OpenParen,
            LexerToken::CloseParen => LexerTokenKind::CloseParen,
            LexerToken::OpenBrace => LexerTokenKind::OpenBrace,
            LexerToken::CloseBrace => LexerTokenKind::CloseBrace,
            LexerToken::Semicolon => LexerTokenKind::Semicolon,
            LexerToken::Comment => LexerTokenKind::Comment,
            LexerToken::CompilerDirective => LexerTokenKind::CompilerDirective,
        }
    }
}

impl LexerTokenKind {
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

    pub fn token(&self, text: &str) -> LexerToken {
        match self {
            LexerTokenKind::Identifier => LexerToken::Identifier(text.to_string()),
            LexerTokenKind::Const => LexerToken::Const(text.parse().unwrap()),
            LexerTokenKind::Int => LexerToken::Int,
            LexerTokenKind::Void => LexerToken::Void,
            LexerTokenKind::Return => LexerToken::Return,
            LexerTokenKind::OpenParen => LexerToken::OpenParen,
            LexerTokenKind::CloseParen => LexerToken::CloseParen,
            LexerTokenKind::OpenBrace => LexerToken::OpenBrace,
            LexerTokenKind::CloseBrace => LexerToken::CloseBrace,
            LexerTokenKind::Semicolon => LexerToken::Semicolon,
            LexerTokenKind::Comment => LexerToken::Comment,
            LexerTokenKind::CompilerDirective => LexerToken::CompilerDirective,
        }
    }
}

pub const LEXER_TOKEN_KINDS: &[LexerTokenKind] = &[
    LexerTokenKind::Identifier,
    LexerTokenKind::Const,
    LexerTokenKind::Int,
    LexerTokenKind::Void,
    LexerTokenKind::Return,
    LexerTokenKind::OpenParen,
    LexerTokenKind::CloseParen,
    LexerTokenKind::OpenBrace,
    LexerTokenKind::CloseBrace,
    LexerTokenKind::Semicolon,
    LexerTokenKind::Comment,
    LexerTokenKind::CompilerDirective,
];

