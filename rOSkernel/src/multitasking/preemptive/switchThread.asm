//; rust-compatible signature:
//; fn asmSwitchThread(newStackPtr: u64, newIP: u64) -> !;

.global asmSwitchThread
asmSwitchThread:
     pushfq                  //; push RFLAGS to stack
     mov rax, rsp            //; save old stack pointer in `rax`

     mov rcx, rsi            //; backup new IP
     pop rsi                 //; pop old IP from stack
    
     mov rsp, rdi            //; set new stack pointer from `rdi` (arg 1)

     mov rdi, rax            //; use old stack pointer as argument 1
     call addPausedThread    //; call function with argument
     popfq                   //; restore RFLAGS from stack
     mov rsi, rcx            //; restore new IP
     push rsi                //; push argument 2 (new IP) to stack
     ret                     //; return to new IP


