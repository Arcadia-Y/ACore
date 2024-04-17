use crate::{mm::page_table::get_user_byte_buffer, print, task::current_user_satp};

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffer = get_user_byte_buffer(current_user_satp(), buf, len);
            print!("{}", core::str::from_utf8(buffer.as_slice()).unwrap());
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
