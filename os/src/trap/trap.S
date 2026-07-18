# NOTE:!!important registers in TRAP
# stvec (Supervisor Trap Vector):TrapHandler base addr and mode
# satp (Supervisor Translation and Protection Register):page table base addr, and translation mode(e.g. SV39)
# sepc (Supervisor Exception Program Counter Register):Holds the "virtual addr" of the instrucion when have interrupts/exceptions
# scause (Supervisor Cause Register):Indentify whether Exception/Interrupts and its cause code
# stval (Supervisor Trap Value Register):Holds additional Exception/Interrupts info(e.g faulting virtual address).
# sstatus (Supervisor Status Register):Holds the operating state of Supervisor-mode(e.g. interrupt-enable bits, previous privilege informations...).
# sscratch (Supervisor Scratch Register):Dedicated for use by supervisor softwares.
# sie (Supervisor Interrupt Enable Register):Controls which Supervisor Interrrupts are enabled.
# sip (Supervisor Interrupts Pending Register):Indicating pending supervisor interrupts.
# NOTE:!!important instructions in TRAP
# "sret" (instruction): (Supervisor Return Instruction):Returns from a trap taken into Supervisor mode by restoring the Privillege mode and interrupt-enable status from "sstatus" and resuming execution at address in "sepc";
# "jr [register]"(instruction): (Jump Register): let PC point to the register(normally a function addr).
# "sfence.vma":(TLB flush)tells the CPU to stop using old virtual-to-physical address translations and reload them from the page table
# "fence.i":(I-cache flush)makes the cpu fetch the updated instructions from memory
# NOTE:!!important cheatsheets of S-mode riscv asm:
# load, read(把东西搬进CPU(Register))=> "<-"(Register<-Memory)
# write, store(把东西搬进内存(Memory))=> "->"(Register->Memory)
# 算数运算只能发生在Registers之间,Memorys不能直接参与运算,所以只能read(load)->calculate->write(store)
# NOTE: Integer Registers
# "Caller Saved":(main():caller,foo():callee):callee改了用了不会还原给caller,要caller自己saved
# "Callee Saved":(main():caller,foo():callee):callee改了用了必须还原给caller,不用caller自己saved

# TODO:2026.7.17:plan:7.18重新分析作图理解这个奇怪的trap.S文件,写学相关笔记,学习Trampoline机制
.altmacro
.macro SAVE_GP n # declear a macro
    sd x\n, \n*8(sp) # save x[n]->([n]*8(offset)+sp)
.endm
.macro LOAD_GP n # declear a macro
    ld x\n, \n*8(sp) # load ([n]*8(offset)+sp)->x[n]
.endm
    .section .text.trampoline # set a section called trampoline
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    # exchange the sp and the sscratch
    ##[repr(C)]
    # pub struct TrapContext {
    #     /// general regs[0..31]
    #     pub x: [usize; 32],
    #     /// CSR sstatus
    #     pub sstatus: Sstatus,
    #     /// CSR spec
    #     pub sepc: usize,
    #     /// Addr of Page Table
    #     pub kernel_satp: usize,
    #     /// kernel_stack
    #     pub kernel_sp: usize,
    #     /// Addr of trap_handler function
    #     pub trap_handler: usize,
    # }
    csrrw sp, sscratch, sp // store the current Stack Pointer to the sscratch
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
    # 把当前的sstatus和sepc寄存器的内容存入trapcontext中
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it on the kernel stack
    # 把user stack pointer存入trapcontext中
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # load kernel_satp into t0
    # 把kernel页表寄存器存入t0
    ld t0, 34*8(sp)
    # load trap_handler into t1
    # 把traphandler函数的地址load到t1中去
    ld t1, 36*8(sp)
    # move to kernel_sp
    # 把sp指向kernel_sp,准备从kernel_sp开始执行
    ld sp, 35*8(sp)
    # switch to kernel space
    # 把trapcontext的页表数据从t0写入satp
    # NOTE: riscv64汇编--csr汇编指令
    # 语法特殊:e.g."csrw(r) [csr] [源寄存器/目标寄存器]"
    # 语义:let the [csr] read(write) [源寄存器/目标寄存器];
    # 补充:"csrrw rd, csr, rs"
    # 语义:(atomic instruction) let rd = csr(旧值)->let csr = rs(新值)
    # p.s."rd == RegisterDestination","rs = RegisterSource"
    csrw satp, t0
    sfence.vma # 刷新tlb,防止旧页表缓存干扰
    # jump to trap_handler ()
    # let PC point to the t1(a trap_handler function's addr)
    jr t1

__restore:
    # a0: *TrapContext in user space(Constant); a1: user space token
    # switch to user space
    # NOTE: ?what is in the "a1" register
    # unsafe {
    #     asm!(
    #         "fence.i",
    #         "jr {restore_va}",      // jump to new addr of __restore asm function
    #         restore_va = in(reg) restore_va,
    #         in("a0") trap_cx_ptr,   // a0 = virt addr of Trap Context
    #         in("a1") user_satp,     // a1 = phy addr of user page table
    #         options(noreturn)
    #     );
    # }
    csrw satp, a1 # 我们发现a1是在trap_return()函数中传入当前用户的user_satp
    sfence.vma # 修改页表后刷新TLB缓存
    csrw sscratch, a0 # sscratch存入trap_cx_ptr,指向trapContext
    mv sp, a0 # <=> sp = a0,即sp指向trapContext(userspace)的地址
    # now sp pointer to TrapContext in user space, start restoring base on it 
    # restore sstatus/sepc
    # 恢复sstatus和sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # restore general purpose registers except x0/sp/tp
    # 恢复通用寄存器,除了几个不用恢复的x0(本来就是0),sp(后面再恢复),tp(线程指针,暂时用不到)
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    # 恢复用户栈指针,一切恢复完成
    ld sp, 2*8(sp)
   # 读取sstatus和sepc等csr之后让machine回到user模式
    sret
    # NOTE: 为什么要保存所有通用寄存器然后恢复所有寄存器?我看标准不是说有caller saved和callee saved吗,有些寄存器不用saved吧?
    # 好问题,caller saved和callee saved是针对function call设计的,trap handler不适用,因为
    # trap可能在任何时候发生,必须自行保存所有needed的registers用于trap restore
    # TODO: [optional]优化寄存器saved分类,降低开销(错误!!!!,caller saved和callee saved不适用于trap_handler)
