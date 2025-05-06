use alloc::vec::Vec;
use alloc::vec;
use crate::sync::UPSafeCell;

// trait DeadlockChecker {
//     fn add_task(&self, tid: usize);
//     fn add_lock(&self, lid: usize, capacity: usize);
//     fn try_lock(&self, tid: usize, lid: usize, cost: usize);
//     fn try_unlock(&self, tid: usize, lid: usize, cost: usize);
// }

/// check mutex and Semaphore
pub struct DeadlockChecker {
    checked: bool,
    inner: UPSafeCell<CheckerInner>,
}

pub struct CheckerInner {
    available: Vec<isize>,
    allocation: Vec<Vec<isize>>,
    need: Vec<Vec<isize>>,
}

impl DeadlockChecker {
    /// new DeadlockChecker
    pub fn new() -> Self {
        Self {
            checked: false,
            inner: unsafe{ UPSafeCell::new(CheckerInner { 
                available: Vec::new(), allocation: vec![Vec::new(); 1], need: vec![Vec::new(); 1],
            })}
        }
    }

    /// activate checker
    pub fn activate(&mut self, checked: bool) {
        self.checked = checked;
    }

    /// add new task
    pub fn add_task(&self, tid: usize) {
        let mut inner = self.inner.exclusive_access();
        while tid >= inner.allocation.len() {
            inner.allocation.push(Vec::new());
            inner.need.push(Vec::new());
        }
        while inner.allocation[tid].len() < inner.available.len() {
            inner.allocation[tid].push(0);
            inner.need[tid].push(0);
        }
        for i in 0..inner.allocation[tid].len() {
            inner.allocation[tid][i] = 0;
            inner.need[tid][i] = 0;
        }
    }
    /// add new lock
    pub fn add_lock(&self, lid: usize, capacity: usize) {
        let mut inner = self.inner.exclusive_access();
        while lid >= inner.available.len() {
            inner.available.push(0);
        }
        inner.available[lid] = capacity as isize;
        let task_count = inner.allocation.len();
        for tid in 0..task_count {
            while inner.allocation[tid].len() <= lid {
                inner.allocation[tid].push(0);
                inner.need[tid].push(0);
            }
        }
    }
    /// actually acquire this lock
    pub fn lock(&self, tid: usize, lid: usize) {
        let mut inner = self.inner.exclusive_access();
        inner.available[lid] -= 1;
        inner.allocation[tid][lid] += 1;
        inner.need[tid][lid] -= 1;
    }
    /// only when you can acquire this lock
    pub fn try_lock(&self, tid: usize, lid: usize) -> bool {
        if self.checked == false {
            return true;
        }

        let mut inner = self.inner.exclusive_access();
        let task_count = inner.allocation.len();
        let lock_count = inner.available.len();

        inner.need[tid][lid] += 1;

        inner.available[lid] -= 1;
        inner.allocation[tid][lid] += 1;
        inner.need[tid][lid] -= 1; 

        let mut work: Vec<isize> = inner.available.clone();
        let mut finish: Vec<bool> = vec![false; inner.allocation.len()];
        
        let mut founded: bool = true;
        while founded {
            founded = false;
            for i in 0..task_count {
                if finish[i] == true {
                    continue;
                }
                let mut can_completed: bool = true;
                for j in 0..lock_count {
                    if inner.need[i][j] > work[j] {
                        can_completed = false;
                        break;
                    }
                }
                if can_completed == true {
                    for j in 0..lock_count {
                        work[j] += inner.allocation[i][j];
                    }
                    finish[i] = true;
                    founded = true;
                    break;
                }
            }
        }
        
        let mut ret = true;
        for i in 0..finish.len() {
            if finish[i] == false {
                ret = false;
                break;
            }
        }

        inner.available[lid] += 1;
        inner.allocation[tid][lid] -= 1;
        inner.need[tid][lid] += 1; 
        
        if ret == false {
            inner.need[tid][lid] -= 1;
        }

        ret
    }
    /// actually not to lock
    pub fn unlock(&self, tid: usize, lid: usize) {
        let mut inner = self.inner.exclusive_access();
        inner.available[lid] += 1;
        inner.allocation[tid][lid] -= 1;
    }

    /// consume resources
    pub fn semaphore(&self, lid: usize) {
        let mut inner = self.inner.exclusive_access();
        inner.available[lid] -= 1;
    }
    
    /// recycle resources
    pub fn unsemaphore(&self, tid: usize, lid: usize) {
        let mut inner = self.inner.exclusive_access();
        inner.need[tid][lid] -= 1;
    }

    /// only when you can acquire this lock
    pub fn try_semaphore(&self, tid: usize, lid: usize) -> bool {
        if self.checked == false {
            return true;
        }

        let mut inner = self.inner.exclusive_access();
        let task_count = inner.allocation.len();
        let lock_count = inner.available.len();

        inner.need[tid][lid] += 1;

        inner.available[lid] -= 1;
        inner.need[tid][lid] -= 1; 

        let work: Vec<isize> = inner.available.clone();
        let mut finish: Vec<bool> = vec![false; inner.allocation.len()];
        
        let mut founded: bool = true;
        while founded {
            founded = false;
            for i in 0..task_count {
                if finish[i] == true {
                    continue;
                }
                let mut can_completed: bool = true;
                for j in 0..lock_count {
                    if inner.need[i][j] > work[j] {
                        can_completed = false;
                        break;
                    }
                }
                if can_completed == true {
                    finish[i] = true;
                    founded = true;
                    break;
                }
            }
        }
        
        let mut ret = true;
        for i in 0..finish.len() {
            if finish[i] == false {
                ret = false;
                break;
            }
        }

        inner.available[lid] += 1;
        inner.need[tid][lid] += 1; 
        
        if ret == false {
            inner.need[tid][lid] -= 1;
        }

        ret
    }

    /// more resources
    pub fn more_capacity(&self, lid: usize) {
        let mut inner = self.inner.exclusive_access();
        inner.available[lid] += 1;
    }
}