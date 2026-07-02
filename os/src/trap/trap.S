# RISC-V RV64 General-Purpose Registers
# ==============================================================================================
# | Reg | ABI   | Category     | Definition                    | Brief Explanation            |
# |-----|-------|--------------|-------------------------------|------------------------------|
# | x0  | zero  | Special      | Constant Zero                 | Always reads as 0.           |
# | x1  | ra    | Special      | Return Address                | Address to return after call.|
# | x2  | sp    | Special      | Stack Pointer                 | Points to the top of stack.  |
# | x3  | gp    | Special      | Global Pointer                | Points to global data.       |
# | x4  | tp    | Special      | Thread Pointer                | Points to thread-local data. |
# |-----|-------|--------------|-------------------------------|------------------------------|
# | x5  | t0    | Temporary    | Temporary Register            | Caller-saved scratch.        |
# | x6  | t1    | Temporary    | Temporary Register            | Caller-saved scratch.        |
# | x7  | t2    | Temporary    | Temporary Register            | Caller-saved scratch.        |
# |-----|-------|--------------|-------------------------------|------------------------------|
# | x8  | s0/fp | Saved        | Saved Register / Frame Pointer| Callee-saved; may be FP.     |
# | x9  | s1    | Saved        | Saved Register                | Callee-saved.               |
# |-----|-------|--------------|-------------------------------|------------------------------|
# | x10 | a0    | Argument     | Argument 0 / Return Value 0   | 1st argument / return value. |
# | x11 | a1    | Argument     | Argument 1 / Return Value 1   | 2nd argument / return value. |
# | x12 | a2    | Argument     | Argument 2                    | 3rd function argument.       |
# | x13 | a3    | Argument     | Argument 3                    | 4th function argument.       |
# | x14 | a4    | Argument     | Argument 4                    | 5th function argument.       |
# | x15 | a5    | Argument     | Argument 5                    | 6th function argument.       |
# | x16 | a6    | Argument     | Argument 6                    | 7th function argument.       |
# | x17 | a7    | Argument     | Argument 7                    | 8th argument / syscall no.   |
# |-----|-------|--------------|-------------------------------|------------------------------|
# | x18 | s2    | Saved        | Saved Register                | Callee-saved.               |
# | x19 | s3    | Saved        | Saved Register                | Callee-saved.               |
# | x20 | s4    | Saved        | Saved Register                | Callee-saved.               |
# | x21 | s5    | Saved        | Saved Register                | Callee-saved.               |
# | x22 | s6    | Saved        | Saved Register                | Callee-saved.               |
# | x23 | s7    | Saved        | Saved Register                | Callee-saved.               |
# | x24 | s8    | Saved        | Saved Register                | Callee-saved.               |
# | x25 | s9    | Saved        | Saved Register                | Callee-saved.               |
# | x26 | s10   | Saved        | Saved Register                | Callee-saved.               |
# | x27 | s11   | Saved        | Saved Register                | Callee-saved.               |
# |-----|-------|--------------|-------------------------------|------------------------------|
# | x28 | t3    | Temporary    | Temporary Register            | Caller-saved scratch.        |
# | x29 | t4    | Temporary    | Temporary Register            | Caller-saved scratch.        |
# | x30 | t5    | Temporary    | Temporary Register            | Caller-saved scratch.        |
# | x31 | t6    | Temporary    | Temporary Register            | Caller-saved scratch.        |
# ==============================================================================================
# 
# Calling Convention
# ------------------
# Caller-saved : ra, t0-t6, a0-a7
# Callee-saved : s0-s11
# 
# Function Arguments
# ------------------
# a0-a7 : Function arguments
# a0-a1 : Return values
# 
# Special Registers
# -----------------
# zero : Constant 0
# ra   : Return address
# sp   : Stack pointer
# gp   : Global pointer
# tp   : Thread pointer
.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    # exchange the sp and the sscratch
    csrrw sp, sscratch, sp
    # now sp->&TrapContext in userspace, sscratch ->user stack
    # save other general purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they were saved in TrapContext
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it on the kernel stack
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # load kernel_satp into t0
    ld t0, 34*8(sp)
    # load trap_handler into t1
    ld t1, 36*8(sp)
    # move to kernel_sp
    ld sp, 35*8(sp)
    # switch to kernel space
    csrw satp, t0
    sfence.vma
    # jump to trap_handler
    jr t1

__restore:
    # a0: *TrapContext in user space(Constant); a1: user space token
    # switch to user space
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # now sp pointer to TrapContext in user space, start restoring base on it 
    # restore sstatus/spec
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw spec, t1
    # restore general purpose registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    ld sp, 2*8(sp)
    sret
