//; rust-compatible signature:
//; asmSwitchThread(oldStackPointer: u64)
.globl asmSwitchThread
asmSwitchThread:
    pushfq                  //; push RFLAGS to stack
    mov rax, rsp            //; save old stack pointer in `rax`
    mov rsp, rdi            //; set new stack pointer from `rdi` (first argument)
    mov rdi, rax            //; use old stack pointer as argument
    call addPausedThread    //; call function with argument
    popfq                   //; restore RFLAGS from stack
    ret                     //; return - pop return address and jump to it
