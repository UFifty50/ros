.global timerInterruptEntry
.extern timer_interrupt_trampoline

// bytes saved: 15 registers * 8 bytes each
.equ GPREG_SAVE_BYTES, 120

timerInterruptEntry:
    // push general-purpose registers in reverse order of GPRegisters struct
    push rax
    push rbx
    push rcx
    push rdx
    push rbp
    push rdi
    push rsi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    // first arg: pointer to saved GP registers
    mov rdi, rsp
    // second arg: pointer to InterruptStackFrame (above GP register save area)
    lea rsi, [rsp + GPREG_SAVE_BYTES]

    call timer_interrupt_trampoline

    // trampoline returns pointer to GP registers to restore in RAX
    mov rsp, rax

    // restore in reverse order
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rsi
    pop rdi
    pop rbp
    pop rdx
    pop rcx
    pop rbx
    pop rax

    iretq