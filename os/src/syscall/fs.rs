use crate::{io::console::getchar, mm::page_table::{get_user_byte_buffer, translate_refmut}, print, task::processor::current_user_satp};

const FD_STDIN: usize = 0;
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

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            assert_eq!(len, 1, "only support sys_read with len=1 from STDIN");
            let c = getchar();
            let user_buf = translate_refmut(current_user_satp(), buf);
            *user_buf = c;
            1
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}
