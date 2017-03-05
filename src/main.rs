#[macro_use]
extern crate nom;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate enum_primitive;
extern crate num_traits;
#[macro_use]
extern crate clap;
use clap::{Arg, App};

use num_traits::FromPrimitive;

use std::fs::File;
use std::io::Read;
use std::io;
use std::str::FromStr;

// Needed because of https://github.com/Geal/nom/issues/345
use nom::{alpha, multispace, digit, IResult, newline};

#[derive(Debug,PartialEq,Eq)]
struct Word {
    word: String,
    score: u8
}

#[derive(Debug,PartialEq,Eq)]
enum RawWord {
    Junk,
    Word(Word)
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

named!(get_word<RawWord>,
    alt!(
        alpha => { |word| {
                let word = String::from(std::str::from_utf8(word).unwrap());
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
        let word = String::from(std::str::from_utf8((comment_opt as Option<(&[u8], &[u8])>).unwrap().1).unwrap());
        RawWord::Word(Word{word: word, score: u8::from_str(std::str::from_utf8(raw_score).unwrap()).unwrap()})
        })
);

fn io_str_error<T: std::error::Error + std::marker::Send + std::marker::Sync + 'static>(se: T) -> std::io::Error {
    return io::Error::new(io::ErrorKind::Other, se);
}

fn get_words_core<F>(filename: &str, mut function: F) -> Result<Vec<Word>, io::Error>
    where F: FnMut(&[u8]) -> nom::IResult<&[u8], RawWord>
{
    let mut f = try!(File::open(filename));
    let mut buffer = Vec::new();
    try!(f.read_to_end(&mut buffer));
    let mut result = Vec::new();
    let mut remaining = buffer.as_slice();
    loop {
        match function(remaining) {
            IResult::Done(further, word) => {
                if let RawWord::Word(x) = word {
                    result.push(x);
                }
                remaining = further;
            }
            rest => {
                match rest {
                    nom::IResult::Error(nom::verbose_errors::Err::Position(_, characters)) => {
                        let location = std::str::from_utf8(characters).map_err(io_str_error)?;
                        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Don't know how to parse: {}", location.chars().take(50).collect::<String>())));
                    },
                    _ => {}
                };
            }
        }
        if remaining.len() == 0 {
            break;
        }
    }
    return Ok(result);
}

fn get_words(filename: &str) -> Result<Vec<Word>, io::Error> {
    return get_words_core(filename, get_word);
}

fn get_wottas(filename: &str) -> Result<Vec<Word>, io::Error> {
    return get_words_core(filename, get_wotta);
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

fn run_beatnik(words: Vec<Word>) {
    let mut stack: Vec<u8> = Vec::new();
    let mut pc: usize = 0;
    loop {
        debug!("'{}' = {} ({:?})", words[pc].word, words[pc].score, action(words[pc].score));
        match action(words[pc].score) {
            Command::PUSH => {
                pc +=1;
                debug!("Pushing {} to stack", words[pc].score);
                stack.push(words[pc].score);
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
                    pc += words[pc].score as usize;
                }
            }
            Command::SKIP_AHEAD_ZERO => {
                let check = stack.pop().expect("check value");
                pc +=1;
                if check == 0 {
                    pc += words[pc].score as usize;
                }
            }
            Command::SKIP_BACK_ZERO => {
                let check = stack.pop().expect("check value");
                if check == 0 {
                    pc -= words[pc+1].score as usize;
                }
            }
            Command::SKIP_BACK_NONZERO => {
                let check = stack.pop().expect("check value");
                if check != 0 {
                    pc -= words[pc+1].score as usize;
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

fn output_wottasquare(words: Vec<Word>) {
    for word in words {
        println!("[{}:{:?}]", word.score, action(word.score));
    }
}

fn main() {
    env_logger::init().unwrap();
    let matches = App::new("peacenik")
                          .version("1.0")
                          .author("Tom Parker <palfrey@tevp.net>")
                          .about("Beatnik interpreter")
                          .arg(Arg::with_name("output-wottasquare")
                               .short("w")
                               .long("output-wottasquare")
                               .help("Switches into wottasquare output mode"))
                          .arg(Arg::with_name("input-wottasquare")
                               .short("i")
                               .long("input-wottasquare")
                               .help("Switches into wottasquare input mode"))
                          .arg(Arg::with_name("INPUT")
                               .help("Sets the input file to use")
                               .required(true)
                               .index(1))
                          .get_matches();
    let items = if matches.is_present("input-wottasquare") {
        get_wottas(matches.value_of("INPUT").unwrap())
    }
    else {
        get_words(matches.value_of("INPUT").unwrap())
    };
    let words = match items {
        Ok(val) => val,
        Err(err) => {
            if err.kind() == io::ErrorKind::InvalidData {
                println!("{}", err.get_ref().unwrap());
                std::process::exit(-1);
            }
            else {
                panic!("Error during parsing: {}", err);
            }
        }
    };
    if matches.is_present("output-wottasquare") {
        output_wottasquare(words);
    }
    else {
        run_beatnik(words);
    }
}