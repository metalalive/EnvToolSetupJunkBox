use std::io;
use std::cmp::Ordering;
use rand::Rng;

fn get_std_in_rawstring() -> (String, usize) {
    let g0 = String::new(); // immutable
    let mut g1 = g0; // g0 is moved to g1, now the string is mutable, g0 is invalid
    // there could still be 2 mutable references in the same scope ONLY if 
    // one is no longer used before another in your program
    let _gr0 = & mut g1;
    let _gr1 = & g1;
    let _gr2 = & mut g1;
    let _gr3 = & g1;
    // if succeed, the buffer includes read string followed by new-line char `\n`
    // , nread is number of characters read + 1 new-line char
    let nread = io::stdin()
        // a type which represents a handle to standard input of your terminal
        .read_line(& mut g1)
        // reference, similar concept in C, make it mutable
        .expect("failed to read line, standard input error");
    (g1, nread) // move string `g` to return value
}


fn single_round(_answer:u16) -> bool {
    let mut out :bool = false;
    // TODO, figure out smart pointer below, and how to delete it
    let (myg, nread) = get_std_in_rawstring();
    // shadow previous value of `myg` variable, but differemt types, still reuse the same name
    let myg:u16 = match  myg .trim() .parse() {
        Ok(num) => num,
        Err(e) => {
           println!("receive invalid number, reason:{e}");
           0x0 // assign 0 to myg
        } // avoid potential callback hell
    };
    if myg > 0 {
        print!("I guess {myg}, nread:{nread}, ");
        match myg.cmp(&_answer) {
            Ordering::Less    => println!("too small"),
            Ordering::Greater => println!("too big"),
            Ordering::Equal   => {out = true;},
        }
    }
    out
} // end of single_round


fn main() {
    println!("--- guessing-number game starts ---");
    // compiler implicitly treats it as `String` type, it provides function `new()`
    let answer:u16 = rand::thread_rng().gen_range(1..=100); // immutable
    let mut guess_correct :bool = false;
    let nguesses = 1..8; // tuple ??
    // whenever error is thrown Rust simply reports message and then breaks the loop
    for _ in nguesses {
        print!("enter a number:"); // print after input is given, why ?
        guess_correct = single_round(answer);
        if guess_correct {break;}
    } // end of loop
    if guess_correct {
        println!("you win");
    } else {
        println!("you lose, expected answer, {answer}");
    }
} // end of main
