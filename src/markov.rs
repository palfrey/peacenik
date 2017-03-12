use common;
use runner;
use serde_yaml;
use std::{io, str};
use std::collections::BTreeMap;
use std::fs::File;

#[derive(Debug, PartialEq, Eq)]
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
    Newline,
    Word(String),
    Begin,
}

impl Token {
    pub fn string(self: Token) -> String {
        return match self {
                Token::Word(word) => word,
                Token::Junk => String::from(" "),
                Token::Comma => String::from(","),
                Token::FullStop => String::from("."),
                Token::QuestionMark => String::from("?"),
                Token::OpenBracket => String::from("("),
                Token::CloseBracket => String::from(")"),
                Token::Colon => String::from(":"),
                Token::Quote => String::from("\""),
                Token::SingleQuote => String::from("\'"),
                Token::Newline => String::from("\n"),
                Token::Begin => String::from(""),
            }
            .to_lowercase();
    }
}

fn is_alphabetic(c: char) -> bool {
    c.is_alphabetic()
}

fn alpha_or_word_chars(c: char) -> bool {
    c.is_alphabetic() || c == '\'' || c == '’' || c == '-' || c == '\u{2014}' // em-dash
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
        word_match => { |(begin, rest)| {
                let mut word = String::from(begin);
                word += rest;
                Token::Word(word)
            }
        } |
        take_s!( 1 )  => { |_| Token::Junk }
    )
);

fn empty_filter(x: Token) -> Option<Token> {
    Some(x)
}

pub fn get_tokens(filename: &str) -> Result<Vec<Token>, io::Error> {
    return common::get_words_core_fn(filename, get_token, empty_filter);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MarkovInfo {
    scores: BTreeMap<u8, BTreeMap<String, u16>>,
    lookup: BTreeMap<String, BTreeMap<u8, BTreeMap<String, u8>>>,
}

impl MarkovInfo {
    fn new() -> MarkovInfo {
        MarkovInfo {
            scores: BTreeMap::new(),
            lookup: BTreeMap::new(),
        }
    }

    fn add_token(self: &mut MarkovInfo, last: String, token: &String) {
        let last_hash = self.lookup.entry(last.clone()).or_insert(BTreeMap::new());
        let token_score = runner::score(&token);
        let score_entry = last_hash.entry(token_score).or_insert(BTreeMap::new());
        *score_entry.entry(token.clone()).or_insert(0) += 1;
        let root_score_entry = self.scores.entry(token_score).or_insert(BTreeMap::new());
        *root_score_entry.entry(token.clone()).or_insert(0) += 1;
    }

    fn default_get(self: &MarkovInfo, score: u8) -> String {
        match self.scores.get(&score) {
            Some(score_hash) => score_hash.keys().nth(0).unwrap().clone(),
            None => {
                warn!("Have no words with score {} so making up one", score);
                let mut ret = String::new();
                for _ in 0..(score / 10) {
                    ret.push('z');
                }
                for _ in 0..(score % 10) {
                    ret.push('a');
                }
                return ret;
            }
        }
    }

    fn get_token(self: &MarkovInfo, last: &String, score: u8) -> String {
        debug!("Looking up for '{}' and {}", last, score);
        return match self.lookup.get(last) {
            Some(word) => {
                match word.get(&score) {
                    Some(score_hash) => score_hash.keys().nth(0).unwrap().clone(),
                    None => self.default_get(score),
                }
            }
            None => self.default_get(score),
        };
    }
}

pub fn generate_markov<'a>(filename: &str) -> Result<MarkovInfo, io::Error> {
    let tokens = get_tokens(filename)?;
    let str_tokens = tokens.into_iter()
        .filter(|t: &Token| -> bool { if let &Token::Junk = t { false } else { true } })
        .map(|t| t.string());
    let mut res = MarkovInfo::new();
    let mut last = Token::Begin.string();
    for token in str_tokens {
        res.add_token(last, &token);
        last = token.clone();
    }
    println!("{:?}", res.scores.get(&0).unwrap());
    Ok(res)
}

pub fn make_beatnik(wottasquare: &str, markov_fname: &str) -> Result<String, io::Error> {
    let buffer = File::open(markov_fname).unwrap();
    let markov: MarkovInfo = serde_yaml::from_reader(&buffer).map_err(common::io_str_error)?;
    let words = runner::get_wottas(wottasquare)?;
    let mut last = Token::Begin.string();
    let mut out = String::new();
    for word in words {
        let token = markov.get_token(&last, word.score());
        out.push_str(&format!("{} ", token));
        last = token;
    }
    return Ok(out);
}

#[cfg(test)]
mod tests {
    use common;
    use quickcheck::TestResult;
    use super::{empty_filter, get_token};

    #[test]
    fn copes_with_utf_8() {
        common::get_words_core("why—I".as_bytes(), get_token, empty_filter).unwrap();
    }

    quickcheck! {
      fn token_test(xs: String) -> TestResult {
          return match common::get_words_core(xs.as_bytes(), get_token, empty_filter) {
              Ok(_) => TestResult::passed(),
              Err(err) => {
                  println!("Error: '{}'", xs);
                  TestResult::error(format!("{:?}", err))
              }
          }
      }
  }
}
