//! Implementation of system calls
//!
//! This module provides the implementation for system calls that userspace applications
//! can invoke via the `ecall` instruction. When a user application makes a system call,
//! it triggers an 'Environment call from U-mode' exception, which is handled in
//! [`crate::trap::trap_handler`]. Each system call is implemented as a separate function,
//! prefixed with `sys_` followed by the name of the system call.

/// System call identifiers
const SYS_WRITE: usize = 64;      // Write syscall
const SYS_EXIT: usize = 93;       // Exit syscall
const SYS_YIELD: usize = 124;     // Yield syscall
const SYS_GET_TIME: usize = 169;  // Get time syscall
const SYS_TASK_INFO: usize = 410; // Task information syscall

mod fs;
mod process;

use fs::*;
use process::*;
use crate::timer::get_time;
use crate::task::TaskStatus;
use crate::task::update_syscall_times;
use crate::task::{update_syscall_times, TaskStatus};

/// Handles the system call exception based on the `syscall_id` and its arguments.
///
/// # Arguments
/// * `syscall_id`: The identifier for the system call being invoked.
/// * `args`: An array of arguments passed to the system call.
///
/// # Returns
/// Returns an `isize`, which may represent a return value or an error code.
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    update_syscall_times(syscall_id);

    match syscall_id {
        // handle different syscall ids here
        0 => sys_exit(args[0] as i32),
        1 => sys_yield(),
        2 => sys_get_time(args[0] as *mut TimeVal, args[1]),
        3 => sys_task_info(args[0] as *mut TaskInfo),
        // add more syscalls as necessary
        _ => {
            // Handle unknown syscall
            trace!("kernel: unknown syscall_id: {}", syscall_id);
            -1
         }
    }
}