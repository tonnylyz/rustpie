.text

.global pop_context_first
pop_context_first:
# sp <- bottom of context_frame
    mov rsp, rdi
pop_context:
    pop rax
    pop rbx
    pop rcx
    pop rdx
    pop rsi
    pop rdi
    pop rbp
    pop r8
    pop r9
    pop r10
    pop r11
    pop r12
    pop r13
    pop r14
    pop r15
    swapgs
# pop SS, RSP, RFLAGS, CS, RIP
    iretq

# SYSCALL instruction:
#   1. save rip into rcx
#   2. clear RF bit in rflags and save it into r11
#   3. SS/CS loaded from MSR, rip jump to here
# Note: rsp remains as user sp. Nothing on stack
.global syscall_entry
syscall_entry:
    swapgs
    # save user rsp to per cpu
    mov  gs:0x8, rsp
    mov  rsp, gs:0x0
    mov  gs:0x10, rax
    mov  rax, 35
    push rax
    push gs:0x8
    push r11
    mov  rax, 27
    push rax
    push rcx
    push r15
    push r14
    push r13
    push r12
    push r11
    push r10
    push r9
    push r8
    push rbp
    push rdi
    push rsi
    push rdx
    push rcx
    push rbx
    push gs:0x10
    mov  rdi, rsp
    call syscall_rust_entry
    jmp pop_context
# TODO: use sysret to return to ring 3
