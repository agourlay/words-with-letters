mod args;

use std::collections::HashSet;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::mem::swap;

use crate::args::get_args;

fn main() -> io::Result<()> {
    let (cli_dictionary, cli_sentence_length_in_words, cli_letters) = get_args();
    let file = File::open(cli_dictionary)?;
    let reader = BufReader::new(file);
    let my_letters: Vec<char> = cli_letters.chars().collect();
    let found = sentences_for_letters(reader, &my_letters, cli_sentence_length_in_words)?;
    println!(
        "Found {} results -- listed sorted by length:\n",
        found.len()
    );
    found.iter().for_each(|r| println!("{}", r));
    Ok(())
}

#[derive(Eq, PartialEq, Hash)]
struct Sentence {
    words: Vec<String>, // words is expected to be sorted to detect duplicates later on
    remaining_letters: Vec<char>,
    length: usize,
}

impl Sentence {
    fn new(words: Vec<String>, remaining_letters: Vec<char>) -> Sentence {
        let length = words.iter().map(|w| w.len()).sum();
        Sentence {
            words,
            remaining_letters,
            length,
        }
    }

    fn start(word: String, remaining_letters: Vec<char>) -> Sentence {
        let length = word.len();
        Sentence {
            words: vec![word],
            remaining_letters,
            length,
        }
    }

    fn display(&self) -> String {
        let meta = if self.remaining_letters.is_empty() {
            "".to_string()
        } else {
            format!("(unused letters:{:?})", self.remaining_letters)
        };
        let fused = self.words.iter().fold(String::new(), |acc, w| {
            if acc.is_empty() {
                w.clone()
            } else {
                format!("{} {}", acc, w)
            }
        });
        format!("{} {}", fused, meta)
    }
}

fn sentences_for_letters(
    reader: BufReader<File>,
    letters: &Vec<char>,
    sentence_length_in_words: usize,
) -> io::Result<Vec<String>> {
    let words = words_for_letters_in_file(reader, letters)?;
    if sentence_length_in_words > 1 {
        println!(
            "Found {} base words from the dictionary using the input letters {:?}",
            words.len(),
            letters
        );
        println!(
            "Building sentences with {} words, it might take a while depending on your settings...",
            sentence_length_in_words
        );
    }

    let mut sentences_found: HashSet<Sentence> = words
        .iter()
        .map(|w| {
            let mut remaining_letters_for_sentence: Vec<char> = letters.clone();
            for l in w.chars() {
                remove_first(&mut remaining_letters_for_sentence, |rl| rl == &l);
            }
            Sentence::start(w.clone(), remaining_letters_for_sentence)
        })
        .collect();
    let mut in_progress_sentences: HashSet<Sentence> = HashSet::new();

    // TODO try something cleaner with a fold/scan or recursion instead
    for _i in 1..sentence_length_in_words {
        // TODO could also work on several threads by splitting the sentences in chunks
        for sentence in &sentences_found {
            if !sentence.remaining_letters.is_empty() {
                // TODO build a reverse index of the word to quickly get all the possible words for a given letter
                for w in &words {
                    if word_can_build_from_letters(&w, &sentence.remaining_letters) {
                        let mut more_words: Vec<String> = Vec::new();
                        sentence
                            .words
                            .iter()
                            .for_each(|sw| more_words.push(sw.to_owned()));
                        more_words.push(w.clone());
                        more_words.sort_by(|a, b| a.len().cmp(&b.len()).reverse());

                        let mut remaining_letters = sentence.remaining_letters.clone();
                        for l in w.chars() {
                            remove_first(&mut remaining_letters, |rl| rl == &l);
                        }

                        let sentence = Sentence::new(more_words, remaining_letters);
                        in_progress_sentences.insert(sentence);
                    }
                }
            } // the sentence is not carried over for the next round if is already complete
        }
        // Sneaky swap for the next loop to continue where we left off
        swap(&mut sentences_found, &mut in_progress_sentences);
    }

    let mut sorted_sentences: Vec<&Sentence> = sentences_found.iter().collect();
    sorted_sentences.sort_by(|a, b| a.length.cmp(&b.length).reverse());

    let display_sentences: Vec<String> = sorted_sentences
        .iter()
        .map(|sentence| sentence.display())
        .collect();
    Ok(display_sentences)
}

fn remove_first<T, F>(vec: &mut Vec<T>, mut filter: F)
where
    F: for<'a> FnMut(&'a T) -> bool,
{
    let mut removed = false;
    vec.retain(move |item| {
        if removed || !filter(item) {
            true
        } else {
            removed = true;
            false
        }
    });
}

//TODO better be case insensitive (German words)
fn word_can_build_from_letters(word: &String, letters: &Vec<char>) -> bool {
    // quick way out if the word is longer than the letters available
    if word.len() > letters.len() {
        false
    } else {
        let mut remaining_chars = letters.to_owned();
        let mut success = true;
        word.chars().for_each(|c| {
            if success && remaining_chars.contains(&c) {
                remove_first(&mut remaining_chars, |rl| rl == &c);
            } else {
                success = false;
            }
        });
        success
    }
}

fn words_for_letters_in_file(
    reader: BufReader<File>,
    letters: &Vec<char>,
) -> io::Result<Vec<String>> {
    let mut found: Vec<String> = Vec::new();
    for line in reader.lines() {
        let word = line?;
        if word_can_build_from_letters(&word, letters) {
            found.push(word.clone());
        }
    }
    Ok(found)
}

#[cfg(test)]
mod words_with_letters_tests {
    use super::*;

    #[test]
    fn word_can_build_from_letters_duplicate_letter() {
        let word = "happy".to_string();
        let chars = vec!['h', 'a', 'p', 'y'];
        assert_eq!(word_can_build_from_letters(&word, &chars), false)
    }

    #[test]
    fn word_can_build_from_letters_happy() {
        let word = "happy".to_string();
        let chars = vec!['h', 'a', 'p', 'y', 'p'];
        assert_eq!(word_can_build_from_letters(&word, &chars), true)
    }

    #[test]
    fn remove_first_exists() {
        let mut chars = vec!['h', 'a', 'p', 'y', 'p'];
        remove_first(&mut chars, |c| *c == 'a');
        assert_eq!(chars, vec!['h', 'p', 'y', 'p'])
    }

    #[test]
    fn remove_first_no_match() {
        let mut chars = vec!['h', 'a', 'p', 'y', 'p'];
        remove_first(&mut chars, |c| *c == 'z');
        assert_eq!(chars, chars)
    }
}
