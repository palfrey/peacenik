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

mod runner;
use clap::{App, Arg};
use std::io;

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
        runner::get_wottas(matches.value_of("INPUT").unwrap())
    } else {
        runner::get_words(matches.value_of("INPUT").unwrap())
    };
    let words = match items {
        Ok(val) => val,
        Err(err) => {
            if err.kind() == io::ErrorKind::InvalidData {
                println!("{}", err.get_ref().unwrap());
                std::process::exit(-1);
            } else {
                panic!("Error during parsing: {}", err);
            }
        }
    };
    if matches.is_present("output-wottasquare") {
        runner::output_wottasquare(words);
    } else {
        runner::run_beatnik(words);
    }
}
