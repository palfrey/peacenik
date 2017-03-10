use nom::{IResult, verbose_errors};
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;
use std::marker;
use std::str;

fn io_str_error<T: error::Error + marker::Send + marker::Sync + 'static>(se: T) -> io::Error {
    return io::Error::new(io::ErrorKind::Other, se);
}

pub fn get_words_core<Parser, Filter, RawItem, Item>(characters: &[u8],
                                                     mut function: Parser,
                                                     mut filter: Filter)
                                                     -> Result<Vec<Item>, io::Error>
    where Parser: FnMut(&str) -> IResult<&str, RawItem>,
          Filter: FnMut(RawItem) -> Option<Item>,
          Item: fmt::Debug,
          RawItem: fmt::Debug
{
    let mut remaining = str::from_utf8(characters).map_err(io_str_error)?;
    let mut result = Vec::new();
    loop {
        if remaining.is_empty() {
            break;
        }
        match function(remaining) {
            IResult::Done(further, word) => {
                if let Some(x) = filter(word) {
                    println!("Word: '{:?}'", x);
                    result.push(x);
                }
                remaining = further;
            }
            IResult::Error(verbose_errors::Err::Position(errorkind, characters)) => {
                let err = format!("Don't know how to parse due to {:?}: {}",
                                  errorkind,
                                  characters.chars().take(50).collect::<String>());
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

pub fn get_words_core_fn<Parser, Filter, RawItem, Item>(filename: &str,
                                                        function: Parser,
                                                        filter: Filter)
                                                        -> Result<Vec<Item>, io::Error>
    where Parser: FnMut(&str) -> IResult<&str, RawItem>,
          Filter: FnMut(RawItem) -> Option<Item>,
          Item: fmt::Debug,
          RawItem: fmt::Debug
{
    let mut f = try!(File::open(filename));
    let mut buffer = Vec::new();
    try!(f.read_to_end(&mut buffer));
    return get_words_core(buffer.as_slice(), function, filter);
}
