//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, TASK_MANAGER},
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
    /// Task status in its lifecycle
    status: TaskStatus,
    /// The number of system calls made by the task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of the task in microseconds
    time: usize,
}

/// Task exits and submits an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// Current task yields CPU resources to other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// Get time with seconds and microseconds
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

/// Retrieve current task information and pass it to user space
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");

    // Obtain a reference to the current task from the task manager
    let current_task = TASK_MANAGER.get_current_task();  // Assumes there's a method to get the current task

    // Construct a TaskInfo struct, filling in the task details
    let task_info = TaskInfo {
        status: current_task.task_status,     // The current task's status
        syscall_times: current_task.syscall_times,  // Number of system calls made by the task
        time: get_time_us() - current_task.start_time, // Total running time (current time - task start time)
    };

    // Use unsafe to copy TaskInfo struct to user space
    unsafe {
        *ti = task_info;
    }

    0 // Return 0 for success
}
