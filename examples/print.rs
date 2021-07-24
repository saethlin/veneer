#![no_std]
#![feature(lang_items, alloc_error_handler, start)]

veneer::prelude!();

fn main() {
    print!("print-test ");
    print!("\n");
    println!("print-test if {} == {}", 1, 1);

    eprint!("eprint-test ");
    eprint!("\n");
    eprintln!("eprint-test if {} == {}", 1, 1);
}
