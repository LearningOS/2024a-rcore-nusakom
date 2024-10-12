// task.rs

pub mod task;

use crate::config::MAX_SYSCALL_NUM;

// TaskControlBlock 定义
#[repr(C)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_time: usize,
    pub syscall_timestamps: [usize; MAX_SYSCALL_NUM],
    pub first_scheduled_time: Option<usize>,
    pub task_syscall_times: [u32; MAX_SYSCALL_NUM],
}

// 确保你的 get_current_task 函数已经实现并返回正确的类型
pub fn get_current_task() -> &'static TaskControlBlock {
    // 示例实现，替换为实际获取当前任务的逻辑
    unsafe { &*CURRENT_TASK }
}

pub fn get_current_task_time() -> usize {
    let current_task = get_current_task();
    current_task.task_time // 返回任务的总运行时间
}

pub fn get_syscall_times() -> [u32; MAX_SYSCALL_NUM] {
    let current_task = get_current_task();
    current_task.task_syscall_times // 返回系统调用次数数组
}
