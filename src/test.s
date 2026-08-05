  .globl main
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $32, %rsp
  movl $1, -4(%rbp)
  movl -4(%rbp), %r10d
  movl %r10d, -8(%rbp)
  movl $1, -12(%rbp)
  movl -4(%rbp), %r10d
  addl %r10d, -12(%rbp)
  movl -12(%rbp), %r10d
  movl %r10d, -4(%rbp)
  cmpl $0, -8(%rbp)
  movl $0, -16(%rbp)
  sete -16(%rbp)
  movl -16(%rbp), %r10d
  movl %r10d, -20(%rbp)
  cmpl $2, -4(%rbp)
  movl $0, -24(%rbp)
  sete -24(%rbp)
  cmpl $0, -24(%rbp)
  je .L_false_label_6
  cmpl $0, -20(%rbp)
  movl $0, -28(%rbp)
  sete -28(%rbp)
  cmpl $0, -28(%rbp)
  je .L_false_label_6
  movl $1, -32(%rbp)
  jmp .L_end_label_8
.L_false_label_6:
  movl $0, -32(%rbp)
.L_end_label_8:
  movl -32(%rbp), %eax
  movq %rbp, %rsp
  popq %rbp
  ret
  movl $0, %eax
  movq %rbp, %rsp
  popq %rbp
  ret
.section .note.GNU-stack,"",@progbits