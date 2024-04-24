#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

static mut f: [i32; 2801] = [0; 2801];

#[no_mangle]
unsafe fn main() -> i32 {
	let a: i32 = 10000;
	let mut b: i32 = 0;
	let mut c: i32 = 2800;
	let mut d: i32 = 0;
	let mut e: i32 = 0;
	let mut g: i32 = 0;
	while b-c != 0 {
		f[b as usize] = a/5;
		b += 1;
	}
	loop {
		d = 0;
		g = c*2;
		if g == 0 {
			break;
		}
		b = c;
		loop {
			d = d+f[b as usize]*a;
			g -= 1;
			f[b as usize] = d % g;
			d = d/g;
			g -= 1;
			b -= 1;
			if b == 0 {
				break;
			}
			d = d*b;
		}
		c -= 14;
		print!("{}", e+d/a);
		e = d%a;
	}
	print!("\n");
	0
}
