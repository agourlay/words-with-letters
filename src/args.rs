use clap::{value_t, App, Arg};
use std::path::Path;

pub fn get_args() -> (String, usize, String) {
    let matches = App::new("words-with-letters")
        .version("0.1.0")
        .author("Arnaud Gourlay <arnaud.gourlay@gmail.com>")
        .about("Making words with letters")
        .arg(
            Arg::with_name("letters")
                .help("letters to use")
                .long("letters")
                .short("l")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("dictionaryFile")
                .help("dictionary file")
                .long("dictionaryFile")
                .short("dl")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("sentenceLength")
                .help("sentence length in words")
                .long("sentenceLength")
                .short("sl")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let sentence_length =
        value_t!(matches, "sentenceLength", usize).expect("sentenceLength was not an integer");
    if sentence_length == 0 {
        panic!("invalid sentenceLength - must be a positive integer")
    }

    let dictionary_file = matches.value_of("dictionaryFile").expect("impossible");
    if !Path::new(dictionary_file).is_file() {
        panic!("dictionaryFile does not exist")
    }

    let letters = matches.value_of("letters").expect("impossible");
    if letters.is_empty() {
        panic!("letters is empty")
    }

    (
        dictionary_file.to_string(),
        sentence_length,
        letters.to_string(),
    )
}
