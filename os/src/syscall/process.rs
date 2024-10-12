//! Process management syscalls
// 在 src/syscall/process.rs 文件顶部添加
use crate::task::get_current_task;
use crate::task::{get_current_task, TaskControlBlock};

use crate::{
    config::MAX_SYSCALL_NUM,
    task::{exit_current_and_run_next, get_current_task_time, get_syscall_times, suspend_current_and_run_next, TaskStatus, get_current_task}, // 确保 get_current_task 已导入
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

// TaskInfo 结构体
#[allow(dead_code)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
    pub syscall_timestamps: [usize; MAX_SYSCALL_NUM],
    pub first_scheduled_time: Option<usize>,
}

pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn example_function() {
    let current_task: &TaskControlBlock = get_current_task(); // 获取当前任务的 TaskControlBlock
    // 使用 current_task
}

pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

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

pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    if ti.is_null() {
        return -1; // 返回错误代码，指示无效的指针
    }

    trace!("kernel: sys_task_info");
    unsafe {
        let current_task = get_current_task(); // 获取当前任务的 TaskControlBlock
        (*ti).status = current_task.task_status; // 获取任务的当前状态
        (*ti).syscall_times = get_syscall_times(); // 获取系统调用次数
        (*ti).time = get_current_task_time(); // 获取任务的总运行时间
        (*ti).syscall_timestamps = current_task.syscall_timestamps; // 填充系统调用的时间戳
        (*ti).first_scheduled_time = current_task.first_scheduled_time; // 填充任务首次调度时间
    }
    0 // 返回成功
}
