  .globl main
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $20, %rsp
  movl $5, -4(%rbp)
  movl $1, %r11d
  cmpl $0, %r11d
  je .L_e2_label_2
  movl $5, -8(%rbp)
  jmp .L_end_label_3
.L_e2_label_2:
  movl $3, -8(%rbp)
.L_end_label_3:
  movl -4(%rbp), %r10d
  movl %r10d, -12(%rbp)
  movl -12(%rbp), %r11d
  imull -8(%rbp), %r11d
  movl %r11d, -12(%rbp)
  movl -12(%rbp), %r10d
  movl %r10d, -4(%rbp)
  cmpl $5, -4(%rbp)
  movl $0, -16(%rbp)
  setl -16(%rbp)
  cmpl $0, -16(%rbp)
  je .L_else_label_6
  movl $4, -4(%rbp)
  jmp .L_end_label_7
.L_else_label_6:
  movl -4(%rbp), %r10d
  movl %r10d, -20(%rbp)
  movl -20(%rbp), %r11d
  imull $2, %r11d
  movl %r11d, -20(%rbp)
  movl -20(%rbp), %r10d
  movl %r10d, -4(%rbp)
.L_end_label_7:
  movl -4(%rbp), %eax
  movq %rbp, %rsp
  popq %rbp
  ret
  movl $0, %eax
  movq %rbp, %rsp
  popq %rbp
  ret
.section .note.GNU-stack,"",@progbits