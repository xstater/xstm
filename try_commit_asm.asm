.section .text.xstm::context::Context::try_commit,"ax",@progbits
	.globl	xstm::context::Context::try_commit
	.p2align	4, 0x90
.type	xstm::context::Context::try_commit,@function
xstm::context::Context::try_commit:
	.cfi_startproc
	.cfi_personality 155, DW.ref.rust_eh_personality
	.cfi_lsda 27, .Lexception3
	push rbp
	.cfi_def_cfa_offset 16
	push r15
	.cfi_def_cfa_offset 24
	push r14
	.cfi_def_cfa_offset 32
	push r13
	.cfi_def_cfa_offset 40
	push r12
	.cfi_def_cfa_offset 48
	push rbx
	.cfi_def_cfa_offset 56
	sub rsp, 216
	.cfi_def_cfa_offset 272
	.cfi_offset rbx, -56
	.cfi_offset r12, -48
	.cfi_offset r13, -40
	.cfi_offset r14, -32
	.cfi_offset r15, -24
	.cfi_offset rbp, -16
	xor eax, eax
	cmp rax, qword ptr [rdi]
	jo .LBB9_47
	mov qword ptr [rsp + 24], rsi
	mov qword ptr [rsp + 32], 0
	mov qword ptr [rsp + 40], 8
	mov qword ptr [rsp + 48], 0
	mov r14, qword ptr [rdi + 32]
	mov qword ptr [rsp + 16], rdi
	mov rax, qword ptr [rdi + 40]
	mov rcx, rax
	shl rcx, 4
	mov qword ptr [rsp + 64], rcx
	test rax, rax
	je .LBB9_9
	mov rax, qword ptr [rsp + 64]
	add rax, r14
	mov qword ptr [rsp + 8], rax
	mov ebx, 8
	mov r12d, 1
	xor r15d, r15d
	jmp .LBB9_3
	.p2align	4, 0x90
.LBB9_8:
	lea rax, [r14 + r15]
	mov qword ptr [rbx + r15], rbp
	mov qword ptr [rbx + r15 + 8], r13
	mov qword ptr [rsp + 48], r12
	inc r12
	add r15, 16
	add rax, 16
	cmp rax, qword ptr [rsp + 8]
	je .LBB9_9
.LBB9_3:
	mov rdi, qword ptr [r14 + r15]
	mov rax, qword ptr [r14 + r15 + 8]
	call qword ptr [rax + 48]
	mov rbp, rax
	test rax, rax
	je .LBB9_46
	mov r13, rdx
	lea rax, [r12 - 1]
	cmp rax, qword ptr [rsp + 32]
	jne .LBB9_8
	lea rdi, [rsp + 32]
	lea rsi, [rip + .L__unnamed_4]
	call qword ptr [rip + alloc::raw_vec::RawVec<T,A>::grow_one@GOTPCREL]
	mov rbx, qword ptr [rsp + 40]
	jmp .LBB9_8
.LBB9_9:
	mov ebx, 1
	mov rax, qword ptr [rsp + 24]
	lock xadd	qword ptr [rax], rbx
	inc rbx
	js .LBB9_10
	mov r15, qword ptr [rsp + 16]
	mov rax, qword ptr [r15 + 48]
	mov qword ptr [rsp + 8], rax
	inc rax
	cmp rbx, rax
	jle .LBB9_38
	call qword ptr [rip + std::thread::current::current@GOTPCREL]
	xor ecx, ecx
	test rax, rax
	setne cl
	shl ecx, 4
	mov qword ptr [rsp + 104], rax
	mov qword ptr [rsp + 112], rdx
	mov rcx, qword ptr [rdx + rcx]
	mov qword ptr [rsp + 80], rcx
	test rax, rax
	je .LBB9_25
	lock dec	qword ptr [rdx]
	jne .LBB9_25
	lea rdi, [rsp + 112]
	#MEMBARRIER
	call qword ptr [rip + alloc::sync::Arc<T,A>::drop_slow@GOTPCREL]
.LBB9_25:
	mov rax, qword ptr [rsp + 8]
	mov qword ptr [rsp + 88], rax
	mov rcx, qword ptr [r15 + 16]
	test rcx, rcx
	je .LBB9_38
	mov rax, rcx
	mov r12, qword ptr [r15 + 8]
	shl rax, 4
	add rax, r12
	mov qword ptr [rsp + 16], rax
	mov qword ptr [rsp + 24], r14
.LBB9_27:
	mov rdi, qword ptr [r12]
	mov rax, qword ptr [r12 + 8]
	call qword ptr [rax + 32]
	mov qword ptr [rsp + 56], rax
	mov qword ptr [rsp + 96], rbx
	lea rax, [rsp + 80]
	mov qword ptr [rsp + 104], rax
	lea rax, [rip + <std::thread::ThreadId as core::fmt::Debug>::fmt]
	mov qword ptr [rsp + 112], rax
	lea rax, [rsp + 88]
	mov qword ptr [rsp + 120], rax
	mov rax, qword ptr [rip + core::fmt::num::imp::<impl core::fmt::Display for isize>::fmt@GOTPCREL]
	mov qword ptr [rsp + 128], rax
	lea rcx, [rsp + 96]
	mov qword ptr [rsp + 136], rcx
	mov qword ptr [rsp + 144], rax
	lea rcx, [rsp + 56]
	mov qword ptr [rsp + 152], rcx
	mov qword ptr [rsp + 160], rax
	lea rax, [rip + .L__unnamed_5]
	mov qword ptr [rsp + 168], rax
	mov qword ptr [rsp + 176], 5
	mov qword ptr [rsp + 200], 0
	lea rax, [rsp + 104]
	mov qword ptr [rsp + 184], rax
	mov qword ptr [rsp + 192], 4
	lea rdi, [rsp + 168]
	call qword ptr [rip + std::io::stdio::_print@GOTPCREL]
	mov rax, qword ptr [rsp + 56]
	test rax, rax
	js .LBB9_30
	cmp rax, qword ptr [rsp + 8]
	jg .LBB9_46
.LBB9_37:
	add r12, 16
	cmp r12, qword ptr [rsp + 16]
	mov r14, qword ptr [rsp + 24]
	jne .LBB9_27
	jmp .LBB9_38
.LBB9_30:
	mov r15, qword ptr [r12]
	mov qword ptr [rsp + 72], r12
	mov rax, qword ptr [r12 + 8]
	mov r13, qword ptr [rax + 24]
	mov rbp, qword ptr [rsp + 64]
	.p2align	4, 0x90
.LBB9_31:
	test rbp, rbp
	je .LBB9_46
	mov rdi, qword ptr [r14]
	mov rax, qword ptr [r14 + 8]
	call qword ptr [rax + 24]
	mov r12, rax
	mov rdi, r15
	call r13
	add r14, 16
	add rbp, -16
	cmp r12, rax
	jne .LBB9_31
	xor eax, eax
	sub rax, qword ptr [rsp + 56]
	mov r12, qword ptr [rsp + 72]
	cmp rax, qword ptr [rsp + 8]
	jle .LBB9_37
.LBB9_46:
	lea rdi, [rsp + 32]
	call core::ptr::drop_in_place<alloc::vec::Vec<alloc::boxed::Box<dyn xstm::context::write::WriteLogGuard>>>
	mov al, 1
	jmp .LBB9_47
.LBB9_38:
	mov r15, qword ptr [rsp + 40]
	mov r14, qword ptr [rsp + 48]
	shl r14, 4
	.p2align	4, 0x90
.LBB9_39:
	test r14, r14
	je .LBB9_40
	mov rdi, qword ptr [r15]
	mov rax, qword ptr [r15 + 8]
	call qword ptr [rax + 24]
	lea r12, [r15 + 16]
	mov rdi, qword ptr [r15]
	mov rax, qword ptr [r15 + 8]
	add r14, -16
	mov rsi, rbx
	call qword ptr [rax + 32]
	mov r15, r12
	jmp .LBB9_39
.LBB9_40:
	lea rdi, [rsp + 32]
	call core::ptr::drop_in_place<alloc::vec::Vec<alloc::boxed::Box<dyn xstm::context::write::WriteLogGuard>>>
	xor eax, eax
.LBB9_47:
	add rsp, 216
	.cfi_def_cfa_offset 56
	pop rbx
	.cfi_def_cfa_offset 48
	pop r12
	.cfi_def_cfa_offset 40
	pop r13
	.cfi_def_cfa_offset 32
	pop r14
	.cfi_def_cfa_offset 24
	pop r15
	.cfi_def_cfa_offset 16
	pop rbp
	.cfi_def_cfa_offset 8
	ret
.LBB9_10:
	.cfi_def_cfa_offset 272
	lea rdi, [rip + .L__unnamed_6]
	call qword ptr [rip + core::option::unwrap_failed@GOTPCREL]
	ud2
	jmp .LBB9_17
	mov rbx, rax
	mov rdi, rbp
	mov rsi, r13
	call core::ptr::drop_in_place<alloc::boxed::Box<dyn xstm::context::write::ReadLog>>
	jmp .LBB9_18
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
	jmp .LBB9_17
	jmp .LBB9_17
	jmp .LBB9_17
.LBB9_17:
	mov rbx, rax
.LBB9_18:
	lea rdi, [rsp + 32]
	call core::ptr::drop_in_place<alloc::vec::Vec<alloc::boxed::Box<dyn xstm::context::write::WriteLogGuard>>>
	mov rdi, rbx
	call _Unwind_Resume@PLT
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
