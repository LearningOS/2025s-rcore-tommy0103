//! Implementation of [`Memory_function`].

use crate::mm::{PTEFlags, PageTable, VirtAddr};

use super::process::TimeVal;
use alloc::vec::Vec;

pub fn modify_timeval(byteslices: &mut Vec<&'static mut [u8]>, t: TimeVal) {
    if byteslices.get(0).map_or(false, |bytes| bytes.len() >= core::mem::size_of::<TimeVal>()) {
        let bytes = &mut byteslices[0];
        let timeval_ptr = bytes.as_mut_ptr() as *mut TimeVal;
        unsafe {
            let timeval = &mut *timeval_ptr;
            timeval.sec = t.sec;
            timeval.usec = t.usec;
        }
    } else if byteslices.len() >= 2 {
        let sec_ptr = byteslices[0].as_mut_ptr() as *mut usize;
        let usec_ptr = byteslices[1].as_mut_ptr() as *mut usize;
        
        unsafe {
            *sec_ptr = t.sec;
            *usec_ptr = t.usec;
        }
    }
}

pub fn access_byte(token: usize, ptr: *const u8, flags: PTEFlags) -> Option<&'static mut u8> {
    let page_table = PageTable::from_token(token);
    let va = VirtAddr::from(ptr as usize);
    let vpn = va.floor();
    if let Some(pte) = page_table.translate_if_available(vpn, flags) {
        let ppn = pte.ppn();
        let offset = va.page_offset();
        Some(&mut ppn.get_bytes_array()[offset])
    }
    else {
        None
    }
}


