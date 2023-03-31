.section ".text.boot"

.globl _start
_start:
	.option push
	.option norelax
	la gp, __global_pointer$
	.option pop
	la sp, _stack_start
	# Use a0 (hartid) to calculate stack.
	addi t0, a0, 1
	slli t0, t0, 12 # t0 = (hartid + 1) * 4096
	add sp, sp, t0  # sp = _kheap_start + (hartid + 1) * 4096

	# Read primary and mark subsequent cores as non-primary.
	la t1, primary
	lw a1, 0(t1)
	sw zero, 0(t1)
	call start
.globl _halt
_halt:
	wfi
	j _halt

.section ".data.primary"
.globl primary
.align 4
primary:
	.int 1
