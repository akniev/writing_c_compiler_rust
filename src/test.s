  .globl main
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $12, %rsp
  movl $1, -4(%rbp)
.Lstart_do_while.1:
  movl -4(%rbp), %r10d
  movl %r10d, -8(%rbp)
  movl -8(%rbp), %r11d
  imull $2, %r11d
  movl %r11d, -8(%rbp)
  movl -8(%rbp), %r10d
  movl %r10d, -4(%rbp)
.Lcontinue_do_while.1:
  cmpl $11, -4(%rbp)
  movl $0, -12(%rbp)
  setl -12(%rbp)
  cmpl $0, -12(%rbp)
  jne .Lstart_do_while.1
.Lbreak_do_while.1:
  movl -4(%rbp), %eax
  movq %rbp, %rsp
  popq %rbp
  ret
  movl $0, %eax
  movq %rbp, %rsp
  popq %rbp
  ret
.section .note.GNU-stack,"",@progbits