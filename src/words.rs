use std::collections::HashSet;
use std::io::{self, prelude::*, Lines};
use std::mem::swap;

#[derive(Eq, PartialEq, Hash)]
struct Sentence {
    words: Vec<String>, // words is expected to be sorted to detect duplicates later on
    remaining_letters: Vec<char>,
    length: usize,
    max_expansion_reached: bool,
}

impl Sentence {
    fn new(words: Vec<String>, remaining_letters: Vec<char>) -> Sentence {
        let length = words.iter().map(|w| w.len()).sum();
        Sentence {
            words,
            remaining_letters,
            length,
            max_expansion_reached: false
        }
    }

    fn start(word: String, remaining_letters: Vec<char>) -> Sentence {
        let length = word.len();
        Sentence {
            words: vec![word],
            remaining_letters,
            length,
            max_expansion_reached: false
        }
    }

    fn mark_as_max_expansion_reached(&self) -> Sentence {
        Sentence {
            words: self.words.clone(),
            remaining_letters: self.remaining_letters.clone(),
            length: self.length,
            max_expansion_reached: true
        }
    }

    fn is_completed(&self) -> bool {
        self.max_expansion_reached || self.remaining_letters.is_empty()
    }

    fn display(&self, with_unused: bool) -> String {
        let meta = if self.remaining_letters.is_empty() {
            "".to_string()
        } else {
            format!("(unused letters:{:?})", self.remaining_letters)
        };
        let fused = self.words.join(" ");
        if with_unused {
            format!("{} {}", fused, meta)
        } else {
            fused
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct BaseWord {
    word: String,
    chars: Vec<char>,
    char_number: usize,
}

impl BaseWord {
    fn new(word: String) -> BaseWord {
        let word_chars: Vec<char> = word.chars().collect();
        BaseWord {
            word,
            chars: word_chars.clone(),
            char_number: word_chars.len(),
        }
    }

    fn to_sentence(&self, letters: &Vec<char>) -> Sentence {
        let mut remaining_letters_for_sentence: Vec<char> = letters.clone();
        for l in self.chars.iter() {
            remove_first(&mut remaining_letters_for_sentence, |rl| rl == l);
        }
        Sentence::start(self.word.clone(), remaining_letters_for_sentence)
    }

    //TODO better be case insensitive (German words)
    fn word_can_build_from_letters(&self, letters: &Vec<char>) -> bool {
        // quick way out if the word is longer than the letters available
        if self.char_number > letters.len() {
            false
        } else {
            let mut remaining_chars = letters.to_owned();
            let mut success = true;
            self.chars.iter().for_each(|c| {
                if success && remaining_chars.contains(&c) {
                    remove_first(&mut remaining_chars, |rl| rl == c);
                } else {
                    success = false;
                }
            });
            success
        }
    }
}

pub fn sentences_for_letters(
    words: &Vec<BaseWord>,
    letters: &Vec<char>,
    sentence_length_in_words: usize,
    with_display_unused: bool,
    verbose_mode: bool,
) -> Vec<String> {
    let mut sentences_found: HashSet<Sentence> = words
        .iter()
        .map(|w| w.to_sentence(letters))
        .collect();
    let mut in_progress_sentences: HashSet<Sentence> = HashSet::new();

    // TODO try something cleaner with a fold/scan or recursion instead
    for i in 1..sentence_length_in_words {
        if verbose_mode {
            println!(
                "Progress sentences, found {:?} before iteration {}",
                sentences_found.len(),
                i
            );
        }
        // TODO could also work on several threads by splitting the sentences in chunks
        expand_sentences_found(
            &words,
            &sentences_found,
            &mut in_progress_sentences,
            verbose_mode,
        );
        // Sneaky swap for the next loop to continue where we left off
        swap(&mut sentences_found, &mut in_progress_sentences);
        // Reset progress to not analyze several times the same sentence in the next loop
        in_progress_sentences.clear();
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

fn expand_sentences_found(
    base_words: &Vec<BaseWord>,
    sentences_found: &HashSet<Sentence>,
    in_progress_sentences: &mut HashSet<Sentence>,
    verbose_mode: bool,
) {
    for sentence in sentences_found.iter() {
        let mut expanded = false;
        if !sentence.is_completed() {
            if verbose_mode {
                println!(
                    "\tSentence with {:?} remaining letters in progress",
                    &sentence.remaining_letters.len(),
                );
            }
            for w in base_words {
                if w.word_can_build_from_letters(&sentence.remaining_letters) {
                    let mut more_words: Vec<String> = Vec::new();
                    sentence
                        .words
                        .iter()
                        .for_each(|sw| more_words.push(sw.to_owned()));
                    more_words.push(w.word.clone());
                    more_words.sort_by(|a, b| a.len().cmp(&b.len()).reverse());

                    let mut remaining_letters = sentence.remaining_letters.clone();
                    for l in w.chars.iter() {
                        remove_first(&mut remaining_letters, |rl| rl == l);
                    }

                    in_progress_sentences.insert(Sentence::new(more_words, remaining_letters));
                    expanded = true
                }
            }
        }
        if !expanded {
            in_progress_sentences.insert(sentence.mark_as_max_expansion_reached());
        }
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

pub fn words_for_letters_in_file<B: BufRead>(
    reader_lines: Lines<B>,
    letters: &Vec<char>,
) -> io::Result<Vec<BaseWord>> {
    let mut found: Vec<BaseWord> = Vec::new();
    for line in reader_lines {
        let word = BaseWord::new(line?);
        if word.word_can_build_from_letters(letters) {
            found.push(word.clone());
        }
    }
    Ok(found)
}

#[cfg(test)]
extern crate quickcheck;

#[cfg(test)]
mod words_with_letters_tests {
    use super::*;

    #[test]
    fn word_can_build_from_letters_duplicate_letter() {
        let word = "happy".to_string();
        let chars = vec!['h', 'a', 'p', 'y'];
        assert_eq!(BaseWord::new(word).word_can_build_from_letters(&chars), false)
    }

    #[test]
    fn word_can_build_from_letters_happy() {
        let word = "happy".to_string();
        let chars = vec!['h', 'a', 'p', 'y', 'p'];
        assert_eq!(BaseWord::new(word).word_can_build_from_letters(&chars), true)
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
            .map(|w| BaseWord::new(w.to_string()))
            .collect();
        let sentences = sentences_for_letters(&words, &chars, 4, false, false);
        println!("{:#?}", sentences);
        assert_eq!(sentences.len(), 6);
        let expected_result: Vec<String> = vec![
            "this can run you",
            "this can you run",
            "this run can you",
            "this run you can",
            "this you can run",
            "this you run can",
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
        BaseWord::new(word).word_can_build_from_letters(&chars)
    }

    #[quickcheck]
    fn word_cant_be_built_from_different_letters(word: String) -> bool {
        let chars: Vec<char> = word.chars().collect();
        let other_word = format!("{}a", word);
        !BaseWord::new(other_word).word_can_build_from_letters(&chars)
    }
}
