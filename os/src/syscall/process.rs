//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        exit_current_and_run_next, get_syscall_times, get_task_time, suspend_current_and_run_next,
        TaskStatus,
    },
    timer::get_time_us,
};
use crate::task::TaskStatus::Running;

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
    status: TaskStatus,// 任务状态
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],// 记录每个系统调用的调用次数
    /// Total running time of task
    time: usize,// 任务的总运行时间
}
impl TaskInfo {
    pub fn modify_task_info(task_info:*mut Self)->Option<()>{
        unsafe{
            (*task_info).status=Running; // 设置任务状态为 Running
            (*task_info).syscall_times=get_syscall_times();// 获取系统调用次数
            (*task_info).time=get_task_time();// 获取任务的总运行时间
        }
        Some(())// 成功修改后返回 Some
    }
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
    trace!("kernel: sys_task_info");
    match TaskInfo::modify_task_info(ti){
        None => -1,// 如果信息填充失败，返回 -1
        Some(_) => 0// 成功填充任务信息，返回 0
    }
}
