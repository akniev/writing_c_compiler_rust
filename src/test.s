  .globl main
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $8, %rsp
  movl $1, %r11d
  cmpl $0, %r11d
  je .L_false_label_1
  cmpl $0, -4(%rbp)
  je .L_false_label_1
  movl $1, -8(%rbp)
  jmp .L_end_label_3
.L_false_label_1:
  movl $0, -8(%rbp)
.L_end_label_3:
  movl -8(%rbp), %r10d
  movl %r10d, -4(%rbp)
  movl -4(%rbp), %eax
  movq %rbp, %rsp
  popq %rbp
  ret
  movl $0, %eax
  movq %rbp, %rsp
  popq %rbp
  ret
.section .note.GNU-stack,"",@progbits