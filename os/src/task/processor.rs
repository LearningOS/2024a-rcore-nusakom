//!Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.

use super::__switch;
use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock};
use crate::config::PAGE_SIZE;
use crate::mm::{MapPermission, VirtAddr};
use crate::sync::UPSafeCell;
use crate::syscall::process::TaskInfo;
use crate::timer::get_time_ms;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;

/// Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,

    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }

    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }

    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            // release coming task_inner manually
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            warn!("no tasks available in run_tasks");
        }
    }
}

/// Get current task through take, leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get the current user token(addr of page table)
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.get_user_token()
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

/// Get current task info
pub fn current_task_info() -> TaskInfo {
    let current_task_control_block = current_task().unwrap();
    let current_task = current_task_control_block.inner.exclusive_access();

    TaskInfo {
        status: current_task.task_status,
        syscall_times: current_task.task_syscall_trace,
        time: {
            let start = current_task.task_start_time;
            let end = current_task.task_lastest_syscall_time;
            end - start
        },
    }
}

/// Allocate memory
pub fn allocate_memory(start: usize, len: usize, port: usize) -> isize {
    // check
    if start % PAGE_SIZE != 0 {
        return -1;
    }

    if port & !0x7 != 0 || port & 0x7 == 0 {
        return -1;
    }

    let start_address = VirtAddr::from(start);
    let end_address = VirtAddr::from(start + len);

    let current_task_control_block = current_task().unwrap();
    let mut current_task = current_task_control_block.inner.exclusive_access();

    if current_task
        .memory_set
        .include_allocated(start_address, end_address)
    {
        return -1;
    }

    let permissions = MapPermission::from_bits((port as u8) << 1).unwrap() | MapPermission::U;

    current_task
        .memory_set
        .insert_framed_area(start_address, end_address, permissions);

    0
}

/// Free memory
pub fn free_memory(start: usize, len: usize) -> isize {
    if start % PAGE_SIZE != 0 {
        return -1;
    }

    let start_address = VirtAddr::from(start);
    let end_address = VirtAddr::from(start + len);

    if !start_address.aligned() {
        return -1;
    }

    if !end_address.aligned() {
        return -1;
    }

    let current_task_control_block = current_task().unwrap();
    let mut current_task = current_task_control_block.inner.exclusive_access();

    current_task
        .memory_set
        .free_framed_area(start_address, end_address);

    0
}

/// Update task info
pub fn update_task_info(syscall_id: usize) {
    let current_task_control_block = current_task().unwrap();
    let mut current_task = current_task_control_block.inner.exclusive_access();

    current_task.task_lastest_syscall_time = get_time_ms();
    current_task.task_syscall_trace[syscall_id] += 1;
}

///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}

/// Set task priority
pub fn set_priority(priority: isize) -> isize {
    if priority < 2 {
        return -1;
    }

    let cpu_cur_task = current_task().unwrap();
    let mut task_inner = cpu_cur_task.inner_exclusive_access();
    task_inner.priority = priority;

    task_inner.priority
}