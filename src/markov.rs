use common;
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
    SingleQuote,
    Paragraph,
    Newline,
    Word(String),
}

fn is_alphabetic(c: char) -> bool {
    c.is_alphabetic()
}

fn alpha_or_word_chars(c: char) -> bool {
    c.is_alphabetic() || c == '\'' || c == '-' || c == '\u{2014}' // em-dash
}

named!(word_match<&str, (&str, &str)>,
    tuple!(
        take_while1!(is_alphabetic),
        take_while!(alpha_or_word_chars)
    )
);

named!(get_token<&str, Token>,
    alt!(
        tag_s!(".") => {|_| Token::FullStop} |
        tag_s!(",") => {|_| Token::Comma} |
        tag_s!(":") => {|_| Token::Colon} |
        tag_s!("?") => {|_| Token::QuestionMark} |
        tag_s!("(") => {|_| Token::OpenBracket} |
        tag_s!(")") => {|_| Token::CloseBracket} |
        is_a_s!("\"“”") => {|_| Token::Quote} |
        is_a_s!("\r\n") => {|_| Token::Newline} |
        is_a_s!("\'") => {|_| Token::SingleQuote} |
        is_a_s!(" \t|&;") => { |_| Token::Junk } |
        word_match => { |(begin, rest)| {
                let mut word = String::from(begin);
                word += rest;
                Token::Word(word)
            }
        }
    )
);

fn empty_filter(x: Token) -> Option<Token> {
    Some(x)
}

pub fn get_tokens(filename: &str) -> Result<Vec<Token>, io::Error> {
    return common::get_words_core_fn(filename, get_token, empty_filter);
}

#[test]
fn copes_with_utf_8() {
    common::get_words_core("why—I".as_bytes(), get_token, empty_filter).unwrap();
}
