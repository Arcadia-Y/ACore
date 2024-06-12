#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main() -> i32 {
    let mut x = 1.0f32;
    while x < 1e30 {
        x += 1.0;
        print!("X");
    }
    0
}
