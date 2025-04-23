//! Process management syscalls
// use riscv::paging::PTE;

use crate::{mm::{translated_byte_buffer, MapPermission, MemorySet, PTEFlags, VirtAddr}, syscall::mm_utils::access_byte, task::{change_program_brk, current_user_token, exit_current_and_run_next, suspend_current_and_run_next, with_current_memory_set, with_current_syscall_num}, timer::get_time_us};
use super::mm_utils::modify_timeval;
use crate::config::MAX_SYSCALL_NUM;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// A Packaged Type
pub struct SyscallCount {
    /// Bucket
    pub bucket: [usize; MAX_SYSCALL_NUM],
}

impl SyscallCount {
    /// Clear 
    pub fn new() -> Self {
        Self {
            bucket: [0; MAX_SYSCALL_NUM],
        }
    }
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    // -1
    // get_time_ms()
    let token = current_user_token();
    let mut byte_buffer = translated_byte_buffer(token, _ts as *const u8, core::mem::size_of::<TimeVal>());
    let usec = get_time_us();
    let timeval: TimeVal = TimeVal { sec: usec / 1_000_000 , usec };
    modify_timeval(&mut byte_buffer, timeval);
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    let token = current_user_token();
    let ptr = _id as *const u8;
    let flags = match _trace_request {
        0 => PTEFlags::R | PTEFlags::V | PTEFlags::U, 
        1 => PTEFlags::W | PTEFlags::V | PTEFlags::U,
        _ => PTEFlags::empty()
        // _ => todo!()
    };
    if let Some(byte) = access_byte(token, ptr, flags) {
        match _trace_request {
            0 => {
                // println!("byte = {}", *byte);
                *byte as isize
            }
            1 => {
                *byte = _data as u8;
                0
            }
            _ => {
                with_current_syscall_num(|syscall_called| {
                    syscall_called.bucket[_id] as isize
                })
            }
        }       
    }
    else {
        -1
    }
    //-1
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if (_port & 0x7 == 0) || (_port & (!0x7) != 0) {
        // println!("Oops");
        return -1;
    }
    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(_start + _len);
    if !start_va.aligned() {
        return -1;
    }
    let permission: MapPermission = MapPermission::from_bits((_port << 1) as u8).unwrap() | MapPermission::U;
    let ret = with_current_memory_set(|memory_set: &mut MemorySet| {
        memory_set.insert_framed_area_safely(start_va, end_va, permission)
    });
    ret
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(_start + _len);
    if !start_va.aligned() {
        return -1;
    }
    let ret = with_current_memory_set(|memory_set: &mut MemorySet| {
        memory_set.delete_framed_area(start_va, end_va)
    });
    ret
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
