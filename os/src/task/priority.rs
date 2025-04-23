
const BIGSTRIDE: usize = 1600000;
pub struct TaskPriority {
    prio: usize,
    stride: usize,
}

impl TaskPriority {
    pub fn new() -> Self {
        Self {
            prio: 16,
            stride: 0,
        }
    }
    pub fn set_prio(&mut self, prio: usize) {
        self.prio = prio;
    }
    pub fn step_one(&mut self) {
        self.stride += BIGSTRIDE / self.prio;
    }
    pub fn get_stride(&self) -> usize {
        self.stride
    }
}