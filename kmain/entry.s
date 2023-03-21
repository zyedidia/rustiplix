.section ".text.boot"

.globl _start
_start:
	.option push
	.option norelax
	la gp, __global_pointer$
	.option pop
	la sp, _stack_start
	csrr a0, mhartid
	bnez a0, _hlt
	call start
_hlt:
	wfi
	j _hlt
