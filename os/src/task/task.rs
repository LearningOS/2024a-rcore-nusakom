use super::TaskContext;
use std::time::{SystemTime, UNIX_EPOCH};

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in its lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// The task's syscall count (how many syscalls it has performed)
    pub syscall_times: u32,
    /// The current syscall (if any)
    pub current_syscall: Option<u32>,
    /// The time when the task was first scheduled (in milliseconds since epoch)
    pub start_time: u64,
}

impl TaskControlBlock {
    /// Get the current time in milliseconds since UNIX_EPOCH
    fn get_current_time_ms() -> u64 {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
        since_the_epoch.as_millis() as u64
    }

    /// Initialize the task control block with default values
    pub fn new(task_cx: TaskContext) -> Self {
        TaskControlBlock {
            task_status: TaskStatus::UnInit,
            task_cx,
            syscall_times: 0,
            current_syscall: None,
            start_time: Self::get_current_time_ms(), // Initialize the start time
        }
    }

    /// Update the syscall count and set the current syscall
    pub fn record_syscall(&mut self, syscall_id: u32) {
        self.syscall_times += 1;
        self.current_syscall = Some(syscall_id);
    }

    /// Get the time since the task was first scheduled, in milliseconds
    pub fn get_time_since_first_schedule(&self) -> u64 {
        let current_time = Self::get_current_time_ms();
        current_time - self.start_time
    }
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
