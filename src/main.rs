mod args;

use std::collections::HashSet;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, Lines};
use std::mem::swap;

use crate::args::get_args;

fn main() -> io::Result<()> {
    let (cli_dictionary, cli_sentence_length_in_words, cli_letters, verbose_mode) = get_args();
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
    let found = sentences_for_letters(&words, &my_letters, cli_sentence_length_in_words, true, verbose_mode);
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
            length
        }
    }

    fn start(word: String, remaining_letters: Vec<char>) -> Sentence {
        let length = word.len();
        Sentence {
            words: vec![word],
            remaining_letters,
            length
        }
    }

    fn is_completed(&self) -> bool {
        self.remaining_letters.is_empty()
    }

    fn display(&self, with_unused: bool) -> String {
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
        if with_unused {
            format!("{} {}", fused, meta)
        } else {
            format!("{}", fused)
        }
    }
}

fn sentences_for_letters(
    words: &Vec<String>,
    letters: &Vec<char>,
    sentence_length_in_words: usize,
    with_display_unused: bool,
    verbose_mode: bool,
) -> Vec<String> {
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

    if verbose_mode {
        println!(
            "Progress sentences with {} words, current found {} from {} and in progress {:?} at iteration {:?}",
            sentence_length_in_words,
            sentences_found.len(),
            words.len(),
            in_progress_sentences.len(),
            0
        );
    }

    // TODO try something cleaner with a fold/scan or recursion instead
    for _i in 1..sentence_length_in_words {
        if verbose_mode {
            println!(
                "Progress sentences with {} words, current found {:?} and in progress {:?} at iteration {}",
                sentence_length_in_words,
                sentences_found.len(),
                in_progress_sentences.len(),
                _i
            );
        }
        // TODO could also work on several threads by splitting the sentences in chunks
        inner_sentences_found(words, &mut sentences_found, &mut in_progress_sentences);
        // Sneaky swap for the next loop to continue where we left off
        swap(&mut sentences_found, &mut in_progress_sentences);
    }

    let mut sorted_sentences: Vec<&Sentence> = sentences_found.iter().collect();
    sorted_sentences.sort_by(|a, b| {
        if a.length == b.length {
            a.display(false).cmp(&b.display(false))
        } else {
            a.length.cmp(&b.length).reverse()
        }
    });

    let display_sentences: Vec<String> = sorted_sentences
        .iter()
        .map(|sentence| sentence.display(with_display_unused))
        .collect();
    display_sentences
}

fn inner_sentences_found(
    words: &Vec<String>,
    sentences_found: &mut HashSet<Sentence>,
    in_progress_sentences: &mut HashSet<Sentence>,
) {
    for sentence in sentences_found.iter() {
        if !sentence.is_completed() {
            // TODO build a reverse index of the word to quickly get all the possible words for a given letter
            println!(
                "\tProgress inner sentences with {:?}, current words found {:?}",
                &sentence.length,
                words.len()
            );
            for w in words {
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
pub fn word_can_build_from_letters(word: &String, letters: &Vec<char>) -> bool {
    // quick way out if the word is longer than the letters available
    let word_chars: Vec<char> = word.chars().collect();
    if word_chars.len() > letters.len() {
        false
    } else {
        let mut remaining_chars = letters.to_owned();
        let mut success = true;
        word_chars.iter().for_each(|c| {
            if success && remaining_chars.contains(&c) {
                remove_first(&mut remaining_chars, |rl| rl == c);
            } else {
                success = false;
            }
        });
        success
    }
}

fn words_for_letters_in_file<B: BufRead>(
    reader_lines: Lines<B>,
    letters: &Vec<char>,
) -> io::Result<Vec<String>> {
    let mut found: Vec<String> = Vec::new();
    for line in reader_lines {
        let word = line?;
        if word_can_build_from_letters(&word, letters) {
            found.push(word.clone());
        }
    }
    Ok(found)
}

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

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

    #[test]
    fn sentences_for_letters_multiple_words() {
        let chars = vec![
            'y', 'o', 'u', 'h', 'o', 'u', 'i', 'c', 'a', 'n', 'r', 'u', 'n', 't', 'h', 'i', 's',
        ];
        let words = vec!["you", "can", "run", "this"]
            .iter()
            .map(|w| w.to_string())
            .collect();
        let sentences = sentences_for_letters(&words, &chars, 4, false, false);
        assert_eq!(sentences.len(), 15);
        let expected_result: Vec<String> = vec![
            "this can run you",
            "this can you run",
            "this run can you",
            "this run you can",
            "this you can run",
            "this you run can",
            "this can",
            "this run",
            "this you",
            "can run",
            "can you",
            "run can",
            "run you",
            "you can",
            "you run",
        ]
        .iter()
        .map(|w| w.to_string())
        .collect();
        assert_eq!(sentences.len(), expected_result.len());
        assert_eq!(sentences, expected_result)
    }

    #[quickcheck]
    fn word_can_be_built_from_its_own_letters(word: String) -> bool {
        let chars: Vec<char> = word.chars().collect();
        word_can_build_from_letters(&word, &chars)
    }
}
