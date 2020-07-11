mod args;
mod words;

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use std::fs::File;
use std::io::{self, prelude::*, BufReader};

use crate::args::get_args;
use crate::words::*;

fn main() -> io::Result<()> {
    let (cli_dictionary, cli_sentence_length_in_words, cli_letters, verbose_mode, top) = get_args();
    let file = File::open(cli_dictionary)?;
    let reader = BufReader::new(file);
    let my_letters: Vec<char> = cli_letters.chars().collect();
    let words = words_for_letters_in_file(reader.lines(), &my_letters)?;
    if cli_sentence_length_in_words > 1 {
        println!(
            "Found {} base words from the dictionary using the input letters {:?}",
            words.len(),
            &my_letters
        );
        println!(
            "Building sentences with {} words, it might take a while depending on your settings...",
            cli_sentence_length_in_words
        );
    }
    let found = sentences_for_letters(
        &words,
        &my_letters,
        cli_sentence_length_in_words,
        true,
        verbose_mode,
    );
    println!(
        "Found {} results -- listed sorted by length:\n",
        found.len()
    );
    let display_nb = top.unwrap_or_else(|| found.len());
    found
        .iter()
        .take(display_nb)
        .for_each(|r| println!("{}", r));
    Ok(())
}
