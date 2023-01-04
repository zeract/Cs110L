// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    println!("random word: {}", secret_word);

    // Your code here! :)
    let mut guess_letters:Vec<char> = Vec::new();
    let mut guess_word = String::from("");
    let length = secret_word_chars.len();
    for i in 1..=length{
        guess_letters.push('-');
    }
    println!("Welcom to CS110L Hangman!");
    let mut counter = 0;
    loop{
        if counter>=NUM_INCORRECT_GUESSES{
            println!("Sorry, you ran out of guesses!");
            break;
        }else{
            let mut ok = 0;
            for i in 0..length{
                if guess_letters[i] == '-'{
                    ok = 1;
                    break;
                }
            }
            if ok==0{
                println!("Congratulations you guessed the secret word: {}",secret_word);
                break;
            }
            print!("The Word so far is ");
            for i in guess_letters.iter(){
                print!("{}",i);
            }
            print!("\n");
            println!("You have guessed the following letters: {}",guess_word);   
            
            println!("You have {} guesses left",NUM_INCORRECT_GUESSES-counter);
            print!("Please guess a letter: ");
            io::stdout()
                .flush()
                .expect("Error flushing stdout.");
            let mut guess = String::new();
            io::stdin()
                .read_line(&mut guess)
                .expect("Error reading line.");
            
            guess_word += guess.trim();
            let mut flag = 0;
            for i in 0..length{
                if secret_word_chars[i].to_string()==guess.trim(){
                    flag = 1;
                    guess_letters[i] = guess.chars().next().unwrap();
                }
            }
            if flag==0{
                counter +=1;
                println!("Sorry, that letter is not in the word");
            }
            println!("");
            println!("");
        }
    }
}
