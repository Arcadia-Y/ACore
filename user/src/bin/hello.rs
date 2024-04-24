#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use user_lib::yield_;

#[no_mangle]
fn main() -> i32 {
    println!("[user] Hello, world!");
    yield_();
    println!("[user] Hello world!");
    0
}
