//! Types related to task management

use crate::config::MAX_SYSCALL_NUM;

use super::TaskContext;

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// 任务的生命周期状态
    pub task_status: TaskStatus,
    /// 任务上下文
    pub task_cx: TaskContext,
    /// 系统调用次数数组
    pub task_syscall_times: [u32; MAX_SYSCALL_NUM],
    /// 每次系统调用的时间戳（记录系统调用时刻距离任务第一次调度的时间）
    pub syscall_timestamps: [usize; MAX_SYSCALL_NUM],
    /// 任务的总运行时间
    pub task_time: usize,
    /// 任务第一次被调度的时间
    pub first_scheduled_time: Option<usize>,
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}

// 在 task.rs 文件中定义
pub fn get_current_task_time() -> usize {
    // 返回当前任务的运行时间
    // 实现细节根据你的需求
}

pub fn get_syscall_times() -> [u32; MAX_SYSCALL_NUM] {
    // 返回当前任务的系统调用次数
    // 实现细节根据你的需求
}
