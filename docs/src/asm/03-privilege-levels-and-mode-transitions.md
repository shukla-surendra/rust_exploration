# 3. Privilege Levels & Mode Transitions: Rings vs Exception Levels

Every OS needs a hardware-enforced boundary between "kernel code that
can do anything" and "user code that can't touch hardware directly or
read other processes' memory." Both architectures provide this, with
different names and different mechanics.

## x86-64: Rings 0–3, via segment selectors

x86 defines four privilege rings; in practice OSes (including OxideOS)
use only two: **Ring 0** (kernel) and **Ring 3** (userspace). The
current ring is encoded in the low 2 bits of the code segment selector
(`cs`) — there's no separate "current privilege level" register to read
directly; it's baked into which segment descriptor is currently loaded.

Getting to Ring 3 requires setting up **segment descriptors** in the
**GDT** (Global Descriptor Table) for both kernel and user code/data,
then performing a controlled transition — historically via `iret`
(interrupt return), which pops not just an instruction pointer but also
a new `cs`/`ss` (stack segment) pair, changing rings as a side effect of
"returning" from a synthetic interrupt frame you constructed by hand.
This is exactly what OxideOS's ["Getting to Ring 3"](../oxideos/study/05_processes.md)
study chapter walks through — building a fake interrupt frame on the
stack and executing `iret` into it, which is currently the standard way
a userspace process's *very first* instruction ever gets run.

A **TSS** (Task State Segment) is also required — not for x86's
long-abandoned hardware task-switching feature, but because it's where
the CPU looks up **which stack to switch to** when an interrupt/syscall
arrives while running in Ring 3 (user stacks aren't trusted for kernel
code to run on). See
[OxideOS Concepts, Chapter 2](../oxideos/oxide_cocepts/02_interrupts_and_cpu.md)
for the GDT/TSS setup in full.

## aarch64: Exception Levels EL0–EL3, no segmentation at all

aarch64 has **no segment registers, no GDT, no TSS** — privilege is
tracked directly by the CPU as the **current Exception Level**:

| Level | Role |
|---|---|
| EL0 | Userspace (unprivileged) |
| EL1 | Kernel (OS) |
| EL2 | Hypervisor (if virtualizing) |
| EL3 | Secure monitor (TrustZone, firmware) |

`hello-kernel` runs entirely at **EL1** — QEMU's `-kernel` boot (see
[Systems, Chapter 9](../systems/09-hello-kernel-boot-to-execution.md))
starts the CPU there directly, and since it never drops to EL0, it
never needs any of this chapter's transition mechanics — worth noting
explicitly, since it's why `hello-kernel`'s source has none of this
code despite the general aarch64 mechanism existing.

Moving *down* a level (EL1 → EL0, kernel → user) uses **`eret`**
("exception return") — analogous to x86's `iret`, but reading its
target state from two dedicated system registers instead of a hand-built
stack frame:

- `ELR_ELx` (Exception Link Register) — the address to resume at.
- `SPSR_ELx` (Saved Program Status Register) — the target Exception
  Level and processor flags to restore.

Moving *up* a level happens automatically and only via a defined
mechanism — an exception (interrupt, `svc` syscall instruction, or a
fault) — never a general-purpose "raise my privilege" instruction. This
is architecturally cleaner than x86's ring model (no segment tables to
configure at all), one reason modern security-sensitive designs
increasingly favor it, though it means the exception vector table
(next chapter) is doing more of the structural work x86 splits between
the IDT and GDT/TSS.

## The common shape underneath both

Despite the completely different mechanics, both architectures enforce
the same invariant: **moving to a lower privilege level requires the
current, more-privileged code to explicitly construct the target
state and execute one specific instruction** (`iret`/`eret`) — there is
no way for user code to grant itself more privilege; only a
kernel-controlled, single, auditable instruction can ever raise
privilege, and it only does so as the far side of a hardware-defined
exception. That single invariant is the entire foundation privilege
separation rests on, on both architectures — everything else in this
chapter is just how each one implements it.
