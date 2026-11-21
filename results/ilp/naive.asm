	.globl	epidemicsim_backends::kernels::place_naive
	.p2align	2
epidemicsim_backends::kernels::place_naive:
Lfunc_begin67:
	.cfi_startproc
	sub sp, sp, #176
	.cfi_def_cfa_offset 176
	stp x28, x27, [sp, #80]
	stp x26, x25, [sp, #96]
	stp x24, x23, [sp, #112]
	stp x22, x21, [sp, #128]
	stp x20, x19, [sp, #144]
	stp x29, x30, [sp, #160]
	add x29, sp, #160
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
	str x1, [sp, #72]
	subs x27, x2, x3
	b.hs LBB67_30
	mov x19, x5
	mov x20, x4
	mov x21, x2
	mov x23, x0
	ldr w8, [x0, #488]
	str w8, [sp, #60]
	ldp x8, x9, [x0, #56]
	ldp x24, x10, [x6, #8]
	str x10, [sp, #64]
	ldp x11, x10, [x6, #56]
	str x10, [sp, #40]
	ldr x10, [x6, #32]
	stp x10, x11, [sp, #16]
	ldr x10, [x6, #40]
	str x10, [sp, #32]
	cmp x9, x2
	str x9, [sp, #8]
	csel x10, x9, x2, hi
	str x10, [sp, #48]
	mov x26, #0
	add x25, x8, x2
	sub x22, x2, x10
	tbnz w7, #0, LBB67_6
	b LBB67_20
LBB67_2:
	ldr x8, [sp, #32]
	cmp x8, x0
	b.ls LBB67_39
	ldr x8, [sp, #16]
LBB67_4:
	add x8, x8, x0, lsl #2
	ldadd w9, w8, [x8]
LBB67_5:
	add x26, x26, #1
	cmn x27, x26
	b.eq LBB67_30
LBB67_6:
	cmn x22, x26
	b.eq LBB67_31
	ldrb w28, [x25, x26]
	cmp w28, #6
	b.ne LBB67_10
	cmp x26, x19
	b.hs LBB67_32
	mov w8, #-1
	str w8, [x20, x26, lsl #2]
	b LBB67_5
LBB67_10:
	add x1, x21, x26
	mov x0, x23
	ldr w2, [sp, #60]
	ldr x3, [sp, #72]
	bl epidemicsim_core::schedule::location_of
	cmp x26, x19
	b.hs LBB67_33
	str w0, [x20, x26, lsl #2]
	mov w0, w0
	ldr x8, [sp, #64]
	cmp x8, x0
	b.ls LBB67_34
	add x8, x24, x0, lsl #2
	mov w9, #1
	ldadd w9, w8, [x8]
	cmp w28, #2
	ldr x10, [sp, #48]
	b.eq LBB67_2
	cmp w28, #3
	b.ne LBB67_5
	ldr x8, [sp, #40]
	cmp x8, x0
	b.ls LBB67_38
	ldr x8, [sp, #24]
	b LBB67_4
LBB67_16:
	ldr x8, [sp, #32]
	cmp x8, x0
	b.ls LBB67_37
	ldr x9, [sp, #16]
LBB67_18:
	ldr w8, [x9, x0, lsl #2]
	add w8, w8, #1
	str w8, [x9, x0, lsl #2]
LBB67_19:
	add x26, x26, #1
	cmn x27, x26
	b.eq LBB67_30
LBB67_20:
	cmn x22, x26
	b.eq LBB67_31
	ldrb w28, [x25, x26]
	cmp w28, #6
	b.ne LBB67_24
	cmp x26, x19
	b.hs LBB67_32
	mov w8, #-1
	str w8, [x20, x26, lsl #2]
	b LBB67_19
LBB67_24:
	add x1, x21, x26
	mov x0, x23
	ldr w2, [sp, #60]
	ldr x3, [sp, #72]
	bl epidemicsim_core::schedule::location_of
	cmp x26, x19
	b.hs LBB67_33
	str w0, [x20, x26, lsl #2]
	mov w0, w0
	ldr x8, [sp, #64]
	cmp x8, x0
	b.ls LBB67_35
	ldr w8, [x24, x0, lsl #2]
	add w8, w8, #1
	str w8, [x24, x0, lsl #2]
	cmp w28, #2
	ldr x10, [sp, #48]
	b.eq LBB67_16
	cmp w28, #3
	b.ne LBB67_19
	ldr x8, [sp, #40]
	cmp x8, x0
	b.ls LBB67_36
	ldr x9, [sp, #24]
	b LBB67_18
LBB67_30:
	.cfi_def_cfa wsp, 176
	ldp x29, x30, [sp, #160]
	ldp x20, x19, [sp, #144]
	ldp x22, x21, [sp, #128]
	ldp x24, x23, [sp, #112]
	ldp x26, x25, [sp, #96]
	ldp x28, x27, [sp, #80]
	add sp, sp, #176
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
LBB67_31:
	.cfi_restore_state
Lloh271:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.78@PAGE
Lloh272:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.78@PAGEOFF
	mov x0, x10
	ldr x1, [sp, #8]
	bl core::panicking::panic_bounds_check
LBB67_32:
Lloh273:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.79@PAGE
Lloh274:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.79@PAGEOFF
	mov x0, x26
	mov x1, x19
	bl core::panicking::panic_bounds_check
LBB67_33:
Lloh275:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.80@PAGE
Lloh276:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.80@PAGEOFF
	mov x0, x26
	mov x1, x19
	bl core::panicking::panic_bounds_check
LBB67_34:
Lloh277:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGE
Lloh278:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGEOFF
	ldr x1, [sp, #64]
	bl core::panicking::panic_bounds_check
LBB67_35:
Lloh279:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGE
Lloh280:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGEOFF
	ldr x1, [sp, #64]
	bl core::panicking::panic_bounds_check
LBB67_36:
Lloh281:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGE
Lloh282:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGEOFF
	ldr x1, [sp, #40]
	bl core::panicking::panic_bounds_check
LBB67_37:
Lloh283:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGE
Lloh284:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.121@PAGEOFF
	ldr x1, [sp, #32]
	bl core::panicking::panic_bounds_check
LBB67_38:
Lloh285:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGE
Lloh286:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGEOFF
	ldr x1, [sp, #40]
	bl core::panicking::panic_bounds_check
LBB67_39:
Lloh287:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGE
Lloh288:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.122@PAGEOFF
	ldr x1, [sp, #32]
	bl core::panicking::panic_bounds_check
	.loh AdrpAdd	Lloh271, Lloh272
	.loh AdrpAdd	Lloh273, Lloh274
	.loh AdrpAdd	Lloh275, Lloh276
	.loh AdrpAdd	Lloh277, Lloh278
	.loh AdrpAdd	Lloh279, Lloh280
	.loh AdrpAdd	Lloh281, Lloh282
	.loh AdrpAdd	Lloh283, Lloh284
	.loh AdrpAdd	Lloh285, Lloh286
	.loh AdrpAdd	Lloh287, Lloh288
	.globl	epidemicsim_backends::kernels::update_naive
	.p2align	6
epidemicsim_backends::kernels::update_naive:
Lfunc_begin69:
	.cfi_startproc
	sub sp, sp, #368
	.cfi_def_cfa_offset 368
	stp d15, d14, [sp, #208]
	stp d13, d12, [sp, #224]
	stp d11, d10, [sp, #240]
	stp d9, d8, [sp, #256]
	stp x28, x27, [sp, #272]
	stp x26, x25, [sp, #288]
	stp x24, x23, [sp, #304]
	stp x22, x21, [sp, #320]
	stp x20, x19, [sp, #336]
	stp x29, x30, [sp, #352]
	add x29, sp, #352
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
	ldr x28, [x1, #72]
	cbz x28, LBB69_31
	mov x20, x1
	mov x21, x0
	str x8, [sp, #24]
	mov x22, #0
	ldr w8, [x0, #56]
	str w8, [sp, #132]
	ldr x19, [x1, #64]
	ldp x24, x23, [x1, #80]
	ldp x10, x9, [x1]
	str x9, [sp, #120]
	ldr x8, [x0, #48]
	stp x8, x10, [sp, #96]
	ldp x8, x11, [x0, #16]
	add x10, x8, #112
	add x9, x8, #40
	str x8, [sp, #80]
	add x8, x8, #152
	stp x8, x9, [sp, #48]
	fmov d8, #0.50000000
	ldp x8, x9, [x1, #16]
	str x9, [sp, #112]
	str x8, [sp, #88]
	mov x8, #4463067230724161536
	fmov d9, x8
	ldp x8, x9, [x1, #32]
	stp x10, x8, [sp, #32]
	fmov d10, #4.00000000
	fmov d11, #1.00000000
	mov w25, #97
	ldp x8, x10, [x1, #96]
	stp x10, x9, [sp, #64]
	stp x8, x11, [sp, #8]
	b LBB69_5
LBB69_2:
	sub w8, w8, #1
LBB69_3:
	strh w8, [x24, x22, lsl #1]
LBB69_4:
	add x22, x22, #1
	cmp x28, x22
	b.eq LBB69_30
LBB69_5:
	ldrb w26, [x19, x22]
	cmp w26, #6
	b.hi LBB69_7
	lsr w8, w25, w26
	tbnz w8, #0, LBB69_4
LBB69_7:
	cmp x22, x23
	b.hs LBB69_64
	ldrh w8, [x24, x22, lsl #1]
	cmp w8, #2
	b.hs LBB69_2
	ldr x8, [sp, #120]
	cmp x22, x8
	b.hs LBB69_65
	ldp x2, x8, [sp, #96]
	ldr w9, [x8, x22, lsl #2]
	ldr w8, [sp, #132]
	stp w9, w8, [x29, #-168]
	mov w8, #2
	stur x8, [x29, #-160]
	add x0, sp, #168
	sub x1, x29, #168
	bl epidemicsim_core::rng::philox4x32_10
	ldr x8, [sp, #112]
	cmp x22, x8
	b.hs LBB69_66
	ldp s0, s13, [sp, #168]
	ucvtf d0, d0
	fadd d0, d0, d8
	fmul d14, d0, d9
	ldr x8, [sp, #88]
	ldrb w27, [x8, x22]
	cmp w26, #1
	b.eq LBB69_18
	cmp w26, #2
	b.eq LBB69_28
	cmp w26, #3
	b.ne LBB69_22
	ldr x8, [sp, #72]
	cmp x22, x8
	b.hs LBB69_68
	ldr x8, [sp, #64]
	cmp x22, x8
	b.hs LBB69_69
	ldr x8, [sp, #40]
	ldrb w8, [x8, x22]
	cbz w8, LBB69_24
	ldp x9, x8, [sp, #8]
	ldr h0, [x9, x22, lsl #1]
	ucvtf d0, d0
	ldp d15, d1, [x8, #8]
	fcmp d1, #0.0
	fdiv d2, d0, d1
	fminnm d2, d2, d11
	fcsel d12, d2, d11, gt
	fsub d0, d0, d1
	movi.2d v1, #0000000000000000
	fmaxnm d0, d0, d1
	ldr d1, [x8, #24]
	fneg d0, d0
	fdiv d0, d0, d1
	bl _exp2
	fmul d0, d12, d0
	fmul d0, d15, d0
	cmp w27, #5
	b.lo LBB69_25
	b LBB69_70
LBB69_18:
	cmp w27, #5
	b.hs LBB69_71
	ldr x8, [sp, #56]
	ldr d0, [x8, x27, lsl #3]
	fcmp d14, d0
	mov w8, #96
	mov w9, #88
	csel x8, x9, x8, mi
	ldr x9, [sp, #80]
	add x9, x9, x8
	mov w8, #2
	mov w10, #3
	csel w8, w8, w10, mi
	ldr d0, [x9]
	strb w8, [x19, x22]
	fcmp d0, #0.0
	b.le LBB69_27
LBB69_20:
	ucvtf d1, d13
	fadd d1, d1, d8
	fmul d1, d1, d9
	fmul d0, d0, d10
	fadd d1, d1, d8
	fmul d0, d1, d0
	frinta d0, d0
	fmaxnm d0, d0, d11
	fcvtzu w8, d0
	mov w9, #65535
	.p2align	6, , 4
	cmp w8, w9
	csel w8, w8, w9, lo
	b LBB69_3
LBB69_22:
	cmp w27, #5
	b.hs LBB69_67
	ldr x8, [sp, #48]
	ldr d0, [x8, x27, lsl #3]
	fcmp d14, d0
	mov w8, #5
	mov w9, #6
	csel w9, w9, w8, mi
	b LBB69_29
LBB69_24:
	movi.2d v0, #0000000000000000
	cmp w27, #5
	b.hs LBB69_70
LBB69_25:
	ldr x8, [sp, #32]
	ldr d1, [x8, x27, lsl #3]
	fsub d0, d11, d0
	fmul d0, d0, d1
	fcmp d14, d0
	b.pl LBB69_28
	mov w8, #4
	ldr x9, [sp, #80]
	ldr d0, [x9, #104]!
	strb w8, [x19, x22]
	fcmp d0, #0.0
	b.gt LBB69_20
LBB69_27:
	mov w8, #0
	b LBB69_3
LBB69_28:
	mov w9, #5
LBB69_29:
	mov w8, #0
	strb w9, [x19, x22]
	b LBB69_3
LBB69_30:
	mov x22, #0
	mov w14, #0
	ldp x23, x19, [x20, #56]
	ldp x1, x25, [x20, #40]
	ldp x26, x24, [x21]
	ldr x10, [x20, #32]
	ldp x12, x9, [x20, #96]
	ldp x13, x11, [x20]
	ldr x2, [x21, #48]
	mov w15, #1
	ldp x17, x16, [x21, #16]
	fmov d8, #0.50000000
	mov x8, #4463067230724161536
	fmov d9, x8
	fmov d10, #1.00000000
	fmov d11, #4.00000000
	movi.2d v12, #0000000000000000
	ldp x4, x3, [x20, #80]
	b LBB69_34
LBB69_31:
	mov w14, #0
	stp xzr, xzr, [sp, #136]
	str wzr, [sp, #160]
	str xzr, [sp, #152]
	b LBB69_58
LBB69_32:
	ldp x9, x1, [sp, #96]
	ldr x10, [sp, #88]
	mov x11, x27
	ldr w14, [sp, #80]
	ldp x16, x12, [sp, #64]
	ldp x2, x13, [sp, #112]
	mov w15, #1
	ldp x3, x17, [sp, #48]
	ldr x4, [sp, #40]
LBB69_33:
	add x22, x22, #1
	cmp x22, x28
	b.hs LBB69_47
LBB69_34:
	ldrb w8, [x19, x22]
	cbnz w8, LBB69_33
	cmp x22, x23
	b.hs LBB69_62
	ldr w0, [x25, x22, lsl #2]
	cmp x24, x0
	b.ls LBB69_61
	ldr d13, [x26, x0, lsl #3]
	fcmp d13, #0.0
	b.ls LBB69_33
	cmp x22, x1
	b.hs LBB69_72
	cmp x22, x9
	b.hs LBB69_73
	ldrb w8, [x10, x22]
	stp x9, x1, [sp, #96]
	str x10, [sp, #88]
	str w14, [sp, #80]
	stp x16, x12, [sp, #64]
	stp x2, x13, [sp, #112]
	stp x3, x17, [sp, #48]
	str x4, [sp, #40]
	cbz w8, LBB69_42
	ldr d14, [x16]
	ldr h0, [x12, x22, lsl #1]
	ucvtf d0, d0
	ldp d3, d1, [x16, #16]
	fcmp d3, #0.0
	fdiv d2, d0, d3
	fminnm d2, d2, d10
	fcsel d15, d2, d10, gt
	mov x27, x11
	fsub d0, d0, d3
	fmaxnm d0, d0, d12
	fneg d0, d0
	fdiv d0, d0, d1
	bl _exp2
	fmul d0, d15, d0
	fmul d14, d14, d0
	ldp x2, x13, [sp, #112]
	mov w15, #1
	cmp x22, x27
	b.lo LBB69_43
	b LBB69_74
LBB69_42:
	movi.2d v14, #0000000000000000
	mov x27, x11
	cmp x22, x11
	b.hs LBB69_74
LBB69_43:
	ldr w9, [x13, x22, lsl #2]
	ldr w8, [sp, #132]
	stp w9, w8, [x29, #-168]
	stur x15, [x29, #-160]
	add x0, sp, #168
	sub x1, x29, #168
	bl epidemicsim_core::rng::philox4x32_10
	ldr s0, [sp, #168]
	ucvtf d0, d0
	fadd d0, d0, d8
	fmul d0, d0, d9
	fsub d1, d10, d14
	fmul d1, d13, d1
	fcmp d0, d1
	b.pl LBB69_32
	ldr s0, [sp, #172]
	mov w8, #1
	strb w8, [x19, x22]
	ldp x3, x17, [sp, #48]
	cmp x22, x3
	ldp x9, x1, [sp, #96]
	ldr x10, [sp, #88]
	ldr w14, [sp, #80]
	ldp x16, x12, [sp, #64]
	mov w15, #1
	mov w11, #65535
	ldr x4, [sp, #40]
	b.hs LBB69_75
	ucvtf d0, d0
	fadd d0, d0, d8
	fmul d0, d0, d9
	ldr d1, [x17, #80]
	fmul d1, d1, d11
	fadd d0, d0, d8
	fmul d0, d0, d1
	frinta d0, d0
	fmaxnm d0, d0, d10
	fcvtzu w8, d0
	.p2align	6, , 4
	cmp w8, w11
	csel w8, w8, w11, lo
	strh w8, [x4, x22, lsl #1]
	add w14, w14, #1
	mov x11, x27
	ldp x2, x13, [sp, #112]
	b LBB69_33
LBB69_47:
	ldrb w8, [x21, #60]
	cbz w8, LBB69_54
	mov x8, #0
	ldp x9, x0, [x20, #32]
	mov w10, #65535
	ldp x11, x1, [x20, #96]
	b LBB69_50
LBB69_49:
	add x8, x8, #1
	cmp x28, x8
	b.eq LBB69_54
LBB69_50:
	cmp x0, x8
	b.eq LBB69_60
	ldrb w12, [x9, x8]
	cbz w12, LBB69_49
	cmp x8, x1
	b.hs LBB69_63
	ldrh w12, [x11, x8, lsl #1]
	add w13, w12, #1
	cmp w13, w10
	csinc w12, w10, w12, hs
	strh w12, [x11, x8, lsl #1]
	b LBB69_49
LBB69_54:
	stp xzr, xzr, [sp, #136]
	str wzr, [sp, #160]
	str xzr, [sp, #152]
	add x8, sp, #136
	ldr x9, [x20, #64]
LBB69_55:
	ldrb w0, [x9], #1
	cmp x0, #7
	b.hs LBB69_59
	ldr w10, [x8, x0, lsl #2]
	add w10, w10, #1
	str w10, [x8, x0, lsl #2]
	subs x28, x28, #1
	b.ne LBB69_55
	ldr x8, [sp, #24]
LBB69_58:
	ldur q0, [sp, #136]
	str q0, [x8]
	ldur q0, [sp, #148]
	stur q0, [x8, #12]
	str w14, [x8, #28]
	.cfi_def_cfa wsp, 368
	ldp x29, x30, [sp, #352]
	ldp x20, x19, [sp, #336]
	ldp x22, x21, [sp, #320]
	ldp x24, x23, [sp, #304]
	ldp x26, x25, [sp, #288]
	ldp x28, x27, [sp, #272]
	ldp d9, d8, [sp, #256]
	ldp d11, d10, [sp, #240]
	ldp d13, d12, [sp, #224]
	ldp d15, d14, [sp, #208]
	add sp, sp, #368
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
LBB69_59:
	.cfi_restore_state
Lloh297:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.96@PAGE
Lloh298:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.96@PAGEOFF
	mov w1, #7
	bl core::panicking::panic_bounds_check
LBB69_60:
Lloh299:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.97@PAGE
Lloh300:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.97@PAGEOFF
	mov x1, x0
	bl core::panicking::panic_bounds_check
LBB69_61:
Lloh301:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.100@PAGE
Lloh302:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.100@PAGEOFF
	mov x1, x24
	bl core::panicking::panic_bounds_check
LBB69_62:
Lloh303:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.99@PAGE
Lloh304:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.99@PAGEOFF
	mov x0, x22
	mov x1, x23
	bl core::panicking::panic_bounds_check
LBB69_63:
Lloh305:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.98@PAGE
Lloh306:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.98@PAGEOFF
	mov x0, x8
	bl core::panicking::panic_bounds_check
LBB69_64:
Lloh307:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.105@PAGE
Lloh308:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.105@PAGEOFF
	mov x0, x22
	mov x1, x23
	bl core::panicking::panic_bounds_check
LBB69_65:
Lloh309:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.106@PAGE
Lloh310:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.106@PAGEOFF
	mov x0, x22
	ldr x1, [sp, #120]
	bl core::panicking::panic_bounds_check
LBB69_66:
Lloh311:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.107@PAGE
Lloh312:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.107@PAGEOFF
	mov x0, x22
	ldr x1, [sp, #112]
	bl core::panicking::panic_bounds_check
LBB69_67:
Lloh313:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.112@PAGE
Lloh314:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.112@PAGEOFF
	mov x0, x27
	mov w1, #5
	bl core::panicking::panic_bounds_check
LBB69_68:
Lloh315:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.109@PAGE
Lloh316:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.109@PAGEOFF
	mov x0, x22
	ldr x1, [sp, #72]
	bl core::panicking::panic_bounds_check
LBB69_69:
Lloh317:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.110@PAGE
Lloh318:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.110@PAGEOFF
	mov x0, x22
	ldr x1, [sp, #64]
	bl core::panicking::panic_bounds_check
LBB69_70:
Lloh319:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.111@PAGE
Lloh320:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.111@PAGEOFF
	mov x0, x27
	mov w1, #5
	bl core::panicking::panic_bounds_check
LBB69_71:
Lloh321:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.108@PAGE
Lloh322:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.108@PAGEOFF
	mov x0, x27
	mov w1, #5
	bl core::panicking::panic_bounds_check
LBB69_72:
Lloh323:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.101@PAGE
Lloh324:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.101@PAGEOFF
	mov x0, x22
	bl core::panicking::panic_bounds_check
LBB69_73:
Lloh325:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.102@PAGE
Lloh326:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.102@PAGEOFF
	mov x0, x22
	mov x1, x9
	bl core::panicking::panic_bounds_check
LBB69_74:
Lloh327:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.103@PAGE
Lloh328:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.103@PAGEOFF
	mov x0, x22
	mov x1, x27
	bl core::panicking::panic_bounds_check
LBB69_75:
Lloh329:
	adrp x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.104@PAGE
Lloh330:
	add x2, x2, l_anon.d6bb5dd3a9c75ceda41c2f91c42e3a04.104@PAGEOFF
	mov x0, x22
	mov x1, x3
	bl core::panicking::panic_bounds_check
	.loh AdrpAdd	Lloh297, Lloh298
	.loh AdrpAdd	Lloh299, Lloh300
	.loh AdrpAdd	Lloh301, Lloh302
	.loh AdrpAdd	Lloh303, Lloh304
	.loh AdrpAdd	Lloh305, Lloh306
	.loh AdrpAdd	Lloh307, Lloh308
	.loh AdrpAdd	Lloh309, Lloh310
	.loh AdrpAdd	Lloh311, Lloh312
	.loh AdrpAdd	Lloh313, Lloh314
	.loh AdrpAdd	Lloh315, Lloh316
	.loh AdrpAdd	Lloh317, Lloh318
	.loh AdrpAdd	Lloh319, Lloh320
	.loh AdrpAdd	Lloh321, Lloh322
	.loh AdrpAdd	Lloh323, Lloh324
	.loh AdrpAdd	Lloh325, Lloh326
	.loh AdrpAdd	Lloh327, Lloh328
	.loh AdrpAdd	Lloh329, Lloh330
