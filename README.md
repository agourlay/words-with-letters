# words with letters

CLI to generate words and sentences from a given set of letters.

The base words are found in a dictionary file which must contain one word by line. 

The dictionary file is not embedded in the application itself but this repository contains 4 dictionaries (english, french, german and spanish) taken from https://github.com/lorenbrichter/Words.

## Usage

```
words-with-letters -h
words-with-letters 0.1.0
Arnaud Gourlay <arnaud.gourlay@gmail.com>
Making words with letters

USAGE:
    words-with-letters --dictionaryFile <dictionaryFile> --letters <letters> --sentenceLength <sentenceLength>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --dictionaryFile <dictionaryFile>    dictionary file
    -l, --letters <letters>                  letters to use
    -s, --sentenceLength <sentenceLength>    sentence length in words
```

## Examples

```
words-with-letters --dictionaryFile dictionaries/en.txt --letters margana  --sentenceLength 1
Found 51 results -- listed sorted by length:

anagram
amarna (unused letters:['g'])
ragman (unused letters:['a'])
ranga (unused letters:['m', 'a'])
grama (unused letters:['n', 'a'])
argan (unused letters:['m', 'a'])
manga (unused letters:['r', 'a'])
agama (unused letters:['r', 'n'])
grana (unused letters:['m', 'a'])
naga (unused letters:['m', 'r', 'a'])
etc...
```

```
words-with-letters --dictionaryFile dictionaries/fr.txt --letters allezlesbleus  --sentenceLength 3
Found 125 base words from the dictionary using the input letters ['a', 'l', 'l', 'e', 'z', 'l', 'e', 's', 'b', 'l', 'e', 'u', 's']
Building sentences with 3 words, it might take a while depending on your settings...
Found 7503 results -- listed sorted by length:

salez belle lus
allez belle sus
allez bulle ses
salez lebel lus
sellez label su
seuls allez bel
allez bleu sels
sellez lues bal
bleus allez sel
allez elles bus
etc...
```

By default, all results are printed therefore it might makes sense to redirect the output to a file for convenience.

```
words-with-letters --dictionaryFile dictionaries/de.txt --letters lassdiesonnerein  --sentenceLength 3 > words.txt
```

## Internals

It can get slow on large inputs as the implementation does not try yet to be smart.

## Installation from source

Install [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) and then run the following command within the `dlm` directory.

`cargo install --path=.`

Make sure to have `$HOME/.cargo/bin` in your path.