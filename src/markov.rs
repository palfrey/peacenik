use common;
use nom::{digit, is_alphabetic, multispace};
use std::{io, str};

#[derive(Debug,PartialEq,Eq)]
pub enum Token {
    Junk,
    Comma,
    FullStop,
    QuestionMark,
    OpenBracket,
    CloseBracket,
    Colon,
    Quote,
    Word(String),
}

fn alpha_or_word_chars(c: u8) -> bool {
    is_alphabetic(c) || c == '\'' as u8 || c == '-' as u8
}

named!(word_match<&[u8], (&[u8], &[u8])>,
    tuple!(
        take_while1!(is_alphabetic),
        take_while!(alpha_or_word_chars)
    )
);

named!(get_token<Token>,
    alt!(
        tag!(".") => {|_| Token::FullStop} |
        tag!(",") => {|_| Token::Comma} |
        tag!(":") => {|_| Token::Colon} |
        tag!("?") => {|_| Token::QuestionMark} |
        tag!("(") => {|_| Token::OpenBracket} |
        tag!(")") => {|_| Token::CloseBracket} |
        is_a!("\"“”") => {|_| Token::Quote} |
        alt!(multispace | digit | is_a!("|&;")) => { |_| Token::Junk } |
        word_match => { |(begin, rest)| {
                let mut word = String::from(str::from_utf8(begin).expect("bad utf-8"));
                word += str::from_utf8(rest).expect("bad utf-8");
                Token::Word(word)
            }
        }
    )
);

pub fn get_tokens(filename: &str) -> Result<Vec<Token>, io::Error> {
    let empty_filter = |x: Token| Some(x);
    return common::get_words_core(filename, get_token, empty_filter);
}
