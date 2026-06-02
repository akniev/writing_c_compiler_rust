use crate::token::{Token, TokenKind, TOKEN_KINDS};

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while cursor < input.len() {
        let remaining = &input[cursor..];
        let whitespace_len = remaining
            .find(|c: char| !c.is_whitespace())
            .unwrap_or(remaining.len());
        if whitespace_len > 0 {
            cursor += whitespace_len;
            continue;
        }

        let mut best_match: Option<(TokenKind, usize)> = None;
        for token_kind in TOKEN_KINDS {
            let pattern = token_kind.pattern();
            if let Some(found) = pattern.find(remaining) {
                if found.start() == 0 {
                    let length = found.end();
                    if best_match.as_ref().is_none_or(|(_, best_length)| length > *best_length) {
                        best_match = Some((*token_kind, length));
                    }
                }
            }
        }

        if let Some((token_kind, length)) = best_match {
            let text = &input[cursor..cursor + length];
            tokens.push(token_kind.token(text));
            cursor += length;
        } else {
            panic!("Invalid token");
        }
    }

    // Remove comments and compiler directives
    tokens.retain(|token| !matches!(token, Token::Comment | Token::CompilerDirective));

    return tokens;
}