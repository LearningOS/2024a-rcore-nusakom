.altmacro
.macro SAVE_SN n
    sd s\n, (\n + 2) * 8(a0)
.endm
.macro LOAD_SN n
    ld s\n, (\n + 2) * 8(a1)
.endm

.section .text
.globl __switch
__switch:
    # __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext)
    
    # Save the kernel stack pointer of the current task
    sd sp, 8(a0)          # Save current stack pointer

    # Save return address and saved registers (s0 to s11)
    sd ra, 0(a0)          # Save return address
    .set n, 0
    .rept 12
        SAVE_SN %n        # Save s0 to s11
        .set n, n + 1
    .endr

    # Restore return address and saved registers for the next task
    ld ra, 0(a1)          # Restore return address
    .set n, 0
    .rept 12
        LOAD_SN %n        # Restore s0 to s11
        .set n, n + 1
    .endr

    # Restore the kernel stack pointer of the next task
    ld sp, 8(a1)          # Restore stack pointer

    # Return to the next task's context
    ret