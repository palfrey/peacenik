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
extern crate rand;

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

fn word_parser(items: Result<Vec<runner::Word>, io::Error>) -> Vec<runner::Word> {
    return match items {
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
}

fn main() {
    env_logger::init().unwrap();
    let matches = App::new("peacenik")
        .version("1.0")
        .author("Tom Parker <palfrey@tevp.net>")
        .about("Beatnik language tools")
        .subcommand(SubCommand::with_name("run")
            .about("Beatnik interpreter")
            .arg(Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("wottasquare")
            .about("Wottasquare interpreter")
            .arg(Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("wottasquare-dumper")
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
        ("run", Some(args)) => {
            let input_fname = args.value_of("INPUT").expect("input filename");
            let items = runner::get_words_fn(input_fname);
            let words = word_parser(items);
            runner::run_beatnik(&words);
        }
        ("wottasquare", Some(args)) => {
            let input_fname = args.value_of("INPUT").expect("input filename");
            let items = runner::get_wottas_fn(input_fname);
            let words = word_parser(items);
            runner::run_beatnik(&words);
        }
        ("wottasquare-dumper", Some(args)) => {
            let input_fname = args.value_of("INPUT").expect("input filename");
            let items = runner::get_words_fn(input_fname);
            let words = word_parser(items);
            runner::output_wottasquare(words);
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
            let words = runner::get_wottas_fn(input_fname).expect("wottasquare data");
            let markov_data = markov::read_markov(markov_fname).expect("markov data");
            let markov_out = markov::make_beatnik(&words, &markov_data).expect("markov");
            let output_fname = args.value_of("OUTPUT").expect("output name");
            let mut buffer = File::create(output_fname).unwrap();
            buffer.write(markov_out.as_bytes()).unwrap();
        }
        _ => panic!("No command"),
    }
}

#[cfg(test)]
mod tests {
    use markov;
    use quickcheck::TestResult;
    use runner;

    quickcheck!{
        fn wotta_two_way(xs: Vec<u8>, source_words: String) -> TestResult {
            let mut info = String::new();
            info.push_str(&format!("Source_words: {:?}\n", source_words));
            let mut markov = markov::MarkovInfo::new();
            let tokens = markov::get_tokens(&source_words).unwrap();
            let mut last = markov::Token::Begin.string();
            for token in tokens {
                let str_token = token.string();
                markov.add_token(last, &str_token);
                last = str_token;
            }
            info.push_str(&format!("Markov: {:?}\n", markov));

            let words = xs.iter()
                .filter(|x| *x !=&0)
                .map(|x| runner::Word{score:*x, word:String::from("")})
                .collect();
            info.push_str(&format!("Score in: {:?}\n", xs));
            let markov_out = markov::make_beatnik(&words, &markov).unwrap();
            info.push_str(&format!("Markov out: {:?}\n", markov_out));
            let words_out = runner::get_words(&markov_out).unwrap();
            info.push_str(&format!("Words out: {:?}\n", words_out));
            let score_out: Vec<u8> = words_out.iter().map(|x|x.score).filter(|x| x !=&0).collect();
            info.push_str(&format!("Score out: {:?}\n", score_out));
            let res = xs.into_iter().filter(|x| x != &0).collect::<Vec<u8>>() == score_out;
            info.push_str(&format!("Res: {}\n", res));
            if res {
                TestResult::passed()
            }
            else {
                TestResult::error(info)
            }
        }
    }
}
