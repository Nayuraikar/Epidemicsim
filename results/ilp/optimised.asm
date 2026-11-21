	.globl	epidemicsim_backends::kernels::place_optimised
	.p2align	6
epidemicsim_backends::kernels::place_optimised:
Lfunc_begin70:
	.cfi_startproc
	stp d11, d10, [sp, #-128]!
	.cfi_def_cfa_offset 128
	stp d9, d8, [sp, #16]
	stp x28, x27, [sp, #32]
	stp x26, x25, [sp, #48]
	stp x24, x23, [sp, #64]
	stp x22, x21, [sp, #80]
	stp x20, x19, [sp, #96]
	stp x29, x30, [sp, #112]
	add x29, sp, #112
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset w23, -56
	.cfi_offset w24, -64
	.cfi_offset w25, -72
	.cfi_offset w26, -80
	.cfi_offset w27, -88
	.cfi_offset w28, -96
	.cfi_offset b8, -104
	.cfi_offset b9, -112
	.cfi_offset b10, -120
	.cfi_offset b11, -128
	.cfi_remember_state
	sub sp, sp, #560
	str x6, [sp]
	ldr x8, [x1, #16]
	subs x7, x3, x2
	b.lo LBB70_94
	cmp x3, x8
	b.hi LBB70_94
	ldr x8, [x1, #64]
	cmp x3, x8
	b.hi LBB70_97
	ldr x8, [x1, #112]
	cmp x3, x8
	b.hi LBB70_98
	ldr x8, [x1, #136]
	cmp x3, x8
	b.hi LBB70_99
	mov x30, x0
	str x4, [sp, #8]
	ldr x8, [x1, #8]
	add x9, x8, x2, lsl #2
	ldr x8, [x1, #56]
	add x3, x8, x2
	ldr x8, [x1, #104]
	add x0, x8, x2, lsl #2
	ldr x8, [x1, #128]
	add x2, x8, x2, lsl #2
	ands x1, x7, #0xfffffffffffffffc
	stp x5, x7, [sp, #176]
	str x30, [sp, #168]
	stp x3, x9, [sp, #200]
	str x0, [sp, #216]
	str x2, [sp, #192]
	b.eq LBB70_52
	mov x22, #0
	mov w21, #8019
	movk w21, #53841, lsl #16
	ldp x23, x20, [x30]
	lsr x15, x20, #32
	lsr x4, x23, #32
	mov w8, #31161
	movk w8, #40503, lsl #16
	mov w16, #44677
	movk w16, #47975, lsl #16
	ldr d8, [x30, #56]
	add x9, x4, x16
	mov w10, w23
	eor x10, x10, #0x6
	umull x11, w10, w21
	mov w10, #62322
	movk w10, #15470, lsl #16
	mov w0, #23818
	movk w0, #30415, lsl #16
	add x13, x4, x0
	mov w12, #27947
	movk w12, #55974, lsl #16
	add x24, x23, x12
	mov w3, #2959
	movk w3, #12855, lsl #16
	add x14, x4, x3
	str x14, [sp, #400]
	mov w14, #59108
	movk w14, #30941, lsl #16
	add x17, x23, x14
	str x17, [sp, #392]
	mov w19, #47636
	movk w19, #60830, lsl #16
	add x17, x4, x19
	str x17, [sp, #384]
	mov w6, #26777
	movk w6, #43270, lsl #16
	add x17, x4, x6
	str x17, [sp, #376]
	mov w17, #5918
	movk w17, #25710, lsl #16
	add x2, x4, x17
	str x2, [sp, #368]
	str x1, [sp, #120]
	mov w1, #50595
	movk w1, #8149, lsl #16
	add x2, x4, x1
	str x2, [sp, #360]
	mov w2, #29736
	movk w2, #56125, lsl #16
	add x4, x4, x2
	str x4, [sp, #352]
	ldr x4, [sp, #216]
	add w16, w15, w16
	str w16, [sp, #100]
	add w16, w15, w0
	mov x0, x4
	str w16, [sp, #96]
	add w16, w15, w3
	str w16, [sp, #92]
	add w16, w15, w19
	str w16, [sp, #88]
	add x19, x23, x10
	add w16, w15, w6
	str w16, [sp, #84]
	mov x6, x23
	mov x23, x24
	mov x16, #4463067230724161536
	dup.2d v0, x16
	str q0, [sp, #64]
	add w17, w15, w17
	str w17, [sp, #60]
	add w17, w15, w1
	ldr x1, [sp, #120]
	add w15, w15, w2
	stp w15, w17, [sp, #52]
	ldp x2, x3, [sp, #192]
	mov w15, #27320
	movk w15, #27892, lsl #16
	movi.2d v2, #0000000000000000
	fmov d9, #0.50000000
	add w17, w6, w8
	eor x15, x17, x15
	str x15, [sp, #344]
	mov w15, #24733
	movk w15, #5909, lsl #16
	fmov d10, x16
	add x16, x6, x15
	eor x13, x13, x11
	stp x13, x16, [sp, #328]
	eor x9, x9, x11, lsr #32
	str x9, [sp, #320]
	mov w9, #55894
	movk w9, #46412, lsl #16
	add w8, w20, w8
	str w8, [sp, #48]
	add w8, w20, w10
	str w8, [sp, #44]
	add w8, w20, w12
	str w8, [sp, #40]
	add w8, w20, w14
	str w8, [sp, #36]
	add w8, w20, w15
	str w8, [sp, #32]
	add x8, x6, x9
	str x8, [sp, #312]
	add w8, w20, w9
	str w8, [sp, #28]
	mov w8, #21519
	movk w8, #21380, lsl #16
	add x9, x6, x8
	str x9, [sp, #304]
	add w8, w20, w8
	str w8, [sp, #24]
	mov w8, #18305
	movk w8, #36851, lsl #16
	add w9, w6, w8
	str w9, [sp, #300]
	str x20, [sp, #112]
	add w8, w20, w8
	str w8, [sp, #20]
	ldrb w8, [x30, #100]
	str w8, [sp, #132]
	ldrb w8, [x30, #101]
	str w8, [sp, #408]
	ldrb w9, [x30, #102]
	ldp w11, w8, [x30, #88]
	stp w11, w9, [x29, #-208]
	stur w8, [x29, #-212]
	ldp w9, w8, [x30, #64]
	str w9, [sp, #16]
	stur w8, [x29, #-216]
	ldp w9, w8, [x30, #72]
	stur w9, [x29, #-224]
	stur w8, [x29, #-232]
	ldr x8, [x30, #32]
	str x8, [sp, #288]
	ldp x8, x27, [x30, #40]
	str x8, [sp, #240]
	ldp w9, w8, [x30, #80]
	str w9, [sp, #284]
	str w8, [sp, #236]
	mov w24, #36183
	movk w24, #52638, lsl #16
	add x8, x30, #96
	stur x8, [x29, #-240]
	ldrb w20, [x30, #103]
	ldr x8, [sp]
	ldp x28, x9, [x8, #8]
	stur x9, [x29, #-200]
	ldp x11, x9, [x8, #56]
	str x9, [sp, #272]
	ldp x9, x8, [x8, #32]
	stp x9, x11, [sp, #248]
	str x8, [sp, #264]
	mov w15, #24
	mov w16, #16
	mov x17, x3
	mov x4, x2
	stur x0, [x29, #-192]
	ldr x26, [sp, #208]
	ldr x25, [sp, #8]
	str x6, [sp, #104]
	stp x23, x19, [sp, #152]
	b LBB70_8
LBB70_7:
	add x22, x22, #4
	add x25, x25, #16
	add x26, x26, #16
	ldur x8, [x29, #-192]
	add x8, x8, #16
	stur x8, [x29, #-192]
	add x4, x4, #16
	add x17, x17, #4
	cmp x22, x1
	ldr x0, [sp, #216]
	b.hs LBB70_52
LBB70_8:
	cmp x22, x7
	b.hs LBB70_110
	ldr w8, [x2, x22, lsl #2]
	ldr w9, [x0, x22, lsl #2]
	cmp w8, w9
	csel x8, x16, x15, eq
	ldr d0, [x30, x8]
	fcmp d0, #0.0
	ldrb w8, [x3, x22]
	and w8, w8, #0xfffffffd
	ccmp w8, #4, #4, gt
	cset w8, ne
	sturb w8, [x29, #-180]
	orr x23, x22, #0x1
	cmp x23, x7
	b.hs LBB70_109
	ldr w9, [x2, x23, lsl #2]
	ldr w10, [x0, x23, lsl #2]
	cmp w9, w10
	csel x9, x16, x15, eq
	ldr d0, [x30, x9]
	fcmp d0, #0.0
	ldrb w9, [x3, x23]
	and w9, w9, #0xfffffffd
	ccmp w9, #4, #4, gt
	cset w9, ne
	sturb w9, [x29, #-179]
	orr x19, x22, #0x2
	cmp x19, x7
	b.hs LBB70_108
	ldr w10, [x2, x19, lsl #2]
	ldr w11, [x0, x19, lsl #2]
	cmp w10, w11
	csel x10, x16, x15, eq
	ldr d0, [x30, x10]
	fcmp d0, #0.0
	ldrb w10, [x3, x19]
	and w10, w10, #0xfffffffd
	ccmp w10, #4, #4, gt
	cset w10, ne
	sturb w10, [x29, #-178]
	str x22, [sp, #224]
	orr x22, x22, #0x3
	cmp x22, x7
	b.hs LBB70_110
	orr w8, w8, w9
	orr w8, w8, w10
	ldr w9, [x2, x22, lsl #2]
	ldr w10, [x0, x22, lsl #2]
	cmp w9, w10
	csel x9, x16, x15, eq
	ldr d0, [x30, x9]
	fcmp d0, #0.0
	ldrb w9, [x3, x22]
	and w9, w9, #0xfffffffd
	ccmp w9, #4, #4, gt
	cset w9, ne
	sturb w9, [x29, #-177]
	stp q2, q2, [x29, #-176]
	csinc w8, w8, wzr, eq
	stp x4, x17, [x29, #-256]
	cbz w8, LBB70_15
	ldr w8, [sp, #132]
	tbz w8, #0, LBB70_15
	sub x0, x29, #136
Lloh331:
	adrp x1, l_.memset_pattern@PAGE
Lloh332:
	add x1, x1, l_.memset_pattern@PAGEOFF
	mov w2, #16
	bl _memset_pattern16
	ldp x30, x5, [sp, #168]
	ldr x11, [sp, #208]
	ldr w8, [x11, x22, lsl #2]
	ldr w9, [x11, x19, lsl #2]
	ldr w10, [x11, x23, lsl #2]
	ldr x12, [sp, #224]
	ldr w11, [x11, x12, lsl #2]
	ldp w13, w12, [x29, #-128]
	ldp w15, w14, [x29, #-136]
	umull x15, w15, w24
	lsr x16, x15, #32
	ldr x22, [sp, #112]
	ldr w17, [sp, #16]
	eor w17, w17, w22
	eor w16, w17, w16
	umull x14, w14, w24
	lsr x0, x14, #32
	eor w0, w17, w0
	umull x1, w11, w21
	eor x11, x1, x22
	lsr x11, x11, #32
	umull x2, w10, w21
	eor x10, x2, x22
	lsr x3, x10, #32
	umull x4, w13, w24
	lsr x10, x4, #32
	eor w19, w17, w10
	umull x10, w12, w24
	lsr x12, x10, #32
	eor w13, w17, w12
	umull x6, w9, w21
	eor x9, x6, x22
	lsr x7, x9, #32
	umull x17, w8, w21
	eor x8, x17, x22
	lsr x9, x8, #32
	umull x8, w3, w24
	umull x11, w11, w24
	lsr x12, x11, #32
	ldr w23, [sp, #48]
	eor w15, w23, w15
	eor w3, w15, w12
	umull x12, w21, w16
	lsr x15, x12, #32
	ldr w22, [sp, #100]
	eor w16, w22, w1
	eor w1, w16, w15
	lsr x15, x8, #32
	umull x16, w21, w0
	eor w14, w23, w14
	eor w0, w14, w15
	lsr x14, x16, #32
	eor w15, w22, w2
	eor w2, w15, w14
	umull x9, w9, w24
	umull x14, w7, w24
	lsr x15, x14, #32
	eor w4, w23, w4
	eor w4, w4, w15
	umull x15, w21, w19
	lsr x7, x15, #32
	eor w6, w22, w6
	eor w19, w6, w7
	lsr x6, x9, #32
	umull x7, w21, w13
	eor w10, w23, w10
	eor w6, w10, w6
	lsr x10, x7, #32
	eor w13, w22, w17
	eor w17, w13, w10
	umull x10, w24, w2
	umull x13, w24, w1
	lsr x1, x13, #32
	ldr w23, [sp, #44]
	eor w11, w23, w11
	eor w1, w11, w1
	umull x11, w21, w3
	lsr x2, x11, #32
	ldr w22, [sp, #96]
	eor w12, w22, w12
	eor w2, w12, w2
	lsr x12, x10, #32
	umull x0, w21, w0
	eor w8, w23, w8
	eor w3, w8, w12
	lsr x8, x0, #32
	eor w12, w22, w16
	eor w16, w12, w8
	umull x8, w24, w17
	umull x12, w24, w19
	lsr x17, x12, #32
	eor w14, w23, w14
	eor w17, w14, w17
	umull x14, w21, w4
	lsr x4, x14, #32
	eor w15, w22, w15
	eor w4, w15, w4
	lsr x15, x8, #32
	umull x19, w21, w6
	eor w9, w23, w9
	eor w6, w9, w15
	lsr x9, x19, #32
	eor w15, w22, w7
	eor w7, w15, w9
	umull x9, w24, w16
	umull x15, w24, w2
	lsr x16, x15, #32
	ldr w23, [sp, #40]
	eor w13, w23, w13
	eor w16, w13, w16
	umull x13, w21, w1
	lsr x1, x13, #32
	ldr w22, [sp, #92]
	eor w11, w22, w11
	eor w1, w11, w1
	lsr x11, x9, #32
	umull x2, w21, w3
	eor w10, w23, w10
	eor w3, w10, w11
	lsr x10, x2, #32
	eor w11, w22, w0
	eor w0, w11, w10
	umull x10, w24, w7
	umull x11, w24, w4
	lsr x4, x11, #32
	eor w12, w23, w12
	eor w4, w12, w4
	umull x12, w21, w17
	lsr x17, x12, #32
	eor w14, w22, w14
	eor w17, w14, w17
	lsr x14, x10, #32
	umull x6, w21, w6
	eor w8, w23, w8
	eor w8, w8, w14
	str w8, [sp, #144]
	lsr x8, x6, #32
	eor w14, w22, w19
	eor w22, w14, w8
	umull x7, w24, w0
	umull x14, w24, w1
	lsr x0, x14, #32
	ldr w8, [sp, #36]
	eor w15, w8, w15
	eor w0, w15, w0
	umull x15, w21, w16
	lsr x16, x15, #32
	ldr w23, [sp, #88]
	eor w13, w23, w13
	eor w1, w13, w16
	lsr x13, x7, #32
	umull x3, w21, w3
	eor w9, w8, w9
	eor w9, w9, w13
	str w9, [sp, #140]
	lsr x9, x3, #32
	eor w13, w23, w2
	eor w2, w13, w9
	umull x19, w24, w22
	umull x13, w24, w17
	lsr x16, x13, #32
	eor w11, w8, w11
	eor w17, w11, w16
	umull x16, w21, w4
	lsr x11, x16, #32
	eor w12, w23, w12
	eor w4, w12, w11
	lsr x11, x19, #32
	ldr w9, [sp, #144]
	umull x22, w21, w9
	eor w10, w8, w10
	eor w8, w10, w11
	str w8, [sp, #136]
	lsr x10, x22, #32
	eor w11, w23, w6
	eor w11, w11, w10
	umull x10, w24, w2
	umull x12, w24, w1
	lsr x1, x12, #32
	ldr w9, [sp, #32]
	eor w14, w9, w14
	eor w1, w14, w1
	umull x14, w21, w0
	lsr x0, x14, #32
	ldr w23, [sp, #84]
	eor w15, w23, w15
	eor w0, w15, w0
	lsr x15, x10, #32
	ldr w8, [sp, #140]
	umull x2, w21, w8
	eor w8, w9, w7
	eor w8, w8, w15
	lsr x15, x2, #32
	eor w3, w23, w3
	eor w3, w3, w15
	umull x11, w24, w11
	str x11, [sp, #144]
	umull x15, w24, w4
	lsr x4, x15, #32
	eor w13, w9, w13
	eor w13, w13, w4
	umull x17, w21, w17
	lsr x4, x17, #32
	eor w16, w23, w16
	eor w4, w16, w4
	lsr x16, x11, #32
	ldr w11, [sp, #136]
	umull x6, w21, w11
	eor w9, w9, w19
	eor w7, w9, w16
	lsr x9, x6, #32
	eor w16, w23, w22
	eor w23, w16, w9
	umull x22, w24, w3
	umull x16, w24, w0
	lsr x0, x16, #32
	ldr w11, [sp, #28]
	eor w12, w11, w12
	eor w3, w12, w0
	umull x0, w21, w1
	lsr x12, x0, #32
	ldr w9, [sp, #60]
	eor w14, w9, w14
	eor w19, w14, w12
	lsr x12, x22, #32
	umull x1, w21, w8
	eor w8, w11, w10
	eor w8, w8, w12
	str w8, [sp, #140]
	lsr x8, x1, #32
	eor w10, w9, w2
	eor w2, w10, w8
	umull x8, w24, w23
	umull x12, w24, w4
	lsr x10, x12, #32
	eor w14, w11, w15
	eor w15, w14, w10
	umull x13, w21, w13
	lsr x10, x13, #32
	eor w14, w9, w17
	eor w17, w14, w10
	lsr x10, x8, #32
	umull x14, w21, w7
	ldr x7, [sp, #184]
	ldr x4, [sp, #144]
	eor w11, w11, w4
	eor w11, w11, w10
	lsr x10, x14, #32
	eor w4, w9, w6
	ldr x6, [sp, #104]
	eor w4, w4, w10
	umull x10, w24, w19
	lsr x10, x10, #32
	ldr w23, [sp, #24]
	eor w16, w23, w16
	eor w16, w16, w10
	umull x3, w21, w3
	lsr x10, x3, #32
	ldr w19, [sp, #56]
	eor w0, w19, w0
	eor w10, w0, w10
	umull x0, w24, w2
	lsr x0, x0, #32
	ldr w9, [sp, #140]
	umull x2, w21, w9
	eor w9, w23, w22
	eor w0, w9, w0
	lsr x9, x2, #32
	eor w1, w19, w1
	eor w9, w1, w9
	ldr x1, [sp, #120]
	umull x17, w24, w17
	lsr x17, x17, #32
	eor w12, w23, w12
	eor w12, w12, w17
	umull x15, w21, w15
	lsr x17, x15, #32
	eor w13, w19, w13
	eor w13, w13, w17
	umull x17, w24, w4
	lsr x17, x17, #32
	umull x11, w21, w11
	eor w8, w23, w8
	eor w8, w8, w17
	lsr x17, x11, #32
	eor w14, w19, w14
	eor w14, w14, w17
	umull x16, w21, w16
	lsr x16, x16, #32
	ldr w4, [sp, #52]
	eor w17, w4, w3
	eor w16, w17, w16
	umull x17, w21, w0
	lsr x17, x17, #32
	eor w0, w4, w2
	ldp x2, x3, [sp, #192]
	eor w17, w0, w17
	umull x12, w21, w12
	lsr x12, x12, #32
	eor w15, w4, w15
	eor w12, w15, w12
	umull x8, w21, w8
	lsr x8, x8, #32
	eor w11, w4, w11
	eor w8, w11, w8
	mul w10, w10, w24
	umull x11, w24, w16
	mov w16, #16
	lsr x11, x11, #32
	ldr w15, [sp, #20]
	eor w10, w15, w10
	eor w10, w10, w11
	mul w9, w9, w24
	umull x11, w24, w17
	ldp x4, x17, [x29, #-256]
	lsr x11, x11, #32
	eor w9, w15, w9
	eor w9, w9, w11
	mul w11, w13, w24
	umull x12, w24, w12
	lsr x12, x12, #32
	eor w11, w15, w11
	eor w11, w11, w12
	mul w12, w14, w24
	umull x8, w24, w8
	lsr x8, x8, #32
	eor w12, w15, w12
	mov w15, #24
	eor w8, w12, w8
	ucvtf d0, w10
	ucvtf d1, w9
	ucvtf d2, w11
	ucvtf d3, w8
	mov.d v0[1], v1[0]
	mov.d v2[1], v3[0]
	fmov.2d v1, #0.50000000
	fadd.2d v0, v0, v1
	ldr q3, [sp, #64]
	fmul.2d v0, v0, v3
	fadd.2d v1, v2, v1
	movi.2d v2, #0000000000000000
	fmul.2d v1, v1, v3
	stp q0, q1, [x29, #-176]
LBB70_15:
	mov x8, #0
	ldr x22, [sp, #224]
	ldp x23, x19, [sp, #152]
	b LBB70_20
LBB70_16:
	ldr x9, [sp, #264]
	cmp x9, x0
	b.ls LBB70_106
	ldr x10, [sp, #248]
LBB70_18:
	ldr w9, [x10, x0, lsl #2]
	add w9, w9, #1
	str w9, [x10, x0, lsl #2]
LBB70_19:
	add x8, x8, #1
	cmp x8, #4
	b.eq LBB70_7
LBB70_20:
	add x0, x22, x8
	cmp x0, x7
	b.hs LBB70_96
	ldr w11, [x4, x8, lsl #2]
	ldur x9, [x29, #-192]
	ldr w10, [x9, x8, lsl #2]
	ldrb w9, [x17, x8]
	ldr w12, [x26, x8, lsl #2]
	sub x13, x29, #176
	ldr d0, [x13, x8, lsl #3]
	cmp w11, w10
	csel x13, x16, x15, eq
	ldr d1, [x30, x13]
	fcmp d0, d1
	b.pl LBB70_25
	sub x13, x29, #180
	ldrb w13, [x13, x8]
	tbz w13, #0, LBB70_25
	cbz x27, LBB70_100
	ldr x11, [sp, #288]
	mul x11, x11, x10
	udiv x11, x11, x27
	ldr w13, [sp, #284]
	add w11, w13, w11
	cmp w9, #3
	b.eq LBB70_27
	b LBB70_30
LBB70_25:
	ldr w13, [sp, #408]
	.p2align	6, , 4
	cmp w13, #0
	csel w11, w11, w10, ne
	cmp w9, #3
	b.ne LBB70_30
LBB70_27:
	fcmp d8, #0.0
	b.le LBB70_30
	umull x12, w12, w21
	eor x13, x6, x12
	lsr x13, x13, #32
	umull x13, w13, w24
	ldr x14, [sp, #320]
	eor w12, w12, w14
	ldr x14, [sp, #344]
	eor x14, x14, x13, lsr #32
	umull x14, w14, w21
	lsr x15, x14, #32
	umull x12, w24, w12
	lsr x16, x12, #32
	eor w16, w19, w16
	eor w13, w16, w13
	ldr x16, [sp, #328]
	eor w15, w15, w16
	umull x13, w21, w13
	lsr x16, x13, #32
	umull x15, w24, w15
	lsr x17, x15, #32
	eor w12, w23, w12
	eor w12, w12, w17
	ldp x4, x17, [sp, #392]
	eor w14, w17, w14
	eor w14, w14, w16
	umull x12, w21, w12
	lsr x16, x12, #32
	umull x14, w24, w14
	lsr x17, x14, #32
	eor w15, w4, w15
	eor w15, w15, w17
	ldr x17, [sp, #384]
	eor w13, w17, w13
	eor w13, w13, w16
	umull x15, w21, w15
	lsr x16, x15, #32
	umull x13, w24, w13
	lsr x17, x13, #32
	ldr x4, [sp, #336]
	eor w14, w4, w14
	eor w14, w14, w17
	ldr x17, [sp, #376]
	eor w12, w17, w12
	eor w12, w12, w16
	umull x14, w21, w14
	lsr x16, x14, #32
	umull x12, w24, w12
	lsr x17, x12, #32
	ldr x4, [sp, #312]
	eor w13, w4, w13
	eor w13, w13, w17
	ldr x17, [sp, #368]
	eor w15, w17, w15
	eor w15, w15, w16
	umull x13, w21, w13
	lsr x16, x13, #32
	umull x15, w24, w15
	lsr x15, x15, #32
	ldr x17, [sp, #304]
	eor w12, w17, w12
	ldp x4, x17, [x29, #-256]
	eor w12, w12, w15
	ldr x15, [sp, #360]
	eor w14, w15, w14
	eor w14, w14, w16
	mov w16, #16
	umull x12, w21, w12
	lsr x12, x12, #32
	mul w14, w14, w24
	ldr x15, [sp, #352]
	eor w13, w15, w13
	mov w15, #24
	eor w12, w13, w12
	umull x12, w24, w12
	lsr x12, x12, #32
	ldr w13, [sp, #300]
	eor w13, w13, w14
	eor w12, w13, w12
	ucvtf d0, w12
	fadd d0, d0, d9
	fmul d0, d0, d10
	fcmp d0, d8
	mov x12, x10
	b.mi LBB70_34
	ldp w12, w13, [x29, #-212]
	cmp w11, w13
	ccmp w11, w12, #2, hs
	ldur w12, [x29, #-204]
	csel w12, wzr, w12, hs
	ldur w13, [x29, #-216]
	cmp w11, w13
	ldur x13, [x29, #-240]
	cinc x13, x13, hs
	ldur w14, [x29, #-224]
	cmp w11, w14
	cinc x13, x13, hs
	ldur w14, [x29, #-232]
	cmp w11, w14
	cinc x13, x13, hs
	ldrb w13, [x13]
	orr w12, w12, w13
	tst w12, #0x1
	csel w12, w10, w11, ne
	b LBB70_34
LBB70_30:
	ldp w12, w13, [x29, #-212]
	cmp w11, w13
	ccmp w11, w12, #2, hs
	ldur w12, [x29, #-204]
	csel w12, wzr, w12, hs
	ldur w13, [x29, #-216]
	cmp w11, w13
	ldur x13, [x29, #-240]
	cinc x13, x13, hs
	ldur w14, [x29, #-224]
	cmp w11, w14
	cinc x13, x13, hs
	ldur w14, [x29, #-232]
	cmp w11, w14
	cinc x13, x13, hs
	ldrb w13, [x13]
	orr w12, w12, w13
	tst w12, #0x1
	csel w12, w10, w11, ne
	cmp w9, #6
	b.eq LBB70_48
	cmp w9, #4
	b.ne LBB70_34
	cbz x27, LBB70_103
	ldr x11, [sp, #240]
	mul x10, x11, x10
	udiv x10, x10, x27
	ldr w11, [sp, #236]
	add w12, w11, w10
LBB70_34:
	cmp x0, x5
	b.hs LBB70_95
	str w12, [x25, x8, lsl #2]
	mov w0, w12
	tbz w20, #0, LBB70_42
	ldur x10, [x29, #-200]
	cmp x10, x0
	b.ls LBB70_102
	add x10, x28, x0, lsl #2
	mov w11, #1
	ldadd w11, w10, [x10]
	cmp w9, #3
	b.eq LBB70_44
LBB70_38:
	cmp w9, #2
	b.ne LBB70_19
	tbz w20, #0, LBB70_16
	ldr x9, [sp, #264]
	cmp x9, x0
	b.ls LBB70_105
	ldr x9, [sp, #248]
	b LBB70_47
LBB70_42:
	ldur x10, [x29, #-200]
	cmp x10, x0
	b.ls LBB70_101
	ldr w10, [x28, x0, lsl #2]
	add w10, w10, #1
	str w10, [x28, x0, lsl #2]
	cmp w9, #3
	b.ne LBB70_38
LBB70_44:
	tbz w20, #0, LBB70_50
	ldr x9, [sp, #272]
	cmp x9, x0
	b.ls LBB70_104
	ldr x9, [sp, #256]
LBB70_47:
	add x9, x9, x0, lsl #2
	mov w10, #1
	ldadd w10, w9, [x9]
	b LBB70_19
LBB70_48:
	cmp x0, x5
	b.hs LBB70_95
	mov w9, #-1
	str w9, [x25, x8, lsl #2]
	b LBB70_19
LBB70_50:
	ldr x9, [sp, #272]
	cmp x9, x0
	b.ls LBB70_107
	ldr x10, [sp, #256]
	b LBB70_18
LBB70_52:
	cmp x1, x7
	ldr x24, [sp, #8]
	b.ne LBB70_54
LBB70_53:
	add sp, sp, #560
	.cfi_def_cfa wsp, 128
	ldp x29, x30, [sp, #112]
	ldp x20, x19, [sp, #96]
	ldp x22, x21, [sp, #80]
	ldp x24, x23, [sp, #64]
	ldp x26, x25, [sp, #48]
	ldp x28, x27, [sp, #32]
	ldp d9, d8, [sp, #16]
	ldp d11, d10, [sp], #128
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore w23
	.cfi_restore w24
	.cfi_restore w25
	.cfi_restore w26
	.cfi_restore w27
	.cfi_restore w28
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	ret
LBB70_54:
	.cfi_restore_state
	ldr x8, [x30]
	stur x8, [x29, #-232]
	ldrb w19, [x30, #101]
	ldr d8, [x30, #56]
	ldrb w8, [x30, #102]
	stur w8, [x29, #-192]
	ldp w9, w8, [x30, #88]
	stp w8, w9, [x29, #-204]
	ldp w9, w11, [x30, #64]
	str w9, [sp, #400]
	ldp w9, w8, [x30, #72]
	stp w9, w11, [x29, #-212]
	stur w8, [x29, #-216]
	add x8, x30, #96
	stur x8, [x29, #-224]
	ldr x8, [x30, #32]
	str x8, [sp, #376]
	ldp x8, x10, [x30, #40]
	str x8, [sp, #392]
	ldp w9, w8, [x30, #80]
	str w9, [sp, #368]
	str w8, [sp, #384]
	ldrb w20, [x30, #103]
	ldr x9, [sp]
	ldp x25, x26, [x9, #8]
	ldp x11, x8, [x9, #56]
	stur x11, [x29, #-256]
	stur x8, [x29, #-240]
	fmov d9, #0.50000000
	mov x8, #4463067230724161536
	fmov d10, x8
	ldp x11, x8, [x9, #32]
	str x11, [sp, #408]
	stur x8, [x29, #-248]
	mov x28, x10
	b LBB70_59
LBB70_55:
	ldur x8, [x29, #-248]
	cmp x8, x0
	b.ls LBB70_114
	ldr x11, [sp, #408]
LBB70_57:
	ldr w8, [x11, x0, lsl #2]
	add w8, w8, #1
	str w8, [x11, x0, lsl #2]
LBB70_58:
	add x1, x1, #1
	cmp x1, x7
	mov x0, x9
	b.hs LBB70_53
LBB70_59:
	ldrb w21, [x3, x1]
	ldr w23, [x2, x1, lsl #2]
	ldr w22, [x0, x1, lsl #2]
	cmp w23, w22
	mov w8, #24
	mov w9, #16
	csel x8, x9, x8, eq
	ldr d11, [x30, x8]
	ldr x8, [sp, #208]
	ldr w8, [x8, x1, lsl #2]
	fcmp d11, #0.0
	orr w9, w21, #0x2
	ccmp w9, #6, #4, gt
	b.ne LBB70_64
	.p2align	6, , 4
	cmp w19, #0
	csel w23, w23, w22, ne
	cmp w21, #3
	b.ne LBB70_72
LBB70_61:
	fcmp d8, #0.0
	b.le LBB70_72
	stur w8, [x29, #-176]
	mov x8, #34359738368
	stur x8, [x29, #-172]
	stur wzr, [x29, #-164]
	sub x0, x29, #136
	mov x27, x1
	sub x1, x29, #176
	ldur x2, [x29, #-232]
	bl epidemicsim_core::rng::philox4x32_10
	mov x10, x28
	ldp x2, x3, [sp, #192]
	mov x1, x27
	ldr x0, [sp, #216]
	ldp x30, x5, [sp, #168]
	ldr x7, [sp, #184]
	ldur s0, [x29, #-136]
	ucvtf d0, d0
	fadd d0, d0, d9
	fmul d0, d0, d10
	fcmp d0, d8
	mov x8, x22
	b.mi LBB70_76
	ldp w8, w9, [x29, #-204]
	cmp w23, w9
	ccmp w23, w8, #2, hs
	ldur w8, [x29, #-192]
	csel w8, wzr, w8, hs
	ldp w11, w9, [x29, #-212]
	cmp w23, w9
	ldur x9, [x29, #-224]
	cinc x9, x9, hs
	cmp w23, w11
	cinc x9, x9, hs
	ldur w11, [x29, #-216]
	cmp w23, w11
	cinc x9, x9, hs
	ldrb w9, [x9]
	orr w8, w8, w9
	tst w8, #0x1
	csel w8, w22, w23, ne
	b LBB70_76
LBB70_64:
	and w24, w21, #0xfffffffd
	stur w8, [x29, #-176]
	ldr w8, [sp, #400]
	stur w8, [x29, #-172]
	mov w8, #3
	stur x8, [x29, #-168]
	sub x0, x29, #136
	mov x27, x1
	sub x1, x29, #176
	ldur x2, [x29, #-232]
	bl epidemicsim_core::rng::philox4x32_10
	mov x1, x27
	ldr x8, [sp, #208]
	ldr w8, [x8, x27, lsl #2]
	cmp w24, #4
	b.eq LBB70_70
	ldur s0, [x29, #-136]
	ucvtf d0, d0
	fadd d0, d0, d9
	fmul d0, d0, d10
	fcmp d0, d11
	ldr x24, [sp, #8]
	mov x10, x28
	b.pl LBB70_68
	ldp x5, x7, [sp, #176]
	ldr x30, [sp, #168]
	ldp x2, x3, [sp, #192]
	ldr x0, [sp, #216]
	cbz x10, LBB70_100
	ldr x9, [sp, #376]
	mul x9, x9, x22
	udiv x9, x9, x10
	ldr w11, [sp, #368]
	add w23, w11, w9
	cmp w21, #3
	b.eq LBB70_61
	b LBB70_72
LBB70_68:
	ldp x5, x7, [sp, #176]
	ldr x30, [sp, #168]
	ldp x2, x3, [sp, #192]
	ldr x0, [sp, #216]
	.p2align	6, , 4
	cmp w19, #0
	csel w23, w23, w22, ne
	cmp w21, #3
	b.eq LBB70_61
	b LBB70_72
LBB70_70:
	ldp x5, x7, [sp, #176]
	ldr x24, [sp, #8]
	ldr x30, [sp, #168]
	ldp x2, x3, [sp, #192]
	ldr x0, [sp, #216]
	mov x10, x28
	.p2align	6, , 4
	cmp w19, #0
	csel w23, w23, w22, ne
	cmp w21, #3
	b.eq LBB70_61
LBB70_72:
	ldp w8, w9, [x29, #-204]
	cmp w23, w9
	ccmp w23, w8, #2, hs
	ldur w8, [x29, #-192]
	csel w8, wzr, w8, hs
	ldp w11, w9, [x29, #-212]
	cmp w23, w9
	ldur x9, [x29, #-224]
	cinc x9, x9, hs
	cmp w23, w11
	cinc x9, x9, hs
	ldur w11, [x29, #-216]
	cmp w23, w11
	cinc x9, x9, hs
	ldrb w9, [x9]
	orr w8, w8, w9
	tst w8, #0x1
	csel w8, w22, w23, ne
	cmp w21, #6
	b.eq LBB70_90
	cmp w21, #4
	b.ne LBB70_76
	cbz x10, LBB70_103
	ldr x8, [sp, #392]
	mul x8, x8, x22
	udiv x8, x8, x10
	ldr w9, [sp, #384]
	add w8, w9, w8
LBB70_76:
	cmp x1, x5
	b.hs LBB70_111
	mov x9, x0
	str w8, [x24, x1, lsl #2]
	mov w0, w8
	cmp x26, x0
	tbz w20, #0, LBB70_84
	b.ls LBB70_113
	add x8, x25, x0, lsl #2
	mov w11, #1
	ldadd w11, w8, [x8]
	cmp w21, #3
	b.eq LBB70_86
LBB70_80:
	cmp w21, #2
	b.ne LBB70_58
	tbz w20, #0, LBB70_55
	ldur x8, [x29, #-248]
	cmp x8, x0
	b.ls LBB70_115
	ldr x8, [sp, #408]
	b LBB70_89
LBB70_84:
	b.ls LBB70_112
	ldr w8, [x25, x0, lsl #2]
	add w8, w8, #1
	str w8, [x25, x0, lsl #2]
	cmp w21, #3
	b.ne LBB70_80
LBB70_86:
	tbz w20, #0, LBB70_92
	ldur x8, [x29, #-240]
	cmp x8, x0
	b.ls LBB70_117
	ldur x8, [x29, #-256]
LBB70_89:
	add x8, x8, x0, lsl #2
	mov w11, #1
	ldadd w11, w8, [x8]
	b LBB70_58
LBB70_90:
	cmp x1, x5
	b.hs LBB70_111
	mov x9, x0
	mov w8, #-1
	str w8, [x24, x1, lsl #2]
	b LBB70_58
LBB70_92:
	ldur x8, [x29, #-240]
	cmp x8, x0
	b.ls LBB70_116
	ldur x11, [x29, #-256]
	b LBB70_57
LBB70_94:
Lloh333:
	adrp x9, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.120@PAGE
Lloh334:
	add x9, x9, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.120@PAGEOFF
	mov x0, x2
	mov x1, x3
	mov x2, x8
	mov x3, x9
	bl core::slice::index::slice_index_fail
LBB70_95:
Lloh335:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.118@PAGE
Lloh336:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.118@PAGEOFF
	add x0, x22, x8
	mov x1, x5
	bl core::panicking::panic_bounds_check
LBB70_96:
Lloh337:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.117@PAGE
Lloh338:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.117@PAGEOFF
	mov x1, x7
	bl core::panicking::panic_bounds_check
LBB70_97:
Lloh339:
	adrp x9, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.113@PAGE
Lloh340:
	add x9, x9, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.113@PAGEOFF
	mov x0, x2
	mov x1, x3
	mov x2, x8
	mov x3, x9
	bl core::slice::index::slice_index_fail
LBB70_98:
Lloh341:
	adrp x9, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.114@PAGE
Lloh342:
	add x9, x9, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.114@PAGEOFF
	mov x0, x2
	mov x1, x3
	mov x2, x8
	mov x3, x9
	bl core::slice::index::slice_index_fail
LBB70_99:
Lloh343:
	adrp x9, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.115@PAGE
Lloh344:
	add x9, x9, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.115@PAGEOFF
	mov x0, x2
	mov x1, x3
	mov x2, x8
	mov x3, x9
	bl core::slice::index::slice_index_fail
LBB70_100:
Lloh345:
	adrp x0, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.126@PAGE
Lloh346:
	add x0, x0, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.126@PAGEOFF
	bl core::panicking::panic_const::panic_const_div_by_zero
LBB70_101:
Lloh347:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGE
Lloh348:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGEOFF
	ldur x1, [x29, #-200]
	bl core::panicking::panic_bounds_check
LBB70_102:
Lloh349:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGE
Lloh350:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGEOFF
	ldur x1, [x29, #-200]
	bl core::panicking::panic_bounds_check
LBB70_103:
Lloh351:
	adrp x0, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.127@PAGE
Lloh352:
	add x0, x0, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.127@PAGEOFF
	bl core::panicking::panic_const::panic_const_div_by_zero
LBB70_104:
Lloh353:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGE
Lloh354:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGEOFF
	ldr x1, [sp, #272]
	bl core::panicking::panic_bounds_check
LBB70_105:
Lloh355:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGE
Lloh356:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGEOFF
	ldr x1, [sp, #264]
	bl core::panicking::panic_bounds_check
LBB70_106:
Lloh357:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGE
Lloh358:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGEOFF
	ldr x1, [sp, #264]
	bl core::panicking::panic_bounds_check
LBB70_107:
Lloh359:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGE
Lloh360:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGEOFF
	ldr x1, [sp, #272]
	bl core::panicking::panic_bounds_check
LBB70_108:
	mov x22, x19
Lloh361:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.119@PAGE
Lloh362:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.119@PAGEOFF
	mov x0, x22
	mov x1, x7
	bl core::panicking::panic_bounds_check
LBB70_109:
	mov x22, x23
LBB70_110:
Lloh363:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.119@PAGE
Lloh364:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.119@PAGEOFF
	mov x0, x22
	mov x1, x7
	bl core::panicking::panic_bounds_check
LBB70_111:
Lloh365:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.116@PAGE
Lloh366:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.116@PAGEOFF
	mov x0, x1
	mov x1, x5
	bl core::panicking::panic_bounds_check
LBB70_112:
Lloh367:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGE
Lloh368:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGEOFF
	mov x1, x26
	bl core::panicking::panic_bounds_check
LBB70_113:
Lloh369:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGE
Lloh370:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGEOFF
	mov x1, x26
	bl core::panicking::panic_bounds_check
LBB70_114:
Lloh371:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGE
Lloh372:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGEOFF
	ldur x1, [x29, #-248]
	bl core::panicking::panic_bounds_check
LBB70_115:
Lloh373:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGE
Lloh374:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGEOFF
	ldur x1, [x29, #-248]
	bl core::panicking::panic_bounds_check
LBB70_116:
Lloh375:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGE
Lloh376:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGEOFF
	ldur x1, [x29, #-240]
	bl core::panicking::panic_bounds_check
LBB70_117:
Lloh377:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGE
Lloh378:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGEOFF
	ldur x1, [x29, #-240]
	bl core::panicking::panic_bounds_check
	.loh AdrpAdd	Lloh331, Lloh332
	.loh AdrpAdd	Lloh333, Lloh334
	.loh AdrpAdd	Lloh335, Lloh336
	.loh AdrpAdd	Lloh337, Lloh338
	.loh AdrpAdd	Lloh339, Lloh340
	.loh AdrpAdd	Lloh341, Lloh342
	.loh AdrpAdd	Lloh343, Lloh344
	.loh AdrpAdd	Lloh345, Lloh346
	.loh AdrpAdd	Lloh347, Lloh348
	.loh AdrpAdd	Lloh349, Lloh350
	.loh AdrpAdd	Lloh351, Lloh352
	.loh AdrpAdd	Lloh353, Lloh354
	.loh AdrpAdd	Lloh355, Lloh356
	.loh AdrpAdd	Lloh357, Lloh358
	.loh AdrpAdd	Lloh359, Lloh360
	.loh AdrpAdd	Lloh361, Lloh362
	.loh AdrpAdd	Lloh363, Lloh364
	.loh AdrpAdd	Lloh365, Lloh366
	.loh AdrpAdd	Lloh367, Lloh368
	.loh AdrpAdd	Lloh369, Lloh370
	.loh AdrpAdd	Lloh371, Lloh372
	.loh AdrpAdd	Lloh373, Lloh374
	.loh AdrpAdd	Lloh375, Lloh376
	.loh AdrpAdd	Lloh377, Lloh378
	.globl	epidemicsim_backends::kernels::update_optimised
	.p2align	6
epidemicsim_backends::kernels::update_optimised:
Lfunc_begin71:
	.cfi_startproc
	sub sp, sp, #416
	.cfi_def_cfa_offset 416
	stp x28, x27, [sp, #320]
	stp x26, x25, [sp, #336]
	stp x24, x23, [sp, #352]
	stp x22, x21, [sp, #368]
	stp x20, x19, [sp, #384]
	stp x29, x30, [sp, #400]
	add x29, sp, #400
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset w23, -56
	.cfi_offset w24, -64
	.cfi_offset w25, -72
	.cfi_offset w26, -80
	.cfi_offset w27, -88
	.cfi_offset w28, -96
	.cfi_remember_state
	mov x17, x0
	str x8, [sp]
	str x1, [sp, #80]
	ldr x1, [x1, #72]
	str xzr, [sp, #112]
	str wzr, [sp, #120]
	str xzr, [sp, #128]
	str wzr, [sp, #136]
	str xzr, [sp, #144]
	str wzr, [sp, #152]
	str xzr, [sp, #160]
	str wzr, [sp, #168]
	str xzr, [sp, #176]
	str wzr, [sp, #188]
	ands x16, x1, #0xfffffffffffffffc
	stp x16, x1, [sp, #56]
	str x0, [sp, #96]
	b.eq LBB71_76
	mov x23, #0
	ldr x9, [sp, #80]
	ldp x2, x4, [x9, #56]
	ldr x8, [x9, #48]
	ldp x9, x10, [x9, #80]
	add x15, x9, #4
	add x25, x8, #8
	ldp x5, x3, [x17]
	mov w6, #2
	add x7, sp, #112
	stp x3, x2, [sp, #40]
	str x4, [sp, #32]
	stp x10, x5, [sp, #16]
	b LBB71_4
LBB71_2:
	sub w8, w8, #1
	strh w8, [x15, #2]
LBB71_3:
	add x23, x23, #4
	add x15, x15, #8
	add x25, x25, #16
	cmp x23, x16
	b.hs LBB71_76
LBB71_4:
	cmp x23, x1
	b.hs LBB71_133
	ldrb w8, [x4, x23]
	cbz w8, LBB71_7
	mov x0, #0
	cmp x0, x3
	b.lo LBB71_9
	b LBB71_125
LBB71_7:
	cmp x23, x2
	b.hs LBB71_141
	ldur w0, [x25, #-8]
	cmp x0, x3
	b.hs LBB71_125
LBB71_9:
	ldr d0, [x5, x0, lsl #3]
	sub w9, w8, #5
	.p2align	6, , 4
	cmn w9, #4
	csel w11, wzr, w6, lo
	fcmp d0, #0.0
	ccmp w8, #0, #0, gt
	cset w10, eq
	cinc w11, w11, eq
	add x28, x23, #1
	cmp x28, x1
	b.hs LBB71_130
	add x12, x4, x23
	ldrb w12, [x12, #1]
	cbz w12, LBB71_13
	mov x0, #0
	add x20, x4, x23
	cmp x0, x3
	b.lo LBB71_15
	b LBB71_125
LBB71_13:
	cmp x28, x2
	b.hs LBB71_139
	ldur w0, [x25, #-4]
	add x20, x4, x23
	cmp x0, x3
	b.hs LBB71_125
LBB71_15:
	ldr d0, [x5, x0, lsl #3]
	fcmp d0, #0.0
	ccmp w12, #0, #0, gt
	cset w19, eq
	sub w27, w12, #5
	.p2align	6, , 4
	cmn w27, #4
	csel w12, wzr, w6, lo
	add x22, x23, #2
	cmp x22, x1
	b.hs LBB71_132
	add x13, x4, x23
	ldrb w13, [x13, #2]
	cbz w13, LBB71_19
	mov x0, #0
	add x14, x4, x23
	str x14, [sp, #72]
	cmp x0, x3
	b.lo LBB71_21
	b LBB71_125
LBB71_19:
	cmp x22, x2
	b.hs LBB71_140
	ldr w0, [x25]
	add x14, x4, x23
	str x14, [sp, #72]
	cmp x0, x3
	b.hs LBB71_125
LBB71_21:
	ldr d0, [x5, x0, lsl #3]
	sub w21, w13, #5
	.p2align	6, , 4
	cmn w21, #4
	csel w14, wzr, w6, lo
	fcmp d0, #0.0
	ccmp w13, #0, #0, gt
	cset w26, eq
	cinc w13, w14, eq
	add x24, x23, #3
	cmp x24, x1
	b.hs LBB71_131
	add x30, x4, x23
	ldrb w14, [x30, #3]
	cbz w14, LBB71_25
	mov x0, #0
	str x15, [sp, #104]
	cmp x0, x3
	b.lo LBB71_27
	b LBB71_125
LBB71_25:
	cmp x24, x2
	b.hs LBB71_138
	ldr w0, [x25, #4]
	str x15, [sp, #104]
	cmp x0, x3
	b.hs LBB71_125
LBB71_27:
	ldr d0, [x5, x0, lsl #3]
	sub w0, w14, #5
	sub w15, w14, #5
	str w15, [sp, #88]
	.p2align	6, , 4
	cmn w0, #4
	csel w15, wzr, w6, lo
	fcmp d0, #0.0
	ccmp w14, #0, #0, gt
	cset w14, eq
	str w14, [sp, #92]
	cinc w14, w15, eq
	orr w11, w11, w19
	orr w12, w12, w13
	orr w11, w11, w12
	orr w11, w11, w14
	ldr x15, [sp, #104]
	cbz w11, LBB71_3
	tbz w10, #0, LBB71_33
	ldr x0, [sp, #176]
	cmp x0, #4
	b.hs LBB71_129
	add x8, x7, x0, lsl #4
	str x23, [x8]
	mov w9, #1
	str w9, [x8, #8]
	ldr x8, [sp, #176]
	add x8, x8, #1
	str x8, [sp, #176]
	cmp x8, #4
	b.ne LBB71_39
LBB71_32:
	add x0, sp, #112
	add x4, sp, #188
	mov w1, #4
	ldr x2, [sp, #96]
	ldr x3, [sp, #80]
	str x30, [sp, #8]
	bl epidemicsim_backends::kernels::flush_batch
	ldp x17, x15, [sp, #96]
	ldr x30, [sp, #8]
	add x7, sp, #112
	mov w6, #2
	ldp x5, x4, [sp, #24]
	ldp x3, x2, [sp, #40]
	ldp x16, x1, [sp, #56]
	str xzr, [sp, #176]
	b LBB71_39
LBB71_33:
	cmn w9, #4
	b.lo LBB71_39
	ldr x9, [sp, #16]
	cmp x23, x9
	b.hs LBB71_147
	ldurh w9, [x15, #-4]
	cmp w9, #2
	b.hs LBB71_38
	cmp w8, #2
	b.ne LBB71_68
	mov w8, #5
	strb w8, [x4, x23]
	sturh wzr, [x15, #-4]
	b LBB71_39
LBB71_38:
	sub w8, w9, #1
	sturh w8, [x15, #-4]
LBB71_39:
	cbz w19, LBB71_43
	ldr x0, [sp, #176]
	cmp x0, #3
	b.hi LBB71_129
	add x8, x7, x0, lsl #4
	str x28, [x8]
	mov w9, #1
	str w9, [x8, #8]
	ldr x8, [sp, #176]
	add x8, x8, #1
	str x8, [sp, #176]
	cmp x8, #4
	b.ne LBB71_49
LBB71_42:
	add x0, sp, #112
	add x4, sp, #188
	mov w1, #4
	ldr x2, [sp, #96]
	ldr x3, [sp, #80]
	mov x19, x30
	bl epidemicsim_backends::kernels::flush_batch
	ldp x17, x15, [sp, #96]
	mov x30, x19
	add x7, sp, #112
	mov w6, #2
	ldp x5, x4, [sp, #24]
	ldp x3, x2, [sp, #40]
	ldp x16, x1, [sp, #56]
	str xzr, [sp, #176]
	b LBB71_49
LBB71_43:
	cmn w27, #4
	b.lo LBB71_49
	ldr x8, [sp, #16]
	cmp x28, x8
	b.hs LBB71_146
	ldurh w8, [x15, #-2]
	cmp w8, #2
	b.hs LBB71_48
	ldrb w8, [x20, #1]
	cmp w8, #2
	b.ne LBB71_70
	mov w8, #5
	strb w8, [x20, #1]
	sturh wzr, [x15, #-2]
	b LBB71_49
LBB71_48:
	sub w8, w8, #1
	sturh w8, [x15, #-2]
LBB71_49:
	cbz w26, LBB71_53
	ldr x0, [sp, #176]
	cmp x0, #3
	b.hi LBB71_129
	add x8, x7, x0, lsl #4
	str x22, [x8]
	mov w9, #1
	str w9, [x8, #8]
	ldr x8, [sp, #176]
	add x8, x8, #1
	str x8, [sp, #176]
	cmp x8, #4
	b.ne LBB71_59
LBB71_52:
	add x0, sp, #112
	add x4, sp, #188
	mov w1, #4
	ldr x2, [sp, #96]
	ldr x3, [sp, #80]
	mov x19, x30
	bl epidemicsim_backends::kernels::flush_batch
	ldp x17, x15, [sp, #96]
	mov x30, x19
	add x7, sp, #112
	mov w6, #2
	ldp x5, x4, [sp, #24]
	ldp x3, x2, [sp, #40]
	ldp x16, x1, [sp, #56]
	str xzr, [sp, #176]
	b LBB71_59
LBB71_53:
	cmn w21, #4
	b.lo LBB71_59
	ldr x8, [sp, #16]
	cmp x22, x8
	b.hs LBB71_144
	ldrh w8, [x15]
	cmp w8, #2
	ldr x9, [sp, #72]
	b.hs LBB71_58
	ldrb w8, [x9, #2]
	cmp w8, #2
	b.ne LBB71_72
	mov w8, #5
	strb w8, [x9, #2]
	strh wzr, [x15]
	b LBB71_59
LBB71_58:
	sub w8, w8, #1
	strh w8, [x15]
LBB71_59:
	ldr w8, [sp, #92]
	cbz w8, LBB71_63
	ldr x0, [sp, #176]
	cmp x0, #3
	b.hi LBB71_129
	add x8, x7, x0, lsl #4
	str x24, [x8]
	mov w9, #1
	str w9, [x8, #8]
	ldr x8, [sp, #176]
	add x8, x8, #1
	str x8, [sp, #176]
	cmp x8, #4
	b.ne LBB71_3
LBB71_62:
	add x0, sp, #112
	add x4, sp, #188
	mov w1, #4
	ldr x2, [sp, #96]
	ldr x3, [sp, #80]
	bl epidemicsim_backends::kernels::flush_batch
	ldp x17, x15, [sp, #96]
	add x7, sp, #112
	mov w6, #2
	ldp x5, x4, [sp, #24]
	ldp x3, x2, [sp, #40]
	ldp x16, x1, [sp, #56]
	str xzr, [sp, #176]
	b LBB71_3
LBB71_63:
	ldr w8, [sp, #88]
	cmn w8, #4
	b.lo LBB71_3
	ldr x8, [sp, #16]
	cmp x24, x8
	b.hs LBB71_145
	ldrh w8, [x15, #2]
	cmp w8, #2
	b.hs LBB71_2
	ldrb w8, [x30, #3]
	cmp w8, #2
	b.ne LBB71_74
	mov w8, #5
	strb w8, [x30, #3]
	strh wzr, [x15, #2]
	b LBB71_3
LBB71_68:
	ldr x0, [sp, #176]
	cmp x0, #4
	b.hs LBB71_129
	add x8, x7, x0, lsl #4
	str x23, [x8]
	str w6, [x8, #8]
	ldr x8, [sp, #176]
	add x8, x8, #1
	str x8, [sp, #176]
	cmp x8, #4
	b.eq LBB71_32
	b LBB71_39
LBB71_70:
	ldr x0, [sp, #176]
	cmp x0, #3
	b.hi LBB71_129
	add x8, x7, x0, lsl #4
	str x28, [x8]
	str w6, [x8, #8]
	ldr x8, [sp, #176]
	add x8, x8, #1
	str x8, [sp, #176]
	cmp x8, #4
	b.eq LBB71_42
	b LBB71_49
LBB71_72:
	ldr x0, [sp, #176]
	cmp x0, #3
	b.hi LBB71_129
	add x8, x7, x0, lsl #4
	str x22, [x8]
	str w6, [x8, #8]
	ldr x8, [sp, #176]
	add x8, x8, #1
	str x8, [sp, #176]
	cmp x8, #4
	b.eq LBB71_52
	b LBB71_59
LBB71_74:
	ldr x0, [sp, #176]
	cmp x0, #3
	b.hi LBB71_129
	add x8, x7, x0, lsl #4
	str x24, [x8]
	str w6, [x8, #8]
	ldr x8, [sp, #176]
	add x8, x8, #1
	str x8, [sp, #176]
	cmp x8, #4
	b.ne LBB71_3
	b LBB71_62
LBB71_76:
	cmp x16, x1
	b.ne LBB71_83
LBB71_77:
	ldr x1, [sp, #176]
	add x0, sp, #112
	add x4, sp, #188
	mov x19, x17
	mov x2, x17
	ldr x3, [sp, #80]
	bl epidemicsim_backends::kernels::flush_batch
	ldp x15, x1, [sp, #56]
	str xzr, [sp, #176]
	cbz x1, LBB71_109
	ldrb w8, [x19, #60]
	tbz w8, #0, LBB71_109
	ldr x9, [sp, #80]
	ldp x8, x0, [x9, #96]
	ldp x9, x14, [x9, #32]
	sub x11, x1, #1
	cmp x14, x11
	csel x10, x14, x11, lo
	cmp x10, x0
	csel x10, x10, x0, lo
	add x10, x10, #1
	cmp x10, #64
	b.ls LBB71_82
	cmp x14, x11
	csel x11, x14, x11, lo
	cmp x11, x0
	csel x11, x11, x0, lo
	add x12, x9, x11
	add x12, x12, #1
	cmp x8, x12
	b.hs LBB71_102
	add x11, x8, x11, lsl #1
	add x11, x11, #2
	cmp x9, x11
	b.hs LBB71_102
LBB71_82:
	mov x10, #0
	b LBB71_104
LBB71_83:
	ldr x8, [sp, #80]
	ldp x23, x19, [x8, #56]
	ldr x20, [x8, #48]
	ldp x21, x24, [x17]
	add x22, sp, #112
	mov w27, #1
	mov x25, x16
	ldp x28, x26, [x8, #80]
	b LBB71_86
LBB71_84:
	sub w8, w9, #1
	strh w8, [x28, x25, lsl #1]
LBB71_85:
	add x25, x25, #1
	cmp x25, x1
	b.hs LBB71_77
LBB71_86:
	ldrb w8, [x19, x25]
	cbz w8, LBB71_88
	mov x0, #0
	cmp x0, x24
	b.lo LBB71_90
	b LBB71_134
LBB71_88:
	cmp x25, x23
	b.hs LBB71_142
	ldr w0, [x20, x25, lsl #2]
	cmp x0, x24
	b.hs LBB71_134
LBB71_90:
	cbnz w8, LBB71_95
	ldr d0, [x21, x0, lsl #3]
	fcmp d0, #0.0
	b.le LBB71_95
	ldr x0, [sp, #176]
	cmp x0, #4
	b.hs LBB71_129
	add x8, x22, x0, lsl #4
	str x25, [x8]
	str w27, [x8, #8]
	ldr x8, [sp, #176]
	add x8, x8, #1
	str x8, [sp, #176]
	cmp x8, #4
	b.ne LBB71_85
LBB71_94:
	add x0, sp, #112
	add x4, sp, #188
	mov w1, #4
	ldr x2, [sp, #96]
	ldr x3, [sp, #80]
	bl epidemicsim_backends::kernels::flush_batch
	ldr x17, [sp, #96]
	ldr x1, [sp, #64]
	str xzr, [sp, #176]
	b LBB71_85
LBB71_95:
	sub w9, w8, #5
	cmn w9, #4
	b.lo LBB71_85
	cmp x25, x26
	b.hs LBB71_143
	ldrh w9, [x28, x25, lsl #1]
	cmp w9, #2
	b.hs LBB71_84
	cmp w8, #2
	b.ne LBB71_100
	mov w8, #5
	strb w8, [x19, x25]
	strh wzr, [x28, x25, lsl #1]
	b LBB71_85
LBB71_100:
	ldr x0, [sp, #176]
	cmp x0, #4
	b.hs LBB71_129
	add x8, x22, x0, lsl #4
	str x25, [x8]
	mov w9, #2
	str w9, [x8, #8]
	ldr x8, [sp, #176]
	add x8, x8, #1
	str x8, [sp, #176]
	cmp x8, #4
	b.ne LBB71_85
	b LBB71_94
LBB71_102:
	ands x11, x10, #0x3f
	mov w12, #64
	csel x11, x12, x11, eq
	sub x10, x10, x11
	add x11, x8, #64
	add x12, x9, #32
	movi.8h v0, #1
	mov x13, x10
LBB71_103:
	ldp q2, q1, [x11, #-64]
	ldp q4, q3, [x11, #-32]
	ldp q6, q5, [x11]
	ldp q16, q7, [x11, #32]
	ldp q17, q18, [x12, #-32]
	ldp q19, q20, [x12], #64
	cmtst.16b v17, v17, v17
	ushll2.8h v21, v17, #0
	and.16b v21, v21, v0
	ushll.8h v17, v17, #0
	and.16b v17, v17, v0
	cmtst.16b v18, v18, v18
	ushll2.8h v22, v18, #0
	and.16b v22, v22, v0
	ushll.8h v18, v18, #0
	and.16b v18, v18, v0
	cmtst.16b v19, v19, v19
	ushll2.8h v23, v19, #0
	and.16b v23, v23, v0
	ushll.8h v19, v19, #0
	and.16b v19, v19, v0
	cmtst.16b v20, v20, v20
	ushll2.8h v24, v20, #0
	and.16b v24, v24, v0
	ushll.8h v20, v20, #0
	and.16b v20, v20, v0
	uqadd.8h v2, v2, v17
	uqadd.8h v1, v1, v21
	uqadd.8h v4, v4, v18
	uqadd.8h v3, v3, v22
	uqadd.8h v6, v6, v19
	uqadd.8h v5, v5, v23
	uqadd.8h v16, v16, v20
	stp q2, q1, [x11, #-64]
	stp q4, q3, [x11, #-32]
	stp q6, q5, [x11]
	uqadd.8h v1, v7, v24
	stp q16, q1, [x11, #32]
	add x11, x11, #128
	subs x13, x13, #64
	b.ne LBB71_103
LBB71_104:
	mov w11, #65535
LBB71_105:
	cmp x0, x10
	b.eq LBB71_135
	cmp x14, x10
	b.eq LBB71_136
	ldrh w12, [x8, x10, lsl #1]
	ldrb w13, [x9, x10]
	cmp w13, #0
	cinc w12, w12, ne
	.p2align	6, , 4
	cmp w12, w11
	csel w12, w12, w11, lo
	strh w12, [x8, x10, lsl #1]
	add x10, x10, #1
	cmp x1, x10
	b.ne LBB71_105
LBB71_109:
	movi.2d v0, #0000000000000000
	stp q0, q0, [sp, #272]
	stp q0, q0, [sp, #240]
	stp q0, q0, [sp, #208]
	str q0, [sp, #192]
	cbz x15, LBB71_120
	mov x0, #0
	add x9, sp, #192
	add x10, x9, #28
	add x11, x9, #56
	add x12, x9, #84
	ldr x8, [sp, #80]
	ldr x13, [x8, #64]
LBB71_111:
	cmp x0, x1
	b.hs LBB71_128
	ldrb w8, [x13, x0]
	cmp x8, #7
	b.hs LBB71_126
	ldr w14, [x9, x8, lsl #2]
	add w14, w14, #1
	str w14, [x9, x8, lsl #2]
	add x8, x0, #1
	cmp x8, x1
	b.hs LBB71_127
	add x8, x13, x0
	ldrb w8, [x8, #1]
	cmp x8, #6
	b.hi LBB71_126
	ldr w14, [x10, x8, lsl #2]
	add w14, w14, #1
	str w14, [x10, x8, lsl #2]
	add x8, x0, #2
	cmp x8, x1
	b.hs LBB71_127
	add x8, x13, x0
	ldrb w8, [x8, #2]
	cmp x8, #6
	b.hi LBB71_126
	ldr w14, [x11, x8, lsl #2]
	add w14, w14, #1
	str w14, [x11, x8, lsl #2]
	add x8, x0, #3
	cmp x8, x1
	b.hs LBB71_127
	add x8, x13, x0
	ldrb w8, [x8, #3]
	cmp x8, #6
	b.hi LBB71_126
	ldr w14, [x12, x8, lsl #2]
	add w14, w14, #1
	str w14, [x12, x8, lsl #2]
	add x0, x0, #4
	cmp x0, x15
	b.lo LBB71_111
LBB71_120:
	cmp x15, x1
	b.eq LBB71_124
	ldr x8, [sp, #80]
	ldr x8, [x8, #64]
	add x9, sp, #192
LBB71_122:
	ldrb w0, [x8, x15]
	cmp x0, #6
	b.hi LBB71_137
	add x15, x15, #1
	ldr w10, [x9, x0, lsl #2]
	add w10, w10, #1
	str w10, [x9, x0, lsl #2]
	cmp x15, x1
	b.lo LBB71_122
LBB71_124:
	add x8, sp, #192
	ldr w9, [sp, #216]
	ldr w10, [sp, #244]
	add w9, w10, w9
	ldr w10, [sp, #272]
	ldr w11, [sp, #300]
	add w10, w10, w11
	add w9, w9, w10
	ldr q0, [sp, #192]
	ldur q1, [sp, #220]
	add.4s v0, v1, v0
	ldur q1, [x8, #56]
	ldur q2, [x8, #84]
	add.4s v1, v1, v2
	add.4s v0, v0, v1
	ldr d1, [sp, #208]
	ldur d2, [sp, #236]
	add.2s v1, v2, v1
	ldr d2, [sp, #264]
	ldur d3, [x8, #100]
	add.2s v2, v2, v3
	add.2s v1, v1, v2
	ldr w8, [sp, #188]
	ldr x10, [sp]
	str q0, [x10]
	str d1, [x10, #16]
	stp w9, w8, [x10, #24]
	.cfi_def_cfa wsp, 416
	ldp x29, x30, [sp, #400]
	ldp x20, x19, [sp, #384]
	ldp x22, x21, [sp, #368]
	ldp x24, x23, [sp, #352]
	ldp x26, x25, [sp, #336]
	ldp x28, x27, [sp, #320]
	add sp, sp, #416
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore w23
	.cfi_restore w24
	.cfi_restore w25
	.cfi_restore w26
	.cfi_restore w27
	.cfi_restore w28
	ret
LBB71_125:
	.cfi_restore_state
Lloh379:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.125@PAGE
Lloh380:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.125@PAGEOFF
	mov x1, x3
	bl core::panicking::panic_bounds_check
LBB71_126:
Lloh381:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.88@PAGE
Lloh382:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.88@PAGEOFF
	mov x0, x8
	mov w1, #7
	bl core::panicking::panic_bounds_check
LBB71_127:
	mov x0, x8
LBB71_128:
Lloh383:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.89@PAGE
Lloh384:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.89@PAGEOFF
	bl core::panicking::panic_bounds_check
LBB71_129:
Lloh385:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.129@PAGE
Lloh386:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.129@PAGEOFF
	mov w1, #4
	bl core::panicking::panic_bounds_check
LBB71_130:
	mov x23, x28
Lloh387:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.123@PAGE
Lloh388:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.123@PAGEOFF
	mov x0, x23
	bl core::panicking::panic_bounds_check
LBB71_131:
	mov x23, x24
Lloh389:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.123@PAGE
Lloh390:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.123@PAGEOFF
	mov x0, x23
	bl core::panicking::panic_bounds_check
LBB71_132:
	mov x23, x22
LBB71_133:
Lloh391:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.123@PAGE
Lloh392:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.123@PAGEOFF
	mov x0, x23
	bl core::panicking::panic_bounds_check
LBB71_134:
Lloh393:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.125@PAGE
Lloh394:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.125@PAGEOFF
	mov x1, x24
	bl core::panicking::panic_bounds_check
LBB71_135:
Lloh395:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.90@PAGE
Lloh396:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.90@PAGEOFF
	mov x1, x0
	bl core::panicking::panic_bounds_check
LBB71_136:
Lloh397:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.91@PAGE
Lloh398:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.91@PAGEOFF
	mov x0, x14
	mov x1, x14
	bl core::panicking::panic_bounds_check
LBB71_137:
Lloh399:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.87@PAGE
Lloh400:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.87@PAGEOFF
	mov w1, #7
	bl core::panicking::panic_bounds_check
LBB71_138:
	add x23, x23, #3
Lloh401:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.124@PAGE
Lloh402:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.124@PAGEOFF
	mov x0, x23
	ldr x1, [sp, #48]
	bl core::panicking::panic_bounds_check
LBB71_139:
	add x23, x23, #1
Lloh403:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.124@PAGE
Lloh404:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.124@PAGEOFF
	mov x0, x23
	ldr x1, [sp, #48]
	bl core::panicking::panic_bounds_check
LBB71_140:
	add x23, x23, #2
LBB71_141:
Lloh405:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.124@PAGE
Lloh406:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.124@PAGEOFF
	mov x0, x23
	ldr x1, [sp, #48]
	bl core::panicking::panic_bounds_check
LBB71_142:
Lloh407:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.124@PAGE
Lloh408:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.124@PAGEOFF
	mov x0, x25
	mov x1, x23
	bl core::panicking::panic_bounds_check
LBB71_143:
Lloh409:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.128@PAGE
Lloh410:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.128@PAGEOFF
	mov x0, x25
	mov x1, x26
	bl core::panicking::panic_bounds_check
LBB71_144:
	add x23, x23, #2
Lloh411:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.128@PAGE
Lloh412:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.128@PAGEOFF
	mov x0, x23
	ldr x1, [sp, #16]
	bl core::panicking::panic_bounds_check
LBB71_145:
	add x23, x23, #3
Lloh413:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.128@PAGE
Lloh414:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.128@PAGEOFF
	mov x0, x23
	ldr x1, [sp, #16]
	bl core::panicking::panic_bounds_check
LBB71_146:
	add x23, x23, #1
LBB71_147:
Lloh415:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.128@PAGE
Lloh416:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.128@PAGEOFF
	mov x0, x23
	ldr x1, [sp, #16]
	bl core::panicking::panic_bounds_check
	.loh AdrpAdd	Lloh379, Lloh380
	.loh AdrpAdd	Lloh381, Lloh382
	.loh AdrpAdd	Lloh383, Lloh384
	.loh AdrpAdd	Lloh385, Lloh386
	.loh AdrpAdd	Lloh387, Lloh388
	.loh AdrpAdd	Lloh389, Lloh390
	.loh AdrpAdd	Lloh391, Lloh392
	.loh AdrpAdd	Lloh393, Lloh394
	.loh AdrpAdd	Lloh395, Lloh396
	.loh AdrpAdd	Lloh397, Lloh398
	.loh AdrpAdd	Lloh399, Lloh400
	.loh AdrpAdd	Lloh401, Lloh402
	.loh AdrpAdd	Lloh403, Lloh404
	.loh AdrpAdd	Lloh405, Lloh406
	.loh AdrpAdd	Lloh407, Lloh408
	.loh AdrpAdd	Lloh409, Lloh410
	.loh AdrpAdd	Lloh411, Lloh412
	.loh AdrpAdd	Lloh413, Lloh414
	.loh AdrpAdd	Lloh415, Lloh416
	.globl	epidemicsim_backends::kernels::flush_batch
	.p2align	6
epidemicsim_backends::kernels::flush_batch:
Lfunc_begin66:
	.cfi_startproc
	stp d15, d14, [sp, #-160]!
	.cfi_def_cfa_offset 160
	stp d13, d12, [sp, #16]
	stp d11, d10, [sp, #32]
	stp d9, d8, [sp, #48]
	stp x28, x27, [sp, #64]
	stp x26, x25, [sp, #80]
	stp x24, x23, [sp, #96]
	stp x22, x21, [sp, #112]
	stp x20, x19, [sp, #128]
	stp x29, x30, [sp, #144]
	add x29, sp, #144
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset w23, -56
	.cfi_offset w24, -64
	.cfi_offset w25, -72
	.cfi_offset w26, -80
	.cfi_offset w27, -88
	.cfi_offset w28, -96
	.cfi_offset b8, -104
	.cfi_offset b9, -112
	.cfi_offset b10, -120
	.cfi_offset b11, -128
	.cfi_offset b12, -136
	.cfi_offset b13, -144
	.cfi_offset b14, -152
	.cfi_offset b15, -160
	.cfi_remember_state
	sub sp, sp, #352
	stur x4, [x29, #-200]
	mov x21, x0
	cmp x1, #4
	b.ne LBB66_14
	ldr x26, [x21]
	ldr x1, [x3, #8]
	cmp x26, x1
	b.hs LBB66_294
	ldr x27, [x21, #16]
	cmp x27, x1
	b.hs LBB66_295
	ldr x23, [x21, #32]
	cmp x23, x1
	b.hs LBB66_296
	ldr x20, [x21, #48]
	cmp x20, x1
	b.hs LBB66_297
	ldr x9, [x3]
	ldr w10, [x9, x26, lsl #2]
	ldr w8, [x9, x27, lsl #2]
	str x8, [sp, #120]
	ldr w24, [x21, #24]
	ldr w8, [x21, #40]
	str x8, [sp, #112]
	ldr w8, [x21, #56]
	str x8, [sp, #96]
	ldr w11, [x2, #56]
	ldr w8, [x21, #8]
	mov w19, #36183
	movk w19, #52638, lsl #16
	umull x12, w8, w19
	lsr x13, x12, #32
	ldr x22, [x2, #40]
	eor w4, w11, w22
	eor w11, w4, w13
	mov w25, #8019
	movk w25, #53841, lsl #16
	umull x13, w10, w25
	eor x10, x13, x22
	lsr x14, x10, #32
	lsr x10, x22, #32
	mov w15, #31161
	movk w15, #40503, lsl #16
	mov w16, #44677
	movk w16, #47975, lsl #16
	umull x11, w11, w25
	umull x14, w14, w19
	lsr x17, x14, #32
	add w5, w22, w15
	eor w12, w5, w12
	eor w12, w12, w17
	lsr x15, x11, #32
	add w6, w10, w16
	eor w13, w6, w13
	eor w13, w13, w15
	mov w15, #62322
	movk w15, #15470, lsl #16
	mov w16, #23818
	movk w16, #30415, lsl #16
	umull x12, w12, w25
	umull x13, w13, w19
	lsr x17, x13, #32
	add w7, w22, w15
	eor w14, w7, w14
	eor w14, w14, w17
	lsr x15, x12, #32
	add w30, w10, w16
	eor w11, w30, w11
	eor w11, w11, w15
	mov w15, #27947
	movk w15, #55974, lsl #16
	mov w16, #2959
	movk w16, #12855, lsl #16
	umull x14, w14, w25
	umull x11, w11, w19
	lsr x17, x11, #32
	add w15, w22, w15
	str w15, [sp, #140]
	eor w13, w15, w13
	eor w13, w13, w17
	lsr x15, x14, #32
	add w16, w10, w16
	str w16, [sp, #136]
	eor w12, w16, w12
	eor w12, w12, w15
	mov w15, #59108
	movk w15, #30941, lsl #16
	mov w16, #47636
	movk w16, #60830, lsl #16
	umull x13, w13, w25
	umull x12, w12, w19
	lsr x17, x12, #32
	add w0, w22, w15
	eor w11, w0, w11
	eor w11, w11, w17
	lsr x15, x13, #32
	add w28, w10, w16
	eor w14, w28, w14
	eor w14, w14, w15
	mov w15, #24733
	movk w15, #5909, lsl #16
	mov w16, #26777
	movk w16, #43270, lsl #16
	umull x11, w11, w25
	umull x14, w14, w19
	lsr x17, x14, #32
	add w15, w22, w15
	stur w15, [x29, #-192]
	eor w12, w15, w12
	eor w12, w12, w17
	lsr x15, x11, #32
	add w16, w10, w16
	stur w16, [x29, #-208]
	eor w13, w16, w13
	eor w13, w13, w15
	mov w15, #55894
	movk w15, #46412, lsl #16
	mov w16, #5918
	movk w16, #25710, lsl #16
	umull x12, w12, w25
	umull x13, w13, w19
	lsr x17, x13, #32
	add w15, w22, w15
	stur w15, [x29, #-216]
	eor w14, w15, w14
	eor w14, w14, w17
	lsr x15, x12, #32
	add w16, w10, w16
	stur w16, [x29, #-224]
	eor w11, w16, w11
	eor w11, w11, w15
	mov w15, #21519
	movk w15, #21380, lsl #16
	mov w16, #50595
	movk w16, #8149, lsl #16
	umull x14, w14, w25
	umull x11, w11, w19
	lsr x11, x11, #32
	add w15, w22, w15
	stur w15, [x29, #-232]
	eor w13, w15, w13
	eor w11, w13, w11
	lsr x13, x14, #32
	add w15, w10, w16
	stur w15, [x29, #-240]
	eor w12, w15, w12
	eor w12, w12, w13
	mov w13, #29736
	movk w13, #56125, lsl #16
	umull x11, w11, w25
	mul w12, w12, w19
	lsr x11, x11, #32
	add w10, w10, w13
	str w10, [sp, #248]
	eor w10, w10, w14
	eor w10, w10, w11
	mov w11, #18305
	movk w11, #36851, lsl #16
	umull x10, w10, w19
	lsr x13, x10, #32
	add w11, w22, w11
	str w11, [sp, #240]
	eor w11, w12, w11
	eor w11, w11, w13
	ldr w12, [x9, x23, lsl #2]
	str x12, [sp, #104]
	ldr w9, [x9, x20, lsl #2]
	str x9, [sp, #88]
	ucvtf d0, w11
	mov x9, #4463067230724161536
	fmov d1, x9
	ucvtf d2, w10
	ldp x11, x10, [x3, #16]
	ldp x21, x16, [x3, #64]
	ldr x9, [x3, #40]
	str x9, [sp, #224]
	ldr x9, [x3, #32]
	str x9, [sp, #232]
	ldp x9, x1, [x3, #96]
	str x9, [sp, #168]
	ldp x14, x13, [x3, #80]
	ldp x9, x12, [x3, #48]
	stp x11, x9, [sp, #208]
	ldp x9, x15, [x2, #24]
	str x9, [sp, #176]
	ldp x9, x11, [x2]
	str x9, [sp, #200]
	fmov d3, #0.50000000
	fadd d0, d0, d3
	fmul d9, d0, d1
	fadd d0, d2, d3
	fmul d8, d0, d1
	ldur x9, [x29, #-200]
	ldr w2, [x9]
	add x9, x15, #88
	str x9, [sp, #80]
	add x9, x15, #48
	str x9, [sp, #64]
	add x9, x15, #128
	str x9, [sp, #72]
	cmp w8, #1
	str x1, [sp, #128]
	str w0, [sp, #184]
	str w28, [sp, #192]
	stp x12, x10, [sp, #152]
	str x11, [sp, #144]
	b.ne LBB66_66
	cmp x26, x12
	b.hs LBB66_312
	ldr x8, [sp, #216]
	ldr w0, [x8, x26, lsl #2]
	cmp x11, x0
	b.ls LBB66_292
	ldr x8, [sp, #224]
	cmp x26, x8
	b.hs LBB66_317
	cmp x26, x1
	b.hs LBB66_319
	ldr x8, [sp, #200]
	ldr d10, [x8, x0, lsl #3]
	ldr x8, [sp, #232]
	ldrb w8, [x8, x26]
	cbz w8, LBB66_76
	ldr x8, [sp, #168]
	ldrh w0, [x8, x26, lsl #1]
	cmp w0, #1024
	b.hs LBB66_85
	ldr x1, [x15, #16]
	cmp x1, x0
	b.ls LBB66_289
	ldr x8, [x15, #8]
	ldr d0, [x8, x0, lsl #3]
	ldr x1, [sp, #128]
	fmov d1, #1.00000000
	fsub d0, d1, d0
	fmul d0, d10, d0
	fcmp d9, d0
	ldr w0, [sp, #184]
	b.mi LBB66_88
	b LBB66_92
LBB66_14:
	cmp x1, #5
	b.hs LBB66_276
	cbz x1, LBB66_275
	ldur x8, [x29, #-200]
	ldr w22, [x8]
	add x8, x21, x1, lsl #4
	stur x8, [x29, #-192]
	ldp x26, x20, [x3]
	ldr w23, [x2, #56]
	ldr x24, [x2, #48]
	ldp x8, x9, [x3, #16]
	str x9, [sp, #240]
	str x8, [sp, #208]
	ldp x27, x8, [x3, #64]
	stur x8, [x29, #-208]
	ldp x8, x25, [x2, #24]
	str x8, [sp, #184]
	ldp x8, x9, [x3, #32]
	stur x9, [x29, #-232]
	str x8, [sp, #248]
	ldp x8, x9, [x3, #96]
	stur x9, [x29, #-240]
	str x8, [sp, #192]
	ldp x8, x9, [x3, #80]
	stp x8, x9, [x29, #-224]
	fmov d8, #0.50000000
	mov x8, #4463067230724161536
	ldp x9, x10, [x3, #48]
	str x9, [sp, #216]
	fmov d9, x8
	fmov d10, #1.00000000
	ldp x8, x9, [x2]
	stp x9, x10, [sp, #224]
	str x8, [sp, #200]
	b LBB66_19
LBB66_17:
	mov w8, #0
	mov w9, #5
	strb w9, [x27, x19]
	ldur x9, [x29, #-216]
	cmp x19, x9
	b.hs LBB66_65
LBB66_18:
	ldur x9, [x29, #-224]
	strh w8, [x9, x19, lsl #1]
	add x21, x21, #16
	ldur x8, [x29, #-192]
	cmp x21, x8
	b.eq LBB66_275
LBB66_19:
	ldr x19, [x21]
	cmp x19, x20
	b.hs LBB66_277
	ldr w28, [x21, #8]
	ldr w8, [x26, x19, lsl #2]
	stp w8, w23, [x29, #-168]
	stp w28, wzr, [x29, #-160]
	sub x0, x29, #184
	sub x1, x29, #168
	mov x2, x24
	bl epidemicsim_core::rng::philox4x32_10
	ldp s0, s1, [x29, #-184]
	ucvtf d0, d0
	fadd d0, d0, d8
	fmul d12, d0, d9
	ucvtf d0, d1
	fadd d0, d0, d8
	fmul d13, d0, d9
	cmp w28, #1
	b.ne LBB66_29
	ldr x8, [sp, #232]
	cmp x19, x8
	b.hs LBB66_280
	ldp x9, x8, [sp, #216]
	ldr w0, [x9, x19, lsl #2]
	cmp x8, x0
	b.ls LBB66_283
	ldur x8, [x29, #-232]
	cmp x19, x8
	b.hs LBB66_279
	ldur x8, [x29, #-240]
	cmp x19, x8
	b.hs LBB66_282
	ldr x8, [sp, #200]
	ldr d14, [x8, x0, lsl #3]
	ldr x8, [sp, #248]
	ldrb w8, [x8, x19]
	cbz w8, LBB66_39
	ldr x8, [sp, #192]
	ldrh w0, [x8, x19, lsl #1]
	cmp w0, #1024
	b.hs LBB66_47
	ldr x1, [x25, #16]
	cmp x1, x0
	b.ls LBB66_289
	ldr x8, [x25, #8]
	ldr d0, [x8, x0, lsl #3]
	fsub d0, d10, d0
	fmul d0, d14, d0
	fcmp d12, d0
	b.pl LBB66_52
	b LBB66_48
LBB66_29:
	ldr x8, [sp, #240]
	cmp x19, x8
	b.hs LBB66_281
	ldur x8, [x29, #-208]
	cmp x19, x8
	b.hs LBB66_278
	ldr x8, [sp, #208]
	ldrb w28, [x8, x19]
	ldrb w8, [x27, x19]
	cmp w8, #1
	b.eq LBB66_40
	cmp w8, #3
	b.ne LBB66_45
	ldur x8, [x29, #-232]
	cmp x19, x8
	b.hs LBB66_290
	ldur x8, [x29, #-240]
	cmp x19, x8
	b.hs LBB66_291
	ldr x8, [sp, #248]
	ldrb w8, [x8, x19]
	cbz w8, LBB66_56
	ldr x8, [sp, #192]
	ldrh w0, [x8, x19, lsl #1]
	cmp w0, #1024
	b.hs LBB66_57
	ldr x1, [x25, #40]
	cmp x1, x0
	b.ls LBB66_293
	ldr x8, [x25, #32]
	ldr d0, [x8, x0, lsl #3]
	cmp w28, #5
	b.lo LBB66_58
	b LBB66_287
LBB66_39:
	movi.2d v0, #0000000000000000
	fsub d0, d10, d0
	fmul d0, d14, d0
	fcmp d12, d0
	b.pl LBB66_52
	b LBB66_48
LBB66_40:
	cmp w28, #5
	b.hs LBB66_288
	add x8, x25, #48
	ldr d0, [x8, x28, lsl #3]
	fcmp d12, d0
	b.pl LBB66_53
	ldr d0, [x25, #168]
	fcmp d0, #0.0
	b.le LBB66_62
	fadd d1, d13, d8
	fmul d0, d1, d0
	frinta d0, d0
	fmaxnm d0, d0, d10
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	mov w9, #2
	strb w9, [x27, x19]
	ldur x9, [x29, #-216]
	cmp x19, x9
	b.lo LBB66_18
	b LBB66_65
LBB66_45:
	cmp w28, #5
	b.hs LBB66_286
	mov w8, #0
	add x9, x25, #128
	ldr d0, [x9, x28, lsl #3]
	fcmp d12, d0
	mov w9, #5
	mov w10, #6
	csel w9, w10, w9, mi
	strb w9, [x27, x19]
	ldur x9, [x29, #-216]
	cmp x19, x9
	b.lo LBB66_18
	b LBB66_65
LBB66_47:
	ucvtf d0, w0
	ldr x8, [sp, #184]
	ldr d15, [x8]
	ldr d1, [x8, #16]
	fcmp d1, #0.0
	fdiv d2, d0, d1
	fminnm d2, d2, d10
	fcsel d11, d2, d10, gt
	fsub d0, d0, d1
	movi.2d v1, #0000000000000000
	fmaxnm d0, d0, d1
	ldr d1, [x8, #24]
	fneg d0, d0
	fdiv d0, d0, d1
	bl _exp2
	fmul d0, d11, d0
	fmul d0, d15, d0
	fsub d0, d10, d0
	fmul d0, d14, d0
	fcmp d12, d0
	b.pl LBB66_52
LBB66_48:
	ldur x8, [x29, #-208]
	cmp x19, x8
	b.hs LBB66_285
	mov w8, #1
	strb w8, [x27, x19]
	ldur x8, [x29, #-216]
	cmp x19, x8
	b.hs LBB66_284
	ldr d1, [x25, #192]
	fadd d2, d13, d8
	fmul d1, d2, d1
	frinta d1, d1
	fmaxnm d1, d1, d10
	fcvtzu w8, d1
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	ldur x9, [x29, #-224]
	strh w8, [x9, x19, lsl #1]
LBB66_52:
	fcmp d12, d0
	cinc w22, w22, mi
	ldp x9, x8, [x29, #-200]
	str w22, [x9]
	add x21, x21, #16
	cmp x21, x8
	b.ne LBB66_19
	b LBB66_275
LBB66_53:
	ldr d0, [x25, #176]
	fcmp d0, #0.0
	b.le LBB66_63
	fadd d1, d13, d8
	fmul d0, d1, d0
	frinta d0, d0
	fmaxnm d0, d0, d10
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	mov w9, #3
	strb w9, [x27, x19]
	ldur x9, [x29, #-216]
	cmp x19, x9
	b.lo LBB66_18
	b LBB66_65
LBB66_56:
	movi.2d v0, #0000000000000000
	cmp w28, #5
	b.lo LBB66_58
	b LBB66_287
LBB66_57:
	ucvtf d0, w0
	ldr x8, [sp, #184]
	ldp d14, d1, [x8, #8]
	fcmp d1, #0.0
	fdiv d2, d0, d1
	fminnm d2, d2, d10
	fcsel d11, d2, d10, gt
	fsub d0, d0, d1
	movi.2d v1, #0000000000000000
	fmaxnm d0, d0, d1
	ldr d1, [x8, #24]
	fneg d0, d0
	fdiv d0, d0, d1
	bl _exp2
	fmul d0, d11, d0
	fmul d0, d14, d0
	cmp w28, #5
	b.hs LBB66_287
LBB66_58:
	add x8, x25, #88
	ldr d1, [x8, x28, lsl #3]
	fsub d0, d10, d0
	fmul d0, d0, d1
	fcmp d12, d0
	b.pl LBB66_17
	ldr d0, [x25, #184]
	fcmp d0, #0.0
	b.le LBB66_64
	fadd d1, d13, d8
	fmul d0, d1, d0
	frinta d0, d0
	fmaxnm d0, d0, d10
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	mov w9, #4
	strb w9, [x27, x19]
	ldur x9, [x29, #-216]
	cmp x19, x9
	b.lo LBB66_18
	b LBB66_65
LBB66_62:
	mov w8, #0
	mov w9, #2
	strb w9, [x27, x19]
	ldur x9, [x29, #-216]
	cmp x19, x9
	b.lo LBB66_18
	b LBB66_65
LBB66_63:
	mov w8, #0
	mov w9, #3
	strb w9, [x27, x19]
	ldur x9, [x29, #-216]
	cmp x19, x9
	b.lo LBB66_18
	b LBB66_65
LBB66_64:
	mov w8, #0
	mov w9, #4
	strb w9, [x27, x19]
	ldur x9, [x29, #-216]
	cmp x19, x9
	b.lo LBB66_18
LBB66_65:
Lloh157:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.71@PAGE
Lloh158:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.71@PAGEOFF
	mov x0, x19
	ldur x1, [x29, #-216]
	bl core::panicking::panic_bounds_check
LBB66_66:
	cmp x26, x10
	b.hs LBB66_314
	cmp x26, x16
	b.hs LBB66_298
	ldr x8, [sp, #208]
	ldrb w28, [x8, x26]
	ldrb w8, [x21, x26]
	cmp w8, #1
	b.eq LBB66_77
	cmp w8, #3
	b.ne LBB66_82
	ldr x8, [sp, #224]
	cmp x26, x8
	b.hs LBB66_330
	cmp x26, x1
	b.hs LBB66_332
	ldr x8, [sp, #232]
	ldrb w8, [x8, x26]
	cbz w8, LBB66_134
	ldr x8, [sp, #168]
	ldrh w0, [x8, x26, lsl #1]
	cmp w0, #1024
	b.hs LBB66_136
	ldr x1, [x15, #40]
	cmp x1, x0
	b.ls LBB66_293
	ldr x8, [x15, #32]
	ldr d0, [x8, x0, lsl #3]
	b LBB66_139
LBB66_76:
	movi.2d v0, #0000000000000000
	fmov d1, #1.00000000
	fsub d0, d1, d0
	fmul d0, d10, d0
	fcmp d9, d0
	ldr w0, [sp, #184]
	b.mi LBB66_88
	b LBB66_92
LBB66_77:
	cmp w28, #5
	b.hs LBB66_288
	add x8, x15, #48
	ldr d0, [x8, x28, lsl #3]
	fcmp d9, d0
	b.pl LBB66_128
	ldr d0, [x15, #168]
	fcmp d0, #0.0
	ldr w28, [sp, #192]
	b.le LBB66_260
	fmov d1, #0.50000000
	fadd d1, d8, d1
	fmul d0, d1, d0
	frinta d0, d0
	fmov d1, #1.00000000
	fmaxnm d0, d0, d1
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	mov w9, #2
	strb w9, [x21, x26]
	cmp x26, x13
	b.lo LBB66_84
	b LBB66_146
LBB66_82:
	cmp w28, #5
	b.hs LBB66_286
	mov w8, #0
	add x9, x15, #128
	ldr d0, [x9, x28, lsl #3]
	fcmp d9, d0
	mov w9, #5
	mov w10, #6
	csel w9, w10, w9, mi
	ldr w28, [sp, #192]
	strb w9, [x21, x26]
	cmp x26, x13
	b.hs LBB66_146
LBB66_84:
	strh w8, [x14, x26, lsl #1]
	b LBB66_93
LBB66_85:
	stp x24, x15, [sp, #8]
	stp w30, w7, [sp, #24]
	stp w6, w5, [sp, #32]
	stp w2, w4, [sp, #40]
	stp x20, x16, [sp, #48]
	mov x24, x21
	mov x21, x14
	mov x28, x13
	ldr x8, [sp, #176]
	ldr d11, [x8]
	ucvtf d0, w0
	ldr d1, [x8, #16]
	fmov d12, #1.00000000
	fcmp d1, #0.0
	b.le LBB66_87
	fdiv d2, d0, d1
	fmov d3, #1.00000000
	fminnm d12, d2, d3
LBB66_87:
	fsub d0, d0, d1
	movi.2d v1, #0000000000000000
	fmaxnm d0, d0, d1
	ldr x8, [sp, #176]
	ldr d1, [x8, #24]
	fneg d0, d0
	fdiv d0, d0, d1
	bl _exp2
	fmul d0, d12, d0
	fmul d0, d11, d0
	mov x13, x28
	mov x14, x21
	mov x21, x24
	ldp x24, x15, [sp, #8]
	ldp x20, x16, [sp, #48]
	ldp w2, w4, [sp, #40]
	ldp w6, w5, [sp, #32]
	ldp w30, w7, [sp, #24]
	ldr x1, [sp, #128]
	ldr w28, [sp, #192]
	fmov d1, #1.00000000
	fsub d0, d1, d0
	fmul d0, d10, d0
	fcmp d9, d0
	ldr w0, [sp, #184]
	b.pl LBB66_92
LBB66_88:
	cmp x26, x16
	b.hs LBB66_321
	mov w8, #1
	strb w8, [x21, x26]
	cmp x26, x13
	b.hs LBB66_322
	ldr d1, [x15, #192]
	fmov d2, #0.50000000
	fadd d2, d8, d2
	fmul d1, d2, d1
	frinta d1, d1
	fmov d2, #1.00000000
	fmaxnm d1, d1, d2
	fcvtzu w8, d1
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	strh w8, [x14, x26, lsl #1]
LBB66_92:
	fcmp d9, d0
	cinc w2, w2, mi
	ldur x8, [x29, #-200]
	str w2, [x8]
LBB66_93:
	umull x8, w24, w19
	lsr x9, x8, #32
	eor w9, w4, w9
	ldr x10, [sp, #120]
	umull x10, w10, w25
	eor x11, x10, x22
	lsr x11, x11, #32
	umull x9, w9, w25
	umull x11, w11, w19
	lsr x12, x11, #32
	eor w8, w5, w8
	eor w8, w8, w12
	lsr x12, x9, #32
	eor w10, w6, w10
	eor w10, w10, w12
	umull x8, w8, w25
	umull x10, w10, w19
	lsr x12, x10, #32
	eor w11, w7, w11
	eor w11, w11, w12
	lsr x12, x8, #32
	eor w9, w30, w9
	eor w9, w9, w12
	umull x11, w11, w25
	umull x9, w9, w19
	lsr x12, x9, #32
	ldp w26, w3, [sp, #136]
	eor w10, w3, w10
	eor w10, w10, w12
	lsr x12, x11, #32
	eor w8, w26, w8
	eor w8, w8, w12
	umull x10, w10, w25
	umull x8, w8, w19
	lsr x12, x8, #32
	eor w9, w0, w9
	eor w9, w9, w12
	lsr x12, x10, #32
	eor w11, w28, w11
	eor w11, w11, w12
	umull x9, w9, w25
	umull x11, w11, w19
	lsr x12, x11, #32
	mov x17, x24
	ldur w24, [x29, #-192]
	eor w8, w24, w8
	eor w8, w8, w12
	lsr x12, x9, #32
	ldur w24, [x29, #-208]
	eor w10, w24, w10
	eor w10, w10, w12
	umull x8, w8, w25
	umull x10, w10, w19
	lsr x12, x10, #32
	ldur w24, [x29, #-216]
	eor w11, w24, w11
	eor w11, w11, w12
	lsr x12, x8, #32
	ldur w24, [x29, #-224]
	eor w9, w24, w9
	eor w9, w9, w12
	umull x11, w11, w25
	umull x9, w9, w19
	lsr x9, x9, #32
	ldur w12, [x29, #-232]
	eor w10, w12, w10
	eor w9, w10, w9
	lsr x10, x11, #32
	ldur w12, [x29, #-240]
	eor w8, w12, w8
	eor w8, w8, w10
	umull x9, w9, w25
	mul w8, w8, w19
	lsr x9, x9, #32
	ldr w10, [sp, #248]
	eor w10, w10, w11
	eor w9, w10, w9
	umull x9, w9, w19
	lsr x10, x9, #32
	ldr w11, [sp, #240]
	eor w8, w8, w11
	eor w8, w8, w10
	ucvtf d0, w8
	fmov d1, #0.50000000
	fadd d0, d0, d1
	mov x8, #4463067230724161536
	fmov d2, x8
	fmul d9, d0, d2
	ucvtf d0, w9
	fadd d0, d0, d1
	fmul d8, d0, d2
	cmp w17, #1
	b.ne LBB66_102
	ldr x8, [sp, #152]
	cmp x27, x8
	b.hs LBB66_299
	ldr x8, [sp, #216]
	ldr w0, [x8, x27, lsl #2]
	ldr x8, [sp, #144]
	cmp x8, x0
	b.ls LBB66_292
	ldr x8, [sp, #224]
	cmp x27, x8
	b.hs LBB66_302
	cmp x27, x1
	b.hs LBB66_303
	ldr x8, [sp, #200]
	ldr d10, [x8, x0, lsl #3]
	ldr x8, [sp, #232]
	ldrb w8, [x8, x27]
	cbz w8, LBB66_112
	ldr x8, [sp, #168]
	ldrh w0, [x8, x27, lsl #1]
	cmp w0, #1024
	b.hs LBB66_120
	ldr x1, [x15, #16]
	cmp x1, x0
	b.ls LBB66_289
	ldr x8, [x15, #8]
	ldr d0, [x8, x0, lsl #3]
	ldr x1, [sp, #128]
	fmov d1, #1.00000000
	fsub d0, d1, d0
	fmul d0, d10, d0
	fcmp d9, d0
	ldr w0, [sp, #184]
	b.mi LBB66_123
	b LBB66_127
LBB66_102:
	ldr x8, [sp, #160]
	cmp x27, x8
	b.hs LBB66_300
	cmp x27, x16
	b.hs LBB66_301
	ldr x8, [sp, #208]
	ldrb w28, [x8, x27]
	ldrb w8, [x21, x27]
	cmp w8, #1
	b.eq LBB66_113
	cmp w8, #3
	b.ne LBB66_118
	ldr x8, [sp, #224]
	cmp x27, x8
	b.hs LBB66_329
	cmp x27, x1
	b.hs LBB66_331
	ldr x8, [sp, #232]
	ldrb w8, [x8, x27]
	cbz w8, LBB66_135
	ldr x8, [sp, #168]
	ldrh w0, [x8, x27, lsl #1]
	cmp w0, #1024
	b.hs LBB66_147
	ldr x1, [x15, #40]
	cmp x1, x0
	b.ls LBB66_293
	ldr x8, [x15, #32]
	ldr d0, [x8, x0, lsl #3]
	ldr x1, [sp, #128]
	b LBB66_150
LBB66_112:
	movi.2d v0, #0000000000000000
	fmov d1, #1.00000000
	fsub d0, d1, d0
	fmul d0, d10, d0
	fcmp d9, d0
	ldr w0, [sp, #184]
	b.mi LBB66_123
	b LBB66_127
LBB66_113:
	cmp w28, #4
	b.hi LBB66_288
	ldr x8, [sp, #64]
	ldr d0, [x8, x28, lsl #3]
	fcmp d9, d0
	b.pl LBB66_131
	ldr d0, [x15, #168]
	fcmp d0, #0.0
	ldr w28, [sp, #192]
	b.le LBB66_263
	fadd d1, d8, d1
	fmul d0, d1, d0
	frinta d0, d0
	fmov d1, #1.00000000
	fmaxnm d0, d0, d1
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	mov w9, #2
	b LBB66_158
LBB66_118:
	cmp w28, #4
	b.hi LBB66_286
	mov w8, #0
	ldr x9, [sp, #72]
	ldr d0, [x9, x28, lsl #3]
	fcmp d9, d0
	mov w9, #5
	mov w10, #6
	csel w9, w10, w9, mi
	b LBB66_157
LBB66_120:
	stp w30, w7, [sp, #24]
	stp w6, w5, [sp, #32]
	stp w2, w4, [sp, #40]
	stp x20, x16, [sp, #48]
	str x15, [sp, #16]
	mov x24, x21
	mov x21, x14
	mov x26, x13
	ldr x8, [sp, #176]
	ldr d11, [x8]
	ucvtf d0, w0
	ldr d1, [x8, #16]
	fmov d12, #1.00000000
	fcmp d1, #0.0
	b.le LBB66_122
	fdiv d2, d0, d1
	fmov d3, #1.00000000
	fminnm d12, d2, d3
LBB66_122:
	fsub d0, d0, d1
	movi.2d v1, #0000000000000000
	fmaxnm d0, d0, d1
	ldr x8, [sp, #176]
	ldr d1, [x8, #24]
	fneg d0, d0
	fdiv d0, d0, d1
	bl _exp2
	fmul d0, d12, d0
	fmul d0, d11, d0
	mov x13, x26
	mov x14, x21
	mov x21, x24
	ldr x15, [sp, #16]
	ldp x20, x16, [sp, #48]
	ldp w2, w4, [sp, #40]
	ldp w6, w5, [sp, #32]
	ldp w30, w7, [sp, #24]
	ldr x1, [sp, #128]
	ldp w26, w3, [sp, #136]
	ldr w28, [sp, #192]
	fmov d1, #1.00000000
	fsub d0, d1, d0
	fmul d0, d10, d0
	fcmp d9, d0
	ldr w0, [sp, #184]
	b.pl LBB66_127
LBB66_123:
	cmp x27, x16
	b.hs LBB66_323
	mov w8, #1
	strb w8, [x21, x27]
	cmp x27, x13
	b.hs LBB66_324
	ldr d1, [x15, #192]
	fmov d2, #0.50000000
	fadd d2, d8, d2
	fmul d1, d2, d1
	frinta d1, d1
	fmov d2, #1.00000000
	fmaxnm d1, d1, d2
	fcvtzu w8, d1
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	strh w8, [x14, x27, lsl #1]
LBB66_127:
	fcmp d9, d0
	cinc w2, w2, mi
	ldur x8, [x29, #-200]
	str w2, [x8]
	b LBB66_160
LBB66_128:
	ldr d0, [x15, #176]
	fcmp d0, #0.0
	ldr w28, [sp, #192]
	b.le LBB66_261
	fmov d1, #0.50000000
	fadd d1, d8, d1
	fmul d0, d1, d0
	frinta d0, d0
	fmov d1, #1.00000000
	fmaxnm d0, d0, d1
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	mov w9, #3
	strb w9, [x21, x26]
	cmp x26, x13
	b.lo LBB66_84
	b LBB66_146
LBB66_131:
	ldr d0, [x15, #176]
	fcmp d0, #0.0
	ldr w28, [sp, #192]
	b.le LBB66_264
	fadd d1, d8, d1
	fmul d0, d1, d0
	frinta d0, d0
	fmov d1, #1.00000000
	fmaxnm d0, d0, d1
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	mov w9, #3
	b LBB66_158
LBB66_134:
	movi.2d v0, #0000000000000000
	b LBB66_140
LBB66_135:
	movi.2d v0, #0000000000000000
	b LBB66_151
LBB66_136:
	stp w30, w7, [sp, #24]
	stp w6, w5, [sp, #32]
	stp w2, w4, [sp, #40]
	stp x20, x16, [sp, #48]
	stp x14, x15, [sp, #8]
	str x13, [sp]
	ucvtf d0, w0
	ldr x8, [sp, #176]
	ldp d10, d1, [x8, #8]
	fmov d11, #1.00000000
	fcmp d1, #0.0
	b.le LBB66_138
	fdiv d2, d0, d1
	fmov d3, #1.00000000
	fminnm d11, d2, d3
LBB66_138:
	fsub d0, d0, d1
	movi.2d v1, #0000000000000000
	fmaxnm d0, d0, d1
	ldr x8, [sp, #176]
	ldr d1, [x8, #24]
	fneg d0, d0
	fdiv d0, d0, d1
	bl _exp2
	fmul d0, d11, d0
	fmul d0, d10, d0
	ldp x13, x14, [sp]
	ldr x15, [sp, #16]
	ldp x20, x16, [sp, #48]
	ldp w2, w4, [sp, #40]
	ldp w6, w5, [sp, #32]
	ldp w30, w7, [sp, #24]
LBB66_139:
	ldr x1, [sp, #128]
	ldr w0, [sp, #184]
LBB66_140:
	cmp w28, #5
	b.hs LBB66_287
	ldr x8, [sp, #80]
	ldr d1, [x8, x28, lsl #3]
	fmov d2, #1.00000000
	fsub d0, d2, d0
	fmul d0, d0, d1
	fcmp d9, d0
	b.pl LBB66_145
	ldr d0, [x15, #184]
	fcmp d0, #0.0
	ldr w28, [sp, #192]
	b.le LBB66_262
	fmov d1, #0.50000000
	fadd d1, d8, d1
	fmul d0, d1, d0
	frinta d0, d0
	fmov d1, #1.00000000
	fmaxnm d0, d0, d1
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	mov w9, #4
	strb w9, [x21, x26]
	cmp x26, x13
	b.lo LBB66_84
	b LBB66_146
LBB66_145:
	mov w8, #0
	mov w9, #5
	ldr w28, [sp, #192]
	strb w9, [x21, x26]
	cmp x26, x13
	b.lo LBB66_84
LBB66_146:
	mov x1, x13
Lloh159:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.71@PAGE
Lloh160:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.71@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_147:
	stp w30, w7, [sp, #24]
	stp w6, w5, [sp, #32]
	stp w2, w4, [sp, #40]
	stp x20, x16, [sp, #48]
	stp x14, x15, [sp, #8]
	mov x26, x13
	ucvtf d0, w0
	ldr x8, [sp, #176]
	ldp d10, d1, [x8, #8]
	fmov d11, #1.00000000
	fcmp d1, #0.0
	b.le LBB66_149
	fdiv d2, d0, d1
	fmov d3, #1.00000000
	fminnm d11, d2, d3
LBB66_149:
	fsub d0, d0, d1
	movi.2d v1, #0000000000000000
	fmaxnm d0, d0, d1
	ldr x8, [sp, #176]
	ldr d1, [x8, #24]
	fneg d0, d0
	fdiv d0, d0, d1
	bl _exp2
	fmul d0, d11, d0
	fmul d0, d10, d0
	mov x13, x26
	ldp x14, x15, [sp, #8]
	ldp x20, x16, [sp, #48]
	ldp w2, w4, [sp, #40]
	ldp w6, w5, [sp, #32]
	ldp w30, w7, [sp, #24]
	ldr x1, [sp, #128]
	ldp w26, w3, [sp, #136]
LBB66_150:
	ldr w0, [sp, #184]
LBB66_151:
	cmp w28, #4
	b.hi LBB66_287
	ldr x8, [sp, #80]
	ldr d1, [x8, x28, lsl #3]
	fmov d2, #1.00000000
	fsub d0, d2, d0
	fmul d0, d0, d1
	fcmp d9, d0
	b.pl LBB66_156
	ldr d0, [x15, #184]
	fcmp d0, #0.0
	ldr w28, [sp, #192]
	b.le LBB66_265
	fmov d1, #0.50000000
	fadd d1, d8, d1
	fmul d0, d1, d0
	frinta d0, d0
	fmov d1, #1.00000000
	fmaxnm d0, d0, d1
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	mov w9, #4
	b LBB66_158
LBB66_156:
	mov w8, #0
	mov w9, #5
LBB66_157:
	ldr w28, [sp, #192]
LBB66_158:
	strb w9, [x21, x27]
	cmp x27, x13
	b.hs LBB66_304
	strh w8, [x14, x27, lsl #1]
LBB66_160:
	ldp x10, x27, [sp, #104]
	umull x8, w27, w19
	lsr x9, x8, #32
	eor w9, w4, w9
	umull x10, w10, w25
	eor x11, x10, x22
	lsr x11, x11, #32
	umull x9, w9, w25
	umull x11, w11, w19
	lsr x12, x11, #32
	eor w8, w5, w8
	eor w8, w8, w12
	lsr x12, x9, #32
	eor w10, w6, w10
	eor w10, w10, w12
	umull x8, w8, w25
	umull x10, w10, w19
	lsr x12, x10, #32
	eor w11, w7, w11
	eor w11, w11, w12
	lsr x12, x8, #32
	eor w9, w30, w9
	eor w9, w9, w12
	umull x11, w11, w25
	umull x9, w9, w19
	lsr x12, x9, #32
	eor w10, w3, w10
	eor w10, w10, w12
	lsr x12, x11, #32
	eor w8, w26, w8
	eor w8, w8, w12
	umull x10, w10, w25
	umull x8, w8, w19
	lsr x12, x8, #32
	eor w9, w0, w9
	eor w9, w9, w12
	lsr x12, x10, #32
	eor w11, w28, w11
	eor w11, w11, w12
	umull x9, w9, w25
	umull x11, w11, w19
	lsr x12, x11, #32
	ldur w24, [x29, #-192]
	eor w8, w24, w8
	eor w8, w8, w12
	lsr x12, x9, #32
	ldur w24, [x29, #-208]
	eor w10, w24, w10
	eor w10, w10, w12
	umull x8, w8, w25
	umull x10, w10, w19
	lsr x12, x10, #32
	ldur w24, [x29, #-216]
	eor w11, w24, w11
	eor w11, w11, w12
	lsr x12, x8, #32
	ldur w24, [x29, #-224]
	eor w9, w24, w9
	eor w9, w9, w12
	umull x11, w11, w25
	umull x9, w9, w19
	lsr x9, x9, #32
	ldur w12, [x29, #-232]
	eor w10, w12, w10
	eor w9, w10, w9
	lsr x10, x11, #32
	ldur w12, [x29, #-240]
	eor w8, w12, w8
	eor w8, w8, w10
	umull x9, w9, w25
	mul w8, w8, w19
	lsr x9, x9, #32
	ldr w10, [sp, #248]
	eor w10, w10, w11
	eor w9, w10, w9
	umull x9, w9, w19
	lsr x10, x9, #32
	ldr w11, [sp, #240]
	eor w8, w8, w11
	eor w8, w8, w10
	ucvtf d0, w8
	fmov d1, #0.50000000
	fadd d0, d0, d1
	mov x8, #4463067230724161536
	fmov d2, x8
	fmul d9, d0, d2
	ucvtf d0, w9
	fadd d0, d0, d1
	fmul d8, d0, d2
	cmp w27, #1
	ldr x27, [sp, #224]
	b.ne LBB66_169
	ldp x8, x24, [sp, #152]
	cmp x23, x8
	b.hs LBB66_305
	ldr x8, [sp, #216]
	ldr w0, [x8, x23, lsl #2]
	ldr x8, [sp, #144]
	cmp x8, x0
	b.ls LBB66_292
	cmp x23, x27
	b.hs LBB66_308
	cmp x23, x1
	b.hs LBB66_309
	ldr x8, [sp, #200]
	ldr d10, [x8, x0, lsl #3]
	ldr x8, [sp, #232]
	ldrb w8, [x8, x23]
	cbz w8, LBB66_179
	ldr x8, [sp, #168]
	ldrh w0, [x8, x23, lsl #1]
	cmp w0, #1024
	b.hs LBB66_187
	ldr x1, [x15, #16]
	cmp x1, x0
	b.ls LBB66_289
	ldr x8, [x15, #8]
	ldr d0, [x8, x0, lsl #3]
	ldr x1, [sp, #128]
	fmov d1, #1.00000000
	fsub d0, d1, d0
	fmul d0, d10, d0
	fcmp d9, d0
	ldr w0, [sp, #184]
	b.mi LBB66_190
	b LBB66_194
LBB66_169:
	ldr x24, [sp, #160]
	cmp x23, x24
	b.hs LBB66_306
	cmp x23, x16
	b.hs LBB66_307
	ldr x8, [sp, #208]
	ldrb w28, [x8, x23]
	ldrb w8, [x21, x23]
	cmp w8, #1
	b.eq LBB66_180
	cmp w8, #3
	b.ne LBB66_185
	cmp x23, x27
	b.hs LBB66_333
	cmp x23, x1
	b.hs LBB66_334
	ldr x8, [sp, #232]
	ldrb w8, [x8, x23]
	cbz w8, LBB66_198
	ldr x8, [sp, #168]
	ldrh w0, [x8, x23, lsl #1]
	cmp w0, #1024
	b.hs LBB66_199
	ldr x1, [x15, #40]
	cmp x1, x0
	b.ls LBB66_293
	ldr x8, [x15, #32]
	ldr d0, [x8, x0, lsl #3]
	ldr x1, [sp, #128]
	b LBB66_202
LBB66_179:
	movi.2d v0, #0000000000000000
	fmov d1, #1.00000000
	fsub d0, d1, d0
	fmul d0, d10, d0
	fcmp d9, d0
	ldr w0, [sp, #184]
	b.mi LBB66_190
	b LBB66_194
LBB66_180:
	cmp w28, #4
	b.hi LBB66_288
	ldr x8, [sp, #64]
	ldr d0, [x8, x28, lsl #3]
	fcmp d9, d0
	b.pl LBB66_195
	ldr d0, [x15, #168]
	fcmp d0, #0.0
	ldr w28, [sp, #192]
	b.le LBB66_266
	fadd d1, d8, d1
	fmul d0, d1, d0
	frinta d0, d0
	fmov d1, #1.00000000
	fmaxnm d0, d0, d1
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	mov w9, #2
	b LBB66_210
LBB66_185:
	cmp w28, #4
	b.hi LBB66_286
	mov w8, #0
	ldr x9, [sp, #72]
	ldr d0, [x9, x28, lsl #3]
	fcmp d9, d0
	mov w9, #5
	mov w10, #6
	csel w9, w10, w9, mi
	b LBB66_209
LBB66_187:
	stp w30, w7, [sp, #24]
	stp w6, w5, [sp, #32]
	stp w2, w4, [sp, #40]
	stp x20, x16, [sp, #48]
	mov x28, x15
	mov x24, x21
	mov x21, x14
	mov x26, x13
	ldr x8, [sp, #176]
	ldr d11, [x8]
	ucvtf d0, w0
	ldr d1, [x8, #16]
	fmov d12, #1.00000000
	fcmp d1, #0.0
	b.le LBB66_189
	fdiv d2, d0, d1
	fmov d3, #1.00000000
	fminnm d12, d2, d3
LBB66_189:
	fsub d0, d0, d1
	movi.2d v1, #0000000000000000
	fmaxnm d0, d0, d1
	ldr x8, [sp, #176]
	ldr d1, [x8, #24]
	fneg d0, d0
	fdiv d0, d0, d1
	bl _exp2
	fmul d0, d12, d0
	fmul d0, d11, d0
	mov x13, x26
	mov x14, x21
	mov x21, x24
	mov x15, x28
	ldp x20, x16, [sp, #48]
	ldp w2, w4, [sp, #40]
	ldp w6, w5, [sp, #32]
	ldp w30, w7, [sp, #24]
	ldr x1, [sp, #128]
	ldp w26, w3, [sp, #136]
	ldr x27, [sp, #224]
	ldr w28, [sp, #192]
	ldr x24, [sp, #160]
	fmov d1, #1.00000000
	fsub d0, d1, d0
	fmul d0, d10, d0
	fcmp d9, d0
	ldr w0, [sp, #184]
	b.pl LBB66_194
LBB66_190:
	cmp x23, x16
	b.hs LBB66_325
	mov w8, #1
	strb w8, [x21, x23]
	cmp x23, x13
	b.hs LBB66_326
	ldr d1, [x15, #192]
	fmov d2, #0.50000000
	fadd d2, d8, d2
	fmul d1, d2, d1
	frinta d1, d1
	fmov d2, #1.00000000
	fmaxnm d1, d1, d2
	fcvtzu w8, d1
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	strh w8, [x14, x23, lsl #1]
LBB66_194:
	fcmp d9, d0
	cinc w2, w2, mi
	ldur x8, [x29, #-200]
	str w2, [x8]
	b LBB66_212
LBB66_195:
	ldr d0, [x15, #176]
	fcmp d0, #0.0
	ldr w28, [sp, #192]
	b.le LBB66_267
	fadd d1, d8, d1
	fmul d0, d1, d0
	frinta d0, d0
	fmov d1, #1.00000000
	fmaxnm d0, d0, d1
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	mov w9, #3
	b LBB66_210
LBB66_198:
	movi.2d v0, #0000000000000000
	b LBB66_203
LBB66_199:
	stp w30, w7, [sp, #24]
	stp w6, w5, [sp, #32]
	stp w2, w4, [sp, #40]
	stp x20, x16, [sp, #48]
	str x15, [sp, #16]
	mov x27, x14
	mov x26, x13
	ucvtf d0, w0
	ldr x8, [sp, #176]
	ldp d10, d1, [x8, #8]
	fmov d11, #1.00000000
	fcmp d1, #0.0
	b.le LBB66_201
	fdiv d2, d0, d1
	fmov d3, #1.00000000
	fminnm d11, d2, d3
LBB66_201:
	fsub d0, d0, d1
	movi.2d v1, #0000000000000000
	fmaxnm d0, d0, d1
	ldr x8, [sp, #176]
	ldr d1, [x8, #24]
	fneg d0, d0
	fdiv d0, d0, d1
	bl _exp2
	fmul d0, d11, d0
	fmul d0, d10, d0
	mov x13, x26
	mov x14, x27
	ldr x15, [sp, #16]
	ldp x20, x16, [sp, #48]
	ldp w2, w4, [sp, #40]
	ldp w6, w5, [sp, #32]
	ldp w30, w7, [sp, #24]
	ldr x1, [sp, #128]
	ldp w26, w3, [sp, #136]
	ldr x27, [sp, #224]
LBB66_202:
	ldr w0, [sp, #184]
LBB66_203:
	cmp w28, #4
	b.hi LBB66_287
	ldr x8, [sp, #80]
	ldr d1, [x8, x28, lsl #3]
	fmov d2, #1.00000000
	fsub d0, d2, d0
	fmul d0, d0, d1
	fcmp d9, d0
	b.pl LBB66_208
	ldr d0, [x15, #184]
	fcmp d0, #0.0
	ldr w28, [sp, #192]
	b.le LBB66_268
	fmov d1, #0.50000000
	fadd d1, d8, d1
	fmul d0, d1, d0
	frinta d0, d0
	fmov d1, #1.00000000
	fmaxnm d0, d0, d1
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	mov w9, #4
	b LBB66_210
LBB66_208:
	mov w8, #0
	mov w9, #5
LBB66_209:
	ldr w28, [sp, #192]
LBB66_210:
	strb w9, [x21, x23]
	cmp x23, x13
	b.hs LBB66_310
	strh w8, [x14, x23, lsl #1]
LBB66_212:
	ldp x10, x23, [sp, #88]
	umull x8, w23, w19
	lsr x9, x8, #32
	eor w9, w4, w9
	umull x10, w10, w25
	eor x11, x10, x22
	lsr x11, x11, #32
	umull x9, w9, w25
	umull x11, w11, w19
	lsr x12, x11, #32
	eor w8, w5, w8
	eor w8, w8, w12
	lsr x12, x9, #32
	eor w10, w6, w10
	eor w10, w10, w12
	umull x8, w8, w25
	umull x10, w10, w19
	lsr x12, x10, #32
	eor w11, w7, w11
	eor w11, w11, w12
	lsr x12, x8, #32
	eor w9, w30, w9
	eor w9, w9, w12
	umull x11, w11, w25
	umull x9, w9, w19
	lsr x12, x9, #32
	eor w10, w3, w10
	eor w10, w10, w12
	lsr x12, x11, #32
	eor w8, w26, w8
	eor w8, w8, w12
	umull x10, w10, w25
	umull x8, w8, w19
	lsr x12, x8, #32
	eor w9, w0, w9
	eor w9, w9, w12
	lsr x12, x10, #32
	eor w11, w28, w11
	eor w11, w11, w12
	umull x9, w9, w25
	umull x11, w11, w19
	lsr x12, x11, #32
	ldur w0, [x29, #-192]
	eor w8, w0, w8
	eor w8, w8, w12
	lsr x12, x9, #32
	ldur w0, [x29, #-208]
	eor w10, w0, w10
	eor w10, w10, w12
	umull x8, w8, w25
	umull x10, w10, w19
	lsr x12, x10, #32
	ldur w0, [x29, #-216]
	eor w11, w0, w11
	eor w11, w11, w12
	lsr x12, x8, #32
	ldur w0, [x29, #-224]
	eor w9, w0, w9
	eor w9, w9, w12
	umull x11, w11, w25
	umull x9, w9, w19
	lsr x9, x9, #32
	ldur w12, [x29, #-232]
	eor w10, w12, w10
	eor w9, w10, w9
	lsr x10, x11, #32
	ldur w12, [x29, #-240]
	eor w8, w12, w8
	eor w8, w8, w10
	umull x9, w9, w25
	mul w8, w8, w19
	lsr x9, x9, #32
	ldr w10, [sp, #248]
	eor w10, w10, w11
	eor w9, w10, w9
	umull x9, w9, w19
	lsr x10, x9, #32
	ldr w11, [sp, #240]
	eor w8, w8, w11
	eor w8, w8, w10
	ucvtf d0, w8
	fmov d1, #0.50000000
	fadd d0, d0, d1
	mov x8, #4463067230724161536
	fmov d2, x8
	fmul d9, d0, d2
	ucvtf d0, w9
	fadd d0, d0, d1
	fmul d8, d0, d2
	cmp w23, #1
	b.ne LBB66_221
	ldr x8, [sp, #152]
	cmp x20, x8
	b.hs LBB66_311
	ldr x8, [sp, #216]
	ldr w0, [x8, x20, lsl #2]
	ldr x8, [sp, #144]
	cmp x8, x0
	b.ls LBB66_292
	cmp x20, x27
	b.hs LBB66_316
	cmp x20, x1
	b.hs LBB66_318
	ldr x8, [sp, #200]
	ldr d10, [x8, x0, lsl #3]
	ldr x8, [sp, #232]
	ldrb w8, [x8, x20]
	cbz w8, LBB66_231
	ldr x8, [sp, #168]
	ldrh w0, [x8, x20, lsl #1]
	cmp w0, #1024
	b.hs LBB66_239
	ldr x1, [x15, #16]
	cmp x1, x0
	b.ls LBB66_289
	ldr x8, [x15, #8]
	ldr d0, [x8, x0, lsl #3]
	fmov d1, #1.00000000
	fsub d0, d1, d0
	fmul d0, d10, d0
	fcmp d9, d0
	b.mi LBB66_242
	b LBB66_246
LBB66_221:
	cmp x20, x24
	b.hs LBB66_313
	cmp x20, x16
	b.hs LBB66_315
	ldr x8, [sp, #208]
	ldrb w28, [x8, x20]
	ldrb w8, [x21, x20]
	cmp w8, #1
	b.eq LBB66_232
	cmp w8, #3
	b.ne LBB66_237
	cmp x20, x27
	b.hs LBB66_335
	cmp x20, x1
	b.hs LBB66_336
	ldr x8, [sp, #232]
	ldrb w8, [x8, x20]
	cbz w8, LBB66_250
	ldr x8, [sp, #168]
	ldrh w0, [x8, x20, lsl #1]
	cmp w0, #1024
	b.hs LBB66_251
	ldr x1, [x15, #40]
	cmp x1, x0
	b.ls LBB66_293
	ldr x8, [x15, #32]
	ldr d0, [x8, x0, lsl #3]
	b LBB66_254
LBB66_231:
	movi.2d v0, #0000000000000000
	fmov d1, #1.00000000
	fsub d0, d1, d0
	fmul d0, d10, d0
	fcmp d9, d0
	b.mi LBB66_242
	b LBB66_246
LBB66_232:
	cmp w28, #4
	b.hi LBB66_288
	ldr x8, [sp, #64]
	ldr d0, [x8, x28, lsl #3]
	fcmp d9, d0
	b.pl LBB66_247
	ldr d0, [x15, #168]
	fcmp d0, #0.0
	b.le LBB66_269
	fadd d1, d8, d1
	fmul d0, d1, d0
	frinta d0, d0
	fmov d1, #1.00000000
	fmaxnm d0, d0, d1
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	mov w9, #2
	b LBB66_273
LBB66_237:
	cmp w28, #4
	b.hi LBB66_286
	mov w8, #0
	ldr x9, [sp, #72]
	ldr d0, [x9, x28, lsl #3]
	fcmp d9, d0
	mov w9, #5
	mov w10, #6
	csel w9, w10, w9, mi
	b LBB66_273
LBB66_239:
	mov x25, x2
	mov x24, x16
	mov x23, x15
	mov x22, x21
	mov x21, x14
	mov x19, x13
	ldr x8, [sp, #176]
	ldr d11, [x8]
	ucvtf d0, w0
	ldr d1, [x8, #16]
	fmov d12, #1.00000000
	fcmp d1, #0.0
	b.le LBB66_241
	fdiv d2, d0, d1
	fmov d3, #1.00000000
	fminnm d12, d2, d3
LBB66_241:
	fsub d0, d0, d1
	movi.2d v1, #0000000000000000
	fmaxnm d0, d0, d1
	ldr x8, [sp, #176]
	ldr d1, [x8, #24]
	fneg d0, d0
	fdiv d0, d0, d1
	bl _exp2
	fmul d0, d12, d0
	fmul d0, d11, d0
	mov x13, x19
	mov x14, x21
	mov x21, x22
	mov x15, x23
	mov x16, x24
	mov x2, x25
	fmov d1, #1.00000000
	fsub d0, d1, d0
	fmul d0, d10, d0
	fcmp d9, d0
	b.pl LBB66_246
LBB66_242:
	cmp x20, x16
	b.hs LBB66_327
	mov w8, #1
	strb w8, [x21, x20]
	cmp x20, x13
	b.hs LBB66_328
	ldr d1, [x15, #192]
	fmov d2, #0.50000000
	fadd d2, d8, d2
	fmul d1, d2, d1
	frinta d1, d1
	fmov d2, #1.00000000
	fmaxnm d1, d1, d2
	fcvtzu w8, d1
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	strh w8, [x14, x20, lsl #1]
LBB66_246:
	fcmp d9, d0
	cinc w8, w2, mi
	ldur x9, [x29, #-200]
	str w8, [x9]
	b LBB66_275
LBB66_247:
	ldr d0, [x15, #176]
	fcmp d0, #0.0
	b.le LBB66_270
	fadd d1, d8, d1
	fmul d0, d1, d0
	frinta d0, d0
	fmov d1, #1.00000000
	fmaxnm d0, d0, d1
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	mov w9, #3
	b LBB66_273
LBB66_250:
	movi.2d v0, #0000000000000000
	b LBB66_254
LBB66_251:
	mov x23, x15
	mov x22, x21
	mov x21, x14
	mov x19, x13
	ucvtf d0, w0
	ldr x8, [sp, #176]
	ldp d10, d1, [x8, #8]
	fmov d11, #1.00000000
	fcmp d1, #0.0
	b.le LBB66_253
	fdiv d2, d0, d1
	fmov d3, #1.00000000
	fminnm d11, d2, d3
LBB66_253:
	fsub d0, d0, d1
	movi.2d v1, #0000000000000000
	fmaxnm d0, d0, d1
	ldr x8, [sp, #176]
	ldr d1, [x8, #24]
	fneg d0, d0
	fdiv d0, d0, d1
	bl _exp2
	fmul d0, d11, d0
	fmul d0, d10, d0
	mov x13, x19
	mov x14, x21
	mov x21, x22
	mov x15, x23
LBB66_254:
	cmp w28, #4
	b.hi LBB66_287
	ldr x8, [sp, #80]
	ldr d1, [x8, x28, lsl #3]
	fmov d2, #1.00000000
	fsub d0, d2, d0
	fmul d0, d0, d1
	fcmp d9, d0
	b.pl LBB66_259
	ldr d0, [x15, #184]
	fcmp d0, #0.0
	b.le LBB66_271
	fmov d1, #0.50000000
	fadd d1, d8, d1
	fmul d0, d1, d0
	frinta d0, d0
	fmov d1, #1.00000000
	fmaxnm d0, d0, d1
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	b LBB66_272
LBB66_259:
	mov w8, #0
	mov w9, #5
	b LBB66_273
LBB66_260:
	mov w8, #0
	mov w9, #2
	strb w9, [x21, x26]
	cmp x26, x13
	b.lo LBB66_84
	b LBB66_146
LBB66_261:
	mov w8, #0
	mov w9, #3
	strb w9, [x21, x26]
	cmp x26, x13
	b.lo LBB66_84
	b LBB66_146
LBB66_262:
	mov w8, #0
	mov w9, #4
	strb w9, [x21, x26]
	cmp x26, x13
	b.lo LBB66_84
	b LBB66_146
LBB66_263:
	mov w8, #0
	mov w9, #2
	b LBB66_158
LBB66_264:
	mov w8, #0
	mov w9, #3
	b LBB66_158
LBB66_265:
	mov w8, #0
	mov w9, #4
	b LBB66_158
LBB66_266:
	mov w8, #0
	mov w9, #2
	b LBB66_210
LBB66_267:
	mov w8, #0
	mov w9, #3
	b LBB66_210
LBB66_268:
	mov w8, #0
	mov w9, #4
	b LBB66_210
LBB66_269:
	mov w8, #0
	mov w9, #2
	b LBB66_273
LBB66_270:
	mov w8, #0
	mov w9, #3
	b LBB66_273
LBB66_271:
	mov w8, #0
LBB66_272:
	mov w9, #4
LBB66_273:
	strb w9, [x21, x20]
	cmp x20, x13
	b.hs LBB66_320
	strh w8, [x14, x20, lsl #1]
LBB66_275:
	add sp, sp, #352
	.cfi_def_cfa wsp, 160
	ldp x29, x30, [sp, #144]
	ldp x20, x19, [sp, #128]
	ldp x22, x21, [sp, #112]
	ldp x24, x23, [sp, #96]
	ldp x26, x25, [sp, #80]
	ldp x28, x27, [sp, #64]
	ldp d9, d8, [sp, #48]
	ldp d11, d10, [sp, #32]
	ldp d13, d12, [sp, #16]
	ldp d15, d14, [sp], #160
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore w23
	.cfi_restore w24
	.cfi_restore w25
	.cfi_restore w26
	.cfi_restore w27
	.cfi_restore w28
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	.cfi_restore b12
	.cfi_restore b13
	.cfi_restore b14
	.cfi_restore b15
	ret
LBB66_276:
	.cfi_restore_state
Lloh161:
	adrp x3, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.77@PAGE
Lloh162:
	add x3, x3, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.77@PAGEOFF
	mov x0, #0
	mov w2, #4
	bl core::slice::index::slice_index_fail
LBB66_277:
Lloh163:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.76@PAGE
Lloh164:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.76@PAGEOFF
	mov x0, x19
	mov x1, x20
	bl core::panicking::panic_bounds_check
LBB66_278:
Lloh165:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.65@PAGE
Lloh166:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.65@PAGEOFF
	mov x0, x19
	ldur x1, [x29, #-208]
	bl core::panicking::panic_bounds_check
LBB66_279:
Lloh167:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.83@PAGE
Lloh168:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.83@PAGEOFF
	mov x0, x19
	ldur x1, [x29, #-232]
	bl core::panicking::panic_bounds_check
LBB66_280:
Lloh169:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.81@PAGE
Lloh170:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.81@PAGEOFF
	mov x0, x19
	ldr x1, [sp, #232]
	bl core::panicking::panic_bounds_check
LBB66_281:
Lloh171:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.64@PAGE
Lloh172:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.64@PAGEOFF
	mov x0, x19
	ldr x1, [sp, #240]
	bl core::panicking::panic_bounds_check
LBB66_282:
Lloh173:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.84@PAGE
Lloh174:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.84@PAGEOFF
	mov x0, x19
	ldur x1, [x29, #-240]
	bl core::panicking::panic_bounds_check
LBB66_283:
Lloh175:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.82@PAGE
Lloh176:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.82@PAGEOFF
	ldr x1, [sp, #224]
	bl core::panicking::panic_bounds_check
LBB66_284:
Lloh177:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.86@PAGE
Lloh178:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.86@PAGEOFF
	mov x0, x19
	ldur x1, [x29, #-216]
	bl core::panicking::panic_bounds_check
LBB66_285:
Lloh179:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.85@PAGE
Lloh180:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.85@PAGEOFF
	mov x0, x19
	ldur x1, [x29, #-208]
	bl core::panicking::panic_bounds_check
LBB66_286:
Lloh181:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.70@PAGE
Lloh182:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.70@PAGEOFF
	mov x0, x28
	mov w1, #5
	bl core::panicking::panic_bounds_check
LBB66_287:
Lloh183:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.69@PAGE
Lloh184:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.69@PAGEOFF
	mov x0, x28
	mov w1, #5
	bl core::panicking::panic_bounds_check
LBB66_288:
Lloh185:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.66@PAGE
Lloh186:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.66@PAGEOFF
	mov x0, x28
	mov w1, #5
	bl core::panicking::panic_bounds_check
LBB66_289:
Lloh187:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.56@PAGE
Lloh188:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.56@PAGEOFF
	bl core::panicking::panic_bounds_check
LBB66_290:
Lloh189:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.67@PAGE
Lloh190:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.67@PAGEOFF
	mov x0, x19
	ldur x1, [x29, #-232]
	bl core::panicking::panic_bounds_check
LBB66_291:
Lloh191:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.68@PAGE
Lloh192:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.68@PAGEOFF
	mov x0, x19
	ldur x1, [x29, #-240]
	bl core::panicking::panic_bounds_check
LBB66_292:
Lloh193:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.82@PAGE
Lloh194:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.82@PAGEOFF
	ldr x1, [sp, #144]
	bl core::panicking::panic_bounds_check
LBB66_293:
Lloh195:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.57@PAGE
Lloh196:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.57@PAGEOFF
	bl core::panicking::panic_bounds_check
LBB66_294:
Lloh197:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.72@PAGE
Lloh198:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.72@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_295:
Lloh199:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.73@PAGE
Lloh200:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.73@PAGEOFF
	mov x0, x27
	bl core::panicking::panic_bounds_check
LBB66_296:
Lloh201:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.74@PAGE
Lloh202:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.74@PAGEOFF
	mov x0, x23
	bl core::panicking::panic_bounds_check
LBB66_297:
Lloh203:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.75@PAGE
Lloh204:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.75@PAGEOFF
	mov x0, x20
	bl core::panicking::panic_bounds_check
LBB66_298:
	mov x1, x16
Lloh205:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.65@PAGE
Lloh206:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.65@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_299:
	mov x26, x27
Lloh207:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.81@PAGE
Lloh208:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.81@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #152]
	bl core::panicking::panic_bounds_check
LBB66_300:
	mov x26, x27
Lloh209:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.64@PAGE
Lloh210:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.64@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #160]
	bl core::panicking::panic_bounds_check
LBB66_301:
	mov x1, x16
	mov x26, x27
Lloh211:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.65@PAGE
Lloh212:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.65@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_302:
	mov x26, x27
Lloh213:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.83@PAGE
Lloh214:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.83@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #224]
	bl core::panicking::panic_bounds_check
LBB66_303:
	mov x26, x27
Lloh215:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.84@PAGE
Lloh216:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.84@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #128]
	bl core::panicking::panic_bounds_check
LBB66_304:
	mov x1, x13
	mov x26, x27
Lloh217:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.71@PAGE
Lloh218:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.71@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_305:
	mov x26, x23
Lloh219:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.81@PAGE
Lloh220:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.81@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #152]
	bl core::panicking::panic_bounds_check
LBB66_306:
	mov x26, x23
Lloh221:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.64@PAGE
Lloh222:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.64@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #160]
	bl core::panicking::panic_bounds_check
LBB66_307:
	mov x1, x16
	mov x26, x23
Lloh223:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.65@PAGE
Lloh224:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.65@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_308:
	mov x26, x23
Lloh225:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.83@PAGE
Lloh226:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.83@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #224]
	bl core::panicking::panic_bounds_check
LBB66_309:
	mov x26, x23
Lloh227:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.84@PAGE
Lloh228:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.84@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #128]
	bl core::panicking::panic_bounds_check
LBB66_310:
	mov x1, x13
	mov x26, x23
Lloh229:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.71@PAGE
Lloh230:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.71@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_311:
	mov x26, x20
LBB66_312:
Lloh231:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.81@PAGE
Lloh232:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.81@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #152]
	bl core::panicking::panic_bounds_check
LBB66_313:
	mov x26, x20
LBB66_314:
Lloh233:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.64@PAGE
Lloh234:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.64@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #160]
	bl core::panicking::panic_bounds_check
LBB66_315:
	mov x1, x16
	mov x26, x20
Lloh235:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.65@PAGE
Lloh236:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.65@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_316:
	mov x26, x20
LBB66_317:
Lloh237:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.83@PAGE
Lloh238:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.83@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #224]
	bl core::panicking::panic_bounds_check
LBB66_318:
	mov x26, x20
LBB66_319:
Lloh239:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.84@PAGE
Lloh240:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.84@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #128]
	bl core::panicking::panic_bounds_check
LBB66_320:
	mov x1, x13
	mov x26, x20
Lloh241:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.71@PAGE
Lloh242:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.71@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_321:
	mov x1, x16
Lloh243:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.85@PAGE
Lloh244:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.85@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_322:
	mov x1, x13
Lloh245:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.86@PAGE
Lloh246:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.86@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_323:
	mov x1, x16
	mov x26, x27
Lloh247:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.85@PAGE
Lloh248:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.85@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_324:
	mov x1, x13
	mov x26, x27
Lloh249:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.86@PAGE
Lloh250:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.86@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_325:
	mov x1, x16
	mov x26, x23
Lloh251:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.85@PAGE
Lloh252:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.85@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_326:
	mov x1, x13
	mov x26, x23
Lloh253:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.86@PAGE
Lloh254:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.86@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_327:
	mov x1, x16
	mov x26, x20
Lloh255:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.85@PAGE
Lloh256:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.85@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_328:
	mov x1, x13
	mov x26, x20
Lloh257:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.86@PAGE
Lloh258:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.86@PAGEOFF
	mov x0, x26
	bl core::panicking::panic_bounds_check
LBB66_329:
	mov x26, x27
LBB66_330:
Lloh259:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.67@PAGE
Lloh260:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.67@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #224]
	bl core::panicking::panic_bounds_check
LBB66_331:
	mov x26, x27
LBB66_332:
Lloh261:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.68@PAGE
Lloh262:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.68@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #128]
	bl core::panicking::panic_bounds_check
LBB66_333:
	mov x26, x23
Lloh263:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.67@PAGE
Lloh264:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.67@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #224]
	bl core::panicking::panic_bounds_check
LBB66_334:
	mov x26, x23
Lloh265:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.68@PAGE
Lloh266:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.68@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #128]
	bl core::panicking::panic_bounds_check
LBB66_335:
	mov x26, x20
Lloh267:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.67@PAGE
Lloh268:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.67@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #224]
	bl core::panicking::panic_bounds_check
LBB66_336:
	mov x26, x20
Lloh269:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.68@PAGE
Lloh270:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.68@PAGEOFF
	mov x0, x26
	ldr x1, [sp, #128]
	bl core::panicking::panic_bounds_check
	.loh AdrpAdd	Lloh157, Lloh158
	.loh AdrpAdd	Lloh159, Lloh160
	.loh AdrpAdd	Lloh161, Lloh162
	.loh AdrpAdd	Lloh163, Lloh164
	.loh AdrpAdd	Lloh165, Lloh166
	.loh AdrpAdd	Lloh167, Lloh168
	.loh AdrpAdd	Lloh169, Lloh170
	.loh AdrpAdd	Lloh171, Lloh172
	.loh AdrpAdd	Lloh173, Lloh174
	.loh AdrpAdd	Lloh175, Lloh176
	.loh AdrpAdd	Lloh177, Lloh178
	.loh AdrpAdd	Lloh179, Lloh180
	.loh AdrpAdd	Lloh181, Lloh182
	.loh AdrpAdd	Lloh183, Lloh184
	.loh AdrpAdd	Lloh185, Lloh186
	.loh AdrpAdd	Lloh187, Lloh188
	.loh AdrpAdd	Lloh189, Lloh190
	.loh AdrpAdd	Lloh191, Lloh192
	.loh AdrpAdd	Lloh193, Lloh194
	.loh AdrpAdd	Lloh195, Lloh196
	.loh AdrpAdd	Lloh197, Lloh198
	.loh AdrpAdd	Lloh199, Lloh200
	.loh AdrpAdd	Lloh201, Lloh202
	.loh AdrpAdd	Lloh203, Lloh204
	.loh AdrpAdd	Lloh205, Lloh206
	.loh AdrpAdd	Lloh207, Lloh208
	.loh AdrpAdd	Lloh209, Lloh210
	.loh AdrpAdd	Lloh211, Lloh212
	.loh AdrpAdd	Lloh213, Lloh214
	.loh AdrpAdd	Lloh215, Lloh216
	.loh AdrpAdd	Lloh217, Lloh218
	.loh AdrpAdd	Lloh219, Lloh220
	.loh AdrpAdd	Lloh221, Lloh222
	.loh AdrpAdd	Lloh223, Lloh224
	.loh AdrpAdd	Lloh225, Lloh226
	.loh AdrpAdd	Lloh227, Lloh228
	.loh AdrpAdd	Lloh229, Lloh230
	.loh AdrpAdd	Lloh231, Lloh232
	.loh AdrpAdd	Lloh233, Lloh234
	.loh AdrpAdd	Lloh235, Lloh236
	.loh AdrpAdd	Lloh237, Lloh238
	.loh AdrpAdd	Lloh239, Lloh240
	.loh AdrpAdd	Lloh241, Lloh242
	.loh AdrpAdd	Lloh243, Lloh244
	.loh AdrpAdd	Lloh245, Lloh246
	.loh AdrpAdd	Lloh247, Lloh248
	.loh AdrpAdd	Lloh249, Lloh250
	.loh AdrpAdd	Lloh251, Lloh252
	.loh AdrpAdd	Lloh253, Lloh254
	.loh AdrpAdd	Lloh255, Lloh256
	.loh AdrpAdd	Lloh257, Lloh258
	.loh AdrpAdd	Lloh259, Lloh260
	.loh AdrpAdd	Lloh261, Lloh262
	.loh AdrpAdd	Lloh263, Lloh264
	.loh AdrpAdd	Lloh265, Lloh266
	.loh AdrpAdd	Lloh267, Lloh268
	.loh AdrpAdd	Lloh269, Lloh270
