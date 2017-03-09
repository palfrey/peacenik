// Needed because of https://github.com/Geal/nom/issues/345

use common;
use nom::{alpha, digit, multispace, newline};
use num_traits::FromPrimitive;
use std::io;
use std::str::{self, FromStr};

#[derive(Debug,PartialEq,Eq)]
pub struct Word {
    word: String,
    score: u8,
}

#[derive(Debug,PartialEq,Eq)]
enum RawWord {
    Junk,
    Word(Word),
}

fn score(word: &str) -> u8 {
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

named!(get_word<RawWord>,
    alt!(
        alpha => { |word| {
                let word = String::from(str::from_utf8(word).unwrap());
                let sc = score(&word.to_lowercase());
                RawWord::Word(Word{word:word, score:sc})
            }
        } |
        alt!(multispace | digit | is_a!(", !;:.()\"'-?|&"))  => { |_| RawWord::Junk }
    )
);

named!(get_wotta<RawWord>,
    map!(tuple!(
        tag!("["),
        digit,
        opt!(
            tuple!(
                tag!(":"),
                is_not!("]")
            )
        ),
        tag!("]"),
        opt!(newline)
    ), |(_, raw_score, comment_opt, _, _)|{
        let word = String::from(str::from_utf8((comment_opt as Option<(&[u8], &[u8])>).unwrap().1).unwrap());
        RawWord::Word(Word{word: word, score: u8::from_str(str::from_utf8(raw_score).unwrap()).unwrap()})
        })
);

fn word_filter(word: RawWord) -> Option<Word> {
    if let RawWord::Word(x) = word {
        Some(x)
    } else {
        None
    }
}

pub fn get_words(filename: &str) -> Result<Vec<Word>, io::Error> {
    return common::get_words_core(filename, get_word, word_filter);
}

pub fn get_wottas(filename: &str) -> Result<Vec<Word>, io::Error> {
    return common::get_words_core(filename, get_wotta, word_filter);
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
    return match Command::from_u8(act) {
        Some(val) => val,
        None => Command::NOP,
    };
}

pub fn run_beatnik(words: &[Word]) {
    let mut stack: Vec<u8> = Vec::new();
    let mut pc: usize = 0;
    loop {
        debug!("'{}' = {} ({:?})", words[pc].word, words[pc].score, action(words[pc].score));
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
