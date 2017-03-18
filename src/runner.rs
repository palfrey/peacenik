// Needed because of https://github.com/Geal/nom/issues/345

use common::{self, word_match};
use nom::digit;
use num_traits::FromPrimitive;
use std::io;
use std::str::{self, FromStr};

#[derive(Debug)]
pub struct Word {
    pub word: String,
    pub score: u8,
}

impl Word {
    pub fn score(self: &Word) -> u8 {
        self.score
    }
}

#[derive(Debug)]
enum RawWord {
    Junk,
    Word(Word),
}

pub fn score(word: &str) -> u8 {
    let mut result = 0;
    for letter in word.chars() {
        result += match letter {
            // Scrabble scores from https://en.wikipedia.org/wiki/Scrabble_letter_distributions#English
            'a' | 'e' | 'i' | 'o' | 'u' | 'n' | 'r' | 't' | 'l' | 's' => 1,
            'd' | 'g' => 2,
            'b' | 'c' | 'm' | 'p' => 3,
            'f' | 'h' | 'v' | 'w' | 'y' => 4,
            'k' => 5,
            'j' | 'x' => 8,
            'q' | 'z' => 10,
            _ => 0,
        }
    }
    return result;
}

named!(get_word<&str, RawWord>,
    alt!(
        word_match => { |(begin, rest)| {
                let mut word = String::from(begin);
                word += rest;
                let sc = score(&word.to_lowercase());
                RawWord::Word(Word{word:word, score:sc})
            }
        } |
        take_s!( 1 )  => { |_| RawWord::Junk }
    )
);

named!(get_wotta<&str, RawWord>,
    map!(tuple!(
        tag_s!("["),
        digit,
        opt!(
            tuple!(
                tag_s!(":"),
                is_not_s!("]")
            )
        ),
        tag_s!("]"),
        opt!(tag_s!("\n"))
    ), |(_, raw_score, comment_opt, _, _)|{
        let word = String::from((comment_opt as Option<(&str, &str)>).unwrap().1);
        RawWord::Word(Word{word: word, score: u8::from_str(raw_score).unwrap()})
        })
);

fn word_filter(word: RawWord) -> Option<Word> {
    if let RawWord::Word(x) = word {
        Some(x)
    } else {
        None
    }
}

pub fn get_words_fn(filename: &str) -> Result<Vec<Word>, io::Error> {
    return common::get_words_core_fn(filename, get_word, word_filter);
}

#[cfg(test)]
pub fn get_words(buffer: &str) -> Result<Vec<Word>, io::Error> {
    return common::get_words_core(buffer, get_word, word_filter);
}

pub fn get_wottas_fn(filename: &str) -> Result<Vec<Word>, io::Error> {
    return common::get_words_core_fn(filename, get_wotta, word_filter);
}

#[cfg(test)]
pub fn get_wottas(buffer: &str) -> Result<Vec<Word>, io::Error> {
    return common::get_words_core(buffer, get_wotta, word_filter);
}

enum_from_primitive! {
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq)]
    enum Command {
        PUSH = 5,
        DISCARD = 6,
        ADD = 7,
        INPUT = 8,
        OUTPUT = 9,
        SUBTRACT = 10,
        SWAP = 11,
        DUP = 12,
        SKIP_AHEAD_ZERO = 13,
        SKIP_AHEAD_NONZERO = 14,
        SKIP_BACK_ZERO = 15,
        SKIP_BACK_NONZERO = 16,
        STOP = 17,
        NOP,
    }
}

fn action(act: u8) -> Command {
    match Command::from_u8(act) {
        Some(val) => val,
        None => Command::NOP,
    }
}

pub fn run_beatnik(words: &[Word]) {
    let mut stack: Vec<u8> = Vec::new();
    let mut pc: usize = 0;
    loop {
        debug!("'{}' = {} ({:?})",
               words[pc].word,
               words[pc].score,
               action(words[pc].score));
        match action(words[pc].score) {
            Command::PUSH => {
                pc += 1;
                debug!("Pushing {} to stack", words[pc].score);
                stack.push(words[pc].score);
            }
            Command::DISCARD => {
                stack.pop().expect("stack value");
            }
            Command::ADD => {
                let x = stack.pop().expect("first value");
                let y = stack.pop().expect("second value");
                stack.push(x.wrapping_add(y));
            }
            Command::INPUT => {
                // FIXME: read from stdin
                stack.push(b'a');
            }
            Command::OUTPUT => print!("{}", stack.pop().expect("character on stack") as char),
            Command::SUBTRACT => {
                let x = stack.pop().expect("first value");
                let y = stack.pop().expect("second value");
                stack.push(y.wrapping_sub(x));
            }
            Command::SWAP => {
                let x = stack.pop().expect("first value");
                let y = stack.pop().expect("second value");
                stack.push(x);
                stack.push(y);
            }
            Command::DUP => {
                let x = stack.pop().expect("first value");
                stack.push(x);
                stack.push(x);
            }
            Command::SKIP_AHEAD_NONZERO => {
                let check = stack.pop().expect("check value");
                pc += 1;
                if check != 0 {
                    pc += words[pc].score as usize;
                }
            }
            Command::SKIP_AHEAD_ZERO => {
                let check = stack.pop().expect("check value");
                pc += 1;
                if check == 0 {
                    pc += words[pc].score as usize;
                }
            }
            Command::SKIP_BACK_ZERO => {
                let check = stack.pop().expect("check value");
                if check == 0 {
                    pc -= words[pc + 1].score as usize;
                }
            }
            Command::SKIP_BACK_NONZERO => {
                let check = stack.pop().expect("check value");
                if check != 0 {
                    pc -= words[pc + 1].score as usize;
                }
            }
            Command::STOP => break,
            Command::NOP => {}
        }
        pc += 1;
        if pc == words.len() {
            break;
        }
    }
    debug!("Stack: {:?}", stack);
}

pub fn output_wottasquare(words: Vec<Word>) {
    for word in words {
        println!("[{}:{:?}]", word.score, action(word.score));
    }
}

#[cfg(test)]
mod tests {
    use super::{get_words, get_wottas};
    use quickcheck::TestResult;

    #[test]
    fn test_apostrophe() {
        let ref word = get_words("she’s").unwrap()[0];
        assert_eq!(word.score, 7);
    }


    quickcheck! {
      fn word_test(xs: String) -> TestResult {
          return match get_words(&xs) {
              Ok(_) => TestResult::passed(),
              Err(err) => {
                  println!("Error: '{}'", xs);
                  TestResult::error(format!("{:?}", err))
              }
          }
      }

      fn wotta_test(xs: String) -> TestResult {
        if xs.len() == 0 {
            return TestResult::discard();
        }
        if xs.find("]").is_some() {
            return TestResult::discard();
        }

        return match get_wottas(&format!("[1:{}]", xs)) {
            Ok(_) => TestResult::passed(),
            Err(err) => {
                println!("Error: '{}'", xs);
                TestResult::error(format!("{:?}", err))
            }
        }
      }
  }
}
