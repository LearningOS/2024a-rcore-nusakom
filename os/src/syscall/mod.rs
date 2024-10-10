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
        SYS_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYS_EXIT => sys_exit(args[0] as i32),
        SYS_YIELD => sys_yield(),
        SYS_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYS_TASK_INFO => sys_task_info(args[0] as *mut TaskInfo),
        _ => {
            // Return an error code for unsupported syscalls instead of panicking
            println!("Unsupported syscall_id: {}", syscall_id);
            -1 // Error code indicating an unsupported syscall
        }
    }
}