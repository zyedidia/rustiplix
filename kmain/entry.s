.section ".text.boot"

.globl _start
_start:
	la sp, _stack_start
	csrr a0, mhartid
	call start
_hlt:
	wfi
	j _hlt
