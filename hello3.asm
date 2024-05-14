BITS 32

section .text
global _start

_start:
    ; Set up the data to be sent
    mov al, 'A'        ; Load the character 'A' into AL
    
    ; Configure the port
    mov dx, 0x3F8      ; Load the address of COM1 Serial Port into DX
    
    ; Perform the write operation to the port
    out dx, al         ; Write the content of AL to the port pointed by DX

    ; Exit (End the process gracefully in Linux environment)
    mov eax, 1         ; Syscall number for 'exit' in Linux
    xor ebx, ebx       ; Return 0 status
    int 0x80           ; Trigger the syscall interrupt

; This 'hlt' instruction is not typically needed in user-level programs,
; and it can only be executed by kernel-level (privileged) code.
; It is only used here for illustrative purposes if running in an environment
; where execution should halt after operating.
; hlt              
