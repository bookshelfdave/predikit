// Copyright (c) 2025 Dave Parfitt

//use logos::{Logos, SpannedIter};

//use crate::predikit::comp::tokens::*;

// https://lalrpop.github.io/lalrpop/lexer_tutorial/005_external_lib.html

// pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

// pub struct Lexer<'input> {
//     // instead of an iterator over characters, we have a token iterator
//     token_stream: SpannedIter<'input, Token>,
// }

// impl<'input> Lexer<'input> {
//     pub fn new(input: &'input str) -> Self {
//         // the Token::lexer() method is provided by the Logos trait
//         Self {
//             token_stream: Token::lexer(input).spanned(),
//         }
//     }
// }

// impl Iterator for Lexer<'_> {
//     type Item = Spanned<Token, usize, LexicalError>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.token_stream
//             .next()
//             .map(|(token, span)| Ok((span.start, token?, span.end)))
//     }
// }
