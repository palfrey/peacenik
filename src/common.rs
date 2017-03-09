use nom::{IResult, verbose_errors};
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;
use std::str;

pub fn get_words_core<Parser, Filter, RawItem, Item>(filename: &str,
                                                     mut function: Parser,
                                                     mut filter: Filter)
                                                     -> Result<Vec<Item>, io::Error>
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
                if let IResult::Error(verbose_errors::Err::Position(errorkind, characters)) = rest {
                    let err = match str::from_utf8(characters) {
                        Ok(location) => {
                            format!("Don't know how to parse due to {:?}: {}",
                                    errorkind,
                                    location.chars().take(50).collect::<String>())
                        }
                        Err(err) => format!("Don't know how to parse due to {:?}: {}", errorkind, err),
                    };
                    return Err(io::Error::new(io::ErrorKind::InvalidData, err));
                }
            }
        }
        if remaining.is_empty() {
            break;
        }
    }
    return Ok(result);
}
