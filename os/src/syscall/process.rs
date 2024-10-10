//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{exit_current_and_run_next, suspend_current_and_run_next, TaskStatus},
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
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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

/// Get current task information
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    // 获取任务管理器的引用
    let task_manager = crate::task::TASK_MANAGER;
    let current_task_info = unsafe { task_manager.inner.exclusive_access() };
    let current_task_index = current_task_info.current_task;
    let current_task = &current_task_info.tasks[current_task_index];

    // 更新系统调用次数
    current_task.syscall_times[410] += 1; // syscall ID: 410

    // 填充任务信息
    unsafe {
        (*ti).status = TaskStatus::Running; // 当前任务状态为 Running
        (*ti).syscall_times.copy_from_slice(&current_task.syscall_times); // 复制系统调用次数
        (*ti).time = get_time_us() - current_task.start_time; // 计算运行时间，单位为微秒
    }

    0 // 成功返回 0
}
