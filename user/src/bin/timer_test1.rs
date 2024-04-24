#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    let mut x = 1;
    while x != 0 {
        if x % 2 == 1 {
            x += 1;
        } else {
            x -= 1;
        }
        print!("X");
    }
    0
}
