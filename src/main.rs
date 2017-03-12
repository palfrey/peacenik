#[macro_use]
extern crate nom;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate enum_primitive;
extern crate num_traits;
extern crate clap;
extern crate serde_yaml;

#[macro_use]
extern crate serde_derive;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod runner;
mod markov;
mod common;

use clap::{App, Arg, SubCommand};
use std::fs::File;
use std::io::{self, Write};

fn main() {
    env_logger::init().unwrap();
    let matches = App::new("peacenik")
        .version("1.0")
        .author("Tom Parker <palfrey@tevp.net>")
        .about("Beatnik language tools")
        .arg(Arg::with_name("wottasquare")
            .short("w")
            .long("wottasquare")
            .help("Wottasquare input mode (default: Beatnik input)"))
        .subcommand(SubCommand::with_name("run")
            .about("Beatnik interpreter")
            .arg(Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("wottasquare")
            .about("Wottasquare dumper")
            .arg(Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("generate-markov")
            .about("Markov chain generator")
            .arg(Arg::with_name("INPUT")
                .short("i")
                .takes_value(true)
                .help("Sets the input file to use")
                .required(true))
            .arg(Arg::with_name("OUTPUT")
                .short("o")
                .takes_value(true)
                .help("Sets the output file to use")
                .required(true)))
        .subcommand(SubCommand::with_name("markov-beatnik")
            .about("Beatnik from Wottasquare using Markov")
            .arg(Arg::with_name("INPUT")
                .short("i")
                .takes_value(true)
                .help("Sets the input file to use")
                .required(true))
            .arg(Arg::with_name("MARKOV")
                .short("m")
                .takes_value(true)
                .help("Sets the markov file to use")
                .required(true))
            .arg(Arg::with_name("OUTPUT")
                .short("o")
                .takes_value(true)
                .help("Sets the output file to use")
                .required(true)))
        .get_matches();
    match matches.subcommand() {
        ("run", Some(args)) |
        ("wottasquare", Some(args)) => {
            let input_fname = args.value_of("INPUT").expect("input filename");
            let items = if matches.is_present("wottasquare") {
                runner::get_wottas(input_fname)
            } else {
                runner::get_words(input_fname)
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
            match matches.subcommand_name() {
                Some("run") => runner::run_beatnik(&words),
                Some("wottasquare") => runner::output_wottasquare(words),
                _ => panic!("No command"),
            }
        }
        ("generate-markov", Some(args)) => {
            let input_fname = args.value_of("INPUT").unwrap();
            let markov = markov::generate_markov(input_fname).expect("markov");
            let output_fname = args.value_of("OUTPUT").expect("output name");
            let mut buffer = File::create(output_fname).unwrap();
            serde_yaml::to_writer(&mut buffer, &markov).unwrap();
        }
        ("markov-beatnik", Some(args)) => {
            let input_fname = args.value_of("INPUT").unwrap();
            let markov_fname = args.value_of("MARKOV").unwrap();
            let markov = markov::make_beatnik(input_fname, markov_fname).expect("markov");
            let output_fname = args.value_of("OUTPUT").expect("output name");
            let mut buffer = File::create(output_fname).unwrap();
            buffer.write(markov.as_bytes()).unwrap();
        }
        _ => panic!("No command"),
    }
}
