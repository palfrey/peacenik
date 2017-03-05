#[macro_use]
extern crate nom;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate enum_primitive;
extern crate num_traits;

use num_traits::FromPrimitive;

use std::fs::File;
use std::io::Read;
use std::io;
use std::env;

// Needed because of https://github.com/Geal/nom/issues/345
use nom::{alpha, multispace, digit, IResult};

#[derive(Debug,PartialEq,Eq)]
enum Words {
    Junk,
    Word(Vec<u8>)
}

fn array_to_vec(arr: &[u8]) -> Vec<u8> {
     arr.iter().cloned().collect()
}

named!(get_word<Words>,
    alt!(
        alpha  => { |word| Words::Word(array_to_vec(word)) } |
        alt!(multispace | digit | is_a!(", !;:.()\"'-?|&"))  => { |_| Words::Junk }
    )
);

fn io_str_error<T: std::error::Error + std::marker::Send + std::marker::Sync + 'static>(se: T) -> std::io::Error {
    return io::Error::new(io::ErrorKind::Other, se);
}

fn get_words(filename: &str) -> Result<Vec<String>, io::Error> {
    let mut f = try!(File::open(filename));
    let mut buffer = Vec::new();
    try!(f.read_to_end(&mut buffer));
    let mut result = Vec::new();
    let mut remaining = buffer.as_slice();
    loop {
        match get_word(remaining) {
            IResult::Done(further, word) => {
                match word {
                    Words::Junk => {},
                    Words::Word(word) => {
                        let string_word = std::str::from_utf8(&word).map_err(io_str_error)?;
                        result.push(String::from(string_word).to_lowercase());
                    }
                }
                remaining = further;
            }
            rest => {
                match rest {
                    nom::IResult::Error(nom::verbose_errors::Err::Position(_, characters)) => {
                        let location = std::str::from_utf8(characters).map_err(io_str_error)?;
                        return Err(io::Error::new(io::ErrorKind::Other, format!("Don't know how to parse: {}", location.chars().take(50).collect::<String>())));
                    },
                    _ => {}
                };
                
                //rest.to_result().map_err(io_str_error)?;
            }
        }
        if remaining.len() == 0 {
            break;
        }
    }
    return Ok(result);
}

fn score(word: &str) -> u8 {
    let mut result = 0;
    for letter in word.chars() {
        result += match letter {
            // Scrabble scores from https://en.wikipedia.org/wiki/Scrabble_letter_distributions#English
            'a'|'e'|'i'|'o'|'u'|'n'|'r'|'t'|'l'|'s' => 1,
            'd'|'g' => 2,
            'b'|'c'|'m'|'p' => 3,
            'f'|'h'|'v'|'w'|'y' => 4,
            'k' => 5,
            'j'|'x' => 8,
            'q'|'z' => 10,
            _ => 0
        }
    }
    return result;
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
        None => Command::NOP
    }
}

fn main() {
    env_logger::init().unwrap();
    let mut stack: Vec<u8> = Vec::new();
    let words = get_words(&env::args().nth(1).unwrap()).unwrap();
    let scores: Vec<u8> = words.iter().map(|w| score(&w)).collect();
    let mut pc: usize = 0;
    loop {
        debug!("'{}' = {} ({:?})", words[pc], scores[pc], action(scores[pc]));
        match action(scores[pc]) {
            Command::PUSH => {
                pc +=1;
                debug!("Pushing {} to stack", scores[pc]);
                stack.push(scores[pc]);
            }
            Command::DISCARD => {stack.pop().expect("stack value");},
            Command::ADD => {
                let x = stack.pop().expect("first value");
                let y = stack.pop().expect("second value");
                stack.push(x.wrapping_add(y));
            }
            Command::INPUT => {
                // FIXME: read from stdin
                stack.push('a' as u8);
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
                pc +=1;
                if check != 0 {
                    pc += scores[pc] as usize;
                }
            }
            Command::SKIP_AHEAD_ZERO => {
                let check = stack.pop().expect("check value");
                pc +=1;
                if check == 0 {
                    pc += scores[pc] as usize;
                }
            }
            Command::SKIP_BACK_ZERO => {
                let check = stack.pop().expect("check value");
                if check == 0 {
                    pc -= scores[pc+1] as usize;
                }
            }
            Command::SKIP_BACK_NONZERO => {
                let check = stack.pop().expect("check value");
                if check != 0 {
                    pc -= scores[pc+1] as usize;
                }
            }
            Command::STOP => break,
            Command::NOP => {}
        }
        pc += 1;
        if pc == scores.len() {
            break;
        }
    }
    debug!("Stack: {:?}", stack);
}
