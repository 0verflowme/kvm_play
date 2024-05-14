; File: output.asm
BITS 32

section .text
global _start

_start:
    mov al, 'A'        ; Load the character 'A' into AL
    mov dx, 0x3F8      ; COM1 Serial Port
    out dx, al         ; Write AL to the port

    hlt                ; Halt the CPU
