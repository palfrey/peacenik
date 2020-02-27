use nom::{verbose_errors, IResult};
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;
use std::marker;
use std::str;

pub fn io_str_error<T: error::Error + marker::Send + marker::Sync + 'static>(se: T) -> io::Error {
    return io::Error::new(io::ErrorKind::Other, se);
}

pub fn get_words_core<Parser, Filter, RawItem, Item>(
    characters: &str,
    mut function: Parser,
    mut filter: Filter,
) -> Result<Vec<Item>, io::Error>
where
    Parser: FnMut(&str) -> IResult<&str, RawItem>,
    Filter: FnMut(RawItem) -> Option<Item>,
    Item: fmt::Debug,
    RawItem: fmt::Debug,
{
    let mut remaining = characters;
    let mut result = Vec::new();
    loop {
        if remaining.is_empty() {
            break;
        }
        match function(remaining) {
            IResult::Done(further, word) => {
                if let Some(x) = filter(word) {
                    result.push(x);
                }
                remaining = further;
            }
            IResult::Error(verbose_errors::Err::Position(errorkind, characters)) => {
                let err = format!(
                    "Don't know how to parse due to {:?}: {}",
                    errorkind,
                    characters.chars().take(50).collect::<String>()
                );
                return Err(io::Error::new(io::ErrorKind::InvalidData, err));
            }
            IResult::Incomplete(_) => {
                break;
            }
            rest => {
                panic!(format!("foo: {:?}", rest));
            }
        }
    }
    return Ok(result);
}

pub fn get_words_core_fn<Parser, Filter, RawItem, Item>(
    filename: &str,
    function: Parser,
    filter: Filter,
) -> Result<Vec<Item>, io::Error>
where
    Parser: FnMut(&str) -> IResult<&str, RawItem>,
    Filter: FnMut(RawItem) -> Option<Item>,
    Item: fmt::Debug,
    RawItem: fmt::Debug,
{
    let mut f = File::open(filename)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    let characters = str::from_utf8(&buffer).map_err(io_str_error)?;
    return get_words_core(characters, function, filter);
}

fn is_alphabetic(c: char) -> bool {
    c.is_alphabetic()
}

fn alpha_or_word_chars(c: char) -> bool {
    c.is_alphabetic() || c == '\'' || c == '’' || c == '-' || c == '\u{2014}' // em-dash
}

named!(pub word_match<&str, (&str, &str)>,
    tuple!(
        take_while1!(is_alphabetic),
        take_while!(alpha_or_word_chars)
    )
);
