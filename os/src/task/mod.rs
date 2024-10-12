//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::config::{MAX_APP_NUM, MAX_SYSCALL_NUM};
use crate::loader::{get_num_app, init_app_cx};
use crate::sync::UPSafeCell;
use crate::timer::get_time_ms;
use lazy_static::*;
use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: UPSafeCell<TaskManagerInner>,
}

/// Inner of Task Manager
pub struct TaskManagerInner {
    /// task list
    tasks: [TaskControlBlock; MAX_APP_NUM],
    /// id of current `Running` task
    current_task: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
            task_syscall_times: [0; MAX_SYSCALL_NUM],
            syscall_timestamps: [0; MAX_SYSCALL_NUM], // 初始化为 0
            task_time: 0,
            first_scheduled_time: None, // 初始化为 None
        }; MAX_APP_NUM];
        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_cx = TaskContext::goto_restore(init_app_cx(i));
            task.task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}
impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch3, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        task0.task_time = get_time_ms();
        
        // 如果是第一次被调度，记录第一次调度的时间
        if task0.first_scheduled_time.is_none() {
            task0.first_scheduled_time = Some(get_time_ms());
        }
    
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }    

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.tasks[current].task_time = get_time_ms() - inner.tasks[current].task_time;
            inner.tasks[next].task_time = get_time_ms();
            
            // 如果是第一次被调度，记录第一次调度的时间
            if inner.tasks[next].first_scheduled_time.is_none() {
                inner.tasks[next].first_scheduled_time = Some(get_time_ms());
            }
    
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            panic!("All applications completed!");
        }
    }
    
    fn get_syscall_times(&self) -> [u32; MAX_SYSCALL_NUM] {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].task_syscall_times
    }

    fn get_current_task_time(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].task_time
    }

    fn update_syscall_times(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_syscall_times[syscall_id] += 1;
        
        // 记录系统调用的时间戳
        inner.tasks[current].syscall_timestamps[syscall_id] = get_time_ms();
    }
    /// 获取当前任务的系统调用时间戳
    fn get_syscall_timestamps(&self) -> [usize; MAX_SYSCALL_NUM] {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].syscall_timestamps
    }

    /// 获取当前任务第一次调度的时间
    fn get_first_scheduled_time(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].first_scheduled_time
    }
}

/// 获取当前任务的系统调用时间戳
pub fn get_syscall_timestamps() -> [usize; MAX_SYSCALL_NUM] {
    TASK_MANAGER.get_syscall_timestamps()
}

/// 获取当前任务第一次被调度的时间
pub fn get_first_scheduled_time() -> Option<usize> {
    TASK_MANAGER.get_first_scheduled_time()
}

/// Run the first task in task list.
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// Switch current `Running` task to the task we have found,
/// or there is no `Ready` task and we can exit with all applications completed
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// Change the status of current `Running` task into `Ready`.
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// Change the status of current `Running` task into `Exited`.
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

/// Get the syscall times of current task.
pub fn get_syscall_times() -> [u32; MAX_SYSCALL_NUM] {
    TASK_MANAGER.get_syscall_times()
}

/// Get the total running time of current task.
pub fn get_current_task_time() -> usize {
    TASK_MANAGER.get_current_task_time()
}

/// Update the syscall times of current task.
pub fn update_syscall_times(syscall_id: usize) {
    TASK_MANAGER.update_syscall_times(syscall_id);
}