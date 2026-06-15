.altmacro
.macro SAVE_SN n
  sd s\n, (\n+2)*8(a0)
.endm
.macro LOAD_SN n
  ld s\n, (\n+2)*8(a1)
.endm
  .section .text
  .globl __switch
__switch:
  # __switch(
  #     current_task_cx_ptr: *mut TaskContext,
  #     next_task_cx_ptr: *const TaskContext
  # )
  # save kernel stack of current task
  sd sp, 8(a0)
  # save
