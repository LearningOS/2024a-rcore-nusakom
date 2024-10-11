//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.

/// Write syscall identifier
const SYSCALL_WRITE: usize = 64;
/// Exit syscall identifier
const SYSCALL_EXIT: usize = 93;
/// Yield syscall identifier
const SYSCALL_YIELD: usize = 124;
/// Get time syscall identifier
const SYSCALL_GET_TIME: usize = 169;
/// Task info syscall identifier
const SYSCALL_TASK_INFO: usize = 410;

mod fs;
mod process;

use fs::*;
use process::*;
use crate::task::update_syscall_times;

/// Handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> Result<isize, &'static str> {
    update_syscall_times(syscall_id); // Update syscall times
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => {
            let exit_code = args[0] as i32;
            Ok(sys_exit(exit_code))
        },
        SYSCALL_YIELD => Ok(sys_yield()?),
        SYSCALL_GET_TIME => {
            let time_val_ptr = args[0] as *mut TimeVal;
            sys_get_time(time_val_ptr, args[1]).map(|_| 0)
        },
        SYSCALL_TASK_INFO => {
            let task_info_ptr = args[0] as *mut TaskInfo;
            sys_task_info(task_info_ptr).map(|_| 0)
        },
        _ => Err("Unsupported syscall_id"),  // Return an error instead of panicking
    }
}
