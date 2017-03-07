use nom::{IResult, verbose_errors};
use std::error;
use std::fs::File;
use std::io;
use std::io::Read;
use std::marker;
use std::str;
use std::fmt;

fn io_str_error<T: error::Error + marker::Send + marker::Sync + 'static>(se: T) -> io::Error {
    return io::Error::new(io::ErrorKind::Other, se);
}

pub fn get_words_core<Parser, Filter, RawItem, Item>(filename: &str, mut function: Parser, mut filter: Filter) -> Result<Vec<Item>, io::Error>
    where Parser: FnMut(&[u8]) -> IResult<&[u8], RawItem>,
          Filter: FnMut(RawItem) -> Option<Item>,
          Item: fmt::Debug
{
    let mut f = try!(File::open(filename));
    let mut buffer = Vec::new();
    try!(f.read_to_end(&mut buffer));
    let mut result = Vec::new();
    let mut remaining = buffer.as_slice();
    loop {
        match function(remaining) {
            IResult::Done(further, word) => {
                if let Some(x) = filter(word) {
                    println!("Word: '{:?}'", x);
                    result.push(x);
                }
                remaining = further;
            }
            rest => {
                match rest {
                    IResult::Error(verbose_errors::Err::Position(_, characters)) => {
                        let location = str::from_utf8(characters).map_err(io_str_error)?;
                        let err = format!("Don't know how to parse: {}",
                            location.chars().take(50).collect::<String>());
                        return Err(io::Error::new(io::ErrorKind::InvalidData, err));
                    }
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