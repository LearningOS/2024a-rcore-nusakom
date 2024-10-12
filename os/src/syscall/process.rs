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
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
    /// 每次系统调用的时间戳
    syscall_timestamps: [usize; MAX_SYSCALL_NUM],
    /// 任务第一次被调度的时间
    first_scheduled_time: Option<usize>,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
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

/// YOUR JOB: Finish sys_task_info to pass testcases
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    if ti.is_null() {
        return -1; // 返回错误代码，指示无效的指针
    }

    trace!("kernel: sys_task_info");
    unsafe {
        let current_task = get_current_task();  // 获取当前任务的 TaskControlBlock

        (*ti).status = current_task.task_status;  // 获取任务的当前状态
        (*ti).syscall_times = get_syscall_times();  // 获取系统调用次数
        (*ti).time = get_current_task_time();  // 获取任务的总运行时间

        // 新增部分：填充系统调用的时间戳和首次调度时间
        (*ti).syscall_timestamps = current_task.syscall_timestamps;  // 系统调用时间戳
        (*ti).first_scheduled_time = current_task.first_scheduled_time;  // 任务首次调度时间
    }
    0 // 返回成功
}
