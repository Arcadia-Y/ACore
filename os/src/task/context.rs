#[repr(C)]
pub struct TaskContext {
    pub ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    pub fn new(ra: usize, sp: usize) -> Self {
        Self{
            ra,
            sp,
            s: [0; 12],
        }
    }
}
