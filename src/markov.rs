use common::{self, word_match};
use rand;
use rand::Rng;
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
                Token::Comma => String::from(", "),
                Token::FullStop => String::from(". "),
                Token::QuestionMark => String::from("? "),
                Token::OpenBracket => String::from("("),
                Token::CloseBracket => String::from(")"),
                Token::Colon => String::from(": "),
                Token::Quote => String::from("\""),
                Token::SingleQuote => String::from("\'"),
                Token::Newline => String::from("\n"),
                Token::Begin => String::from(""),
            }
            .to_lowercase();
    }
}

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

pub fn get_tokens_fn(filename: &str) -> Result<Vec<Token>, io::Error> {
    return common::get_words_core_fn(filename, get_token, empty_filter);
}

#[cfg(test)]
pub fn get_tokens(buffer: &[u8]) -> Result<Vec<Token>, io::Error> {
    return common::get_words_core(buffer, get_token, empty_filter);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MarkovSymbols {
    count: u16,
    tokens: BTreeMap<String, u16>,
}

impl MarkovSymbols {
    fn new() -> MarkovSymbols {
        MarkovSymbols {
            count: 0,
            tokens: BTreeMap::new(),
        }
    }

    fn add_token(self: &mut MarkovSymbols, token: &String) {
        *self.tokens.entry(token.clone()).or_insert(0) += 1;
        self.count += 1;
    }

    fn get_key(self: &MarkovSymbols) -> String {
        let mut rng = rand::thread_rng();
        let mut choice = rng.gen_range(0, self.count);
        for (key, value) in self.tokens.iter() {
            if &choice < value {
                return key.clone();
            }
            choice -= *value;
        }
        panic!("Didn't find token within range");
    }

    fn count(self: &MarkovSymbols) -> u16 {
        self.count
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct MarkovScores {
    tokens: BTreeMap<u8, MarkovSymbols>,
    count: u16,
}

fn title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c.flat_map(|t| t.to_lowercase())).collect(),
    }
}

impl MarkovScores {
    fn new() -> MarkovScores {
        MarkovScores {
            tokens: BTreeMap::new(),
            count: 0,
        }
    }

    fn get_key(self: &MarkovScores, score: &u8) -> Option<String> {
        return match self.tokens.get(score) {
            Some(score_hash) => {
                let mut rng = rand::thread_rng();
                let zero_score = self.tokens.get(&0).and_then(|x| Some(x.count())).unwrap_or(0);
                if zero_score == 0 {
                    return Some(score_hash.get_key());
                }
                let choice = rng.gen_range(0, self.count);
                if choice < zero_score {
                    let zero_token = self.tokens.get(&0).unwrap().get_key();
                    let mut rest = self.get_key(score).unwrap();
                    if zero_token == Token::FullStop.string() || zero_token == Token::QuestionMark.string() ||
                       zero_token == Token::Newline.string() {
                        rest = title_case(&rest);
                    }
                    Some(zero_token + &rest)
                } else {
                    let normal = score_hash.get_key();
                    if normal == "i" {
                        Some(String::from("I"))
                    } else {
                        Some(normal)
                    }
                }
            }
            None => None,
        };
    }

    fn add_token(self: &mut MarkovScores, score: u8, token: &String) {
        let score_entry = self.tokens.entry(score).or_insert(MarkovSymbols::new());
        score_entry.add_token(token);
        self.count += 1;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MarkovInfo {
    scores: MarkovScores,
    lookup: BTreeMap<String, MarkovScores>,
}

impl MarkovInfo {
    pub fn new() -> MarkovInfo {
        MarkovInfo {
            scores: MarkovScores::new(),
            lookup: BTreeMap::new(),
        }
    }

    pub fn add_token(self: &mut MarkovInfo, last: String, token: &String) {
        let token_score = runner::score(&token.to_lowercase());
        let last_hash = self.lookup.entry(last.clone()).or_insert(MarkovScores::new());
        last_hash.add_token(token_score, token);
        self.scores.add_token(token_score, token);
    }

    fn default_get(self: &MarkovInfo, score: u8) -> String {
        match self.scores.get_key(&score) {
            Some(score_hash) => score_hash,
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
                match word.get_key(&score) {
                    Some(score_hash) => score_hash,
                    None => self.default_get(score),
                }
            }
            None => self.default_get(score),
        };
    }
}

pub fn generate_markov<'a>(filename: &str) -> Result<MarkovInfo, io::Error> {
    let tokens = get_tokens_fn(filename)?;
    let str_tokens = tokens.into_iter()
        .filter(|t: &Token| -> bool { if let &Token::Junk = t { false } else { true } })
        .map(|t| t.string());
    let mut res = MarkovInfo::new();
    let mut last = Token::Begin.string();
    for token in str_tokens {
        res.add_token(last, &token);
        last = token.clone();
    }
    Ok(res)
}

pub fn read_markov(markov_fname: &str) -> Result<MarkovInfo, io::Error> {
    let buffer = File::open(markov_fname).unwrap();
    Ok(serde_yaml::from_reader(&buffer).map_err(common::io_str_error)?)
}

pub fn make_beatnik(words: &Vec<runner::Word>, markov: &MarkovInfo) -> Result<String, io::Error> {
    let mut last = Token::Begin.string();
    let mut out = String::new();
    for word in words {
        let mut token = markov.get_token(&last, word.score());
        if last == "" {
            token = title_case(&token);
        } else if !token.starts_with(".") && !token.starts_with("?") && !token.starts_with(",") {
            out.push_str(" ");
        }

        out.push_str(&token);
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
