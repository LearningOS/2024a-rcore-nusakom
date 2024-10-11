//! Process management syscalls 
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{exit_current_and_run_next, get_current_task_time, get_syscall_times, suspend_current_and_run_next, TaskStatus},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in its life cycle
    status: TaskStatus,
    /// The number of syscalls called by the task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of the task
    time: usize,
}

impl TaskInfo {
    /// Modifies the task info structure with current task data
    pub fn modify_task_info(task_info: *mut Self) -> Result<(), &'static str> {
        if task_info.is_null() {
            return Err("Null pointer for TaskInfo");
        }
        unsafe {
            (*task_info).status = Running;
            (*task_info).syscall_times = get_syscall_times();
            (*task_info).time = get_current_task_time();
        }
        Ok(())
    }
}

/// Task exits and submits an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// Current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// Get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// Retrieves task information and fills the `TaskInfo` struct
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    match TaskInfo::modify_task_info(ti) {
        Ok(_) => 0,    // Return 0 for success
        Err(_) => -1,  // Return -1 for failure (e.g., null pointer)
    }
}
