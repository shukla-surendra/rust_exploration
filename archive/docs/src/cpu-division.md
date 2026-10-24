# Cores, Threads, vCPUs & Fractional CPU: What's Actually Being Divided

You've seen "CPU" divided several different ways — a physical core, a
"thread" shown as a BIOS setting, a cloud vCPU, and Kubernetes'
`cpu: "500m"`. These are **four genuinely different mechanisms**, at
four different layers, and conflating them is the source of most of the
confusion. This page maps all four, in order from "real hardware" to
"pure scheduling illusion."

## The map, up front

| Concept | Layer | Divisible below 1? | What actually backs it |
|---|---|---|---|
| Physical core | Hardware | No | Real, independent silicon |
| SMT/Hyper-Threading logical processor ("thread") | Hardware/firmware | No — but multiplies the apparent count | One core's execution units, time-shared by two register-file copies |
| vCPU | Hypervisor software | Whole vCPUs only — but count can exceed real logical CPUs | The host OS scheduling threads across real logical CPUs |
| milliCPU / fractional CPU (`cpu: "500m"`) | OS scheduler (cgroups) | Yes, down to 1m | Time-slicing — full clock speed, smaller *share of time* |

## 1. Physical cores — the actual hardware floor

A core is real, independent execution hardware: its own arithmetic
units, its own register file, its own L1/L2 cache (see
[Multicore & SMP's private-vs-shared breakdown](./asm/10-multicore-and-smp.md#whats-private-per-core-and-whats-actually-shared)
for the full inventory of what's genuinely duplicated per core).
**There is no division below one core at the hardware execution
level** — you cannot have "half a core" actually executing an
instruction at a given instant. Everything below this point is either a
different physical mechanism (SMT) or pure scheduling (vCPUs,
milliCPU) — not a smaller unit of silicon.

## 2. SMT / Hyper-Threading — the "thread" you saw as a physical setting

This is what `lscpu`'s `Thread(s) per core: 2` (or Windows Task
Manager's logical-processor count) is showing you, and it's genuinely
a **hardware feature**, toggleable in BIOS — not virtualization, not
software.

**What it actually does:** a single physical core duplicates a small
part of itself — the register file and some front-end instruction-
fetch/decode state — into two (occasionally four, on some POWER
designs) independent copies, while the *execution units themselves*
(the ALUs, the FPU, the cache) stay shared, one physical set. The core
presents itself to the OS as two separate logical processors, each with
its own architectural state, but they're still fighting over the same
underlying execution hardware every cycle.

**Why this helps at all:** a single instruction stream frequently
leaves execution units idle — waiting on a cache miss, a branch
misprediction, a dependency stall. SMT lets a *second*, independent
instruction stream fill those otherwise-wasted cycles. This is a real,
measurable throughput win — typically **15–30%**, not the 100% a naive
"two cores" reading would suggest, because both logical threads are
still sharing one core's worth of actual execution capacity.

**Logical CPU count = physical cores × threads per core.** An "8-core,
16-thread" CPU has 8 real cores, each presenting 2 logical processors —
the OS scheduler sees and schedules onto all 16, with the caveat that
any two of those 16 sharing a physical core are still contending for
one core's real execution units.

**The naming collision to watch for:** this is a **hardware thread** —
completely different from a **software thread**
([Concurrency, Workbook Chapter 8](./workbook/08-concurrency.md)'s
`std::thread::spawn`), which is an OS-scheduled unit of *code*, not a
piece of silicon. The OS schedules software threads *onto* hardware
threads (logical processors) — same word, two different layers, and
genuinely a common source of confusion.

## 3. vCPU — what a hypervisor presents to a guest

A **vCPU** is a hypervisor construct: what a guest VM is told it has,
backed underneath by the *host's* own OS scheduler doing ordinary
scheduling work. Concretely, per [QEMU's use cases](./qemu.md):

```sh
qemu-system-x86_64 -smp 4 -accel kvm ...
```

`-smp 4` tells QEMU to present **4 vCPUs** to the guest. Under the
hood, each vCPU is essentially **a host-side thread** that, when the
host's scheduler runs it, executes guest instructions inside a
hardware-virtualized privilege mode — Intel VT-x's "VMX non-root
mode," AMD-V's equivalent, or on aarch64, **EL2**, the hypervisor
exception level sitting *above* EL1 in
[Privilege Levels & Mode Transitions](./asm/03-privilege-levels-and-mode-transitions.md)'s
table (`hello-kernel` never uses EL2 at all — it runs entirely at EL1,
with no hypervisor involved).

The host's ordinary scheduler (Linux's CFS, the same mechanism
scheduling every other process) then distributes those 4 vCPU threads
across whatever real logical CPUs (physical cores × SMT threads) the
host actually has — using exactly the
[context-switching mechanism](./asm/07-context-switching.md) already
covered for ordinary tasks. A vCPU is not new hardware and not a new
scheduling primitive; it's an ordinary thread that happens to spend its
running time inside hardware-virtualized guest execution instead of
ordinary host code.

**Can you request more vCPUs than you have real logical CPUs?** Yes —
this is normal, called **overcommit**. `-smp 8` on a 4-logical-CPU host
works fine; the host scheduler time-slices the 8 vCPU threads across
the 4 real ones, same as it would with any 8 competing processes. It
degrades gracefully under contention (more time-slicing, less true
parallelism) rather than failing outright — fine for bursty workloads
where not every vCPU is busy simultaneously, which is exactly why cloud
providers overcommit vCPUs across tenants as a cost/density strategy.

## 4. milliCPU / fractional CPU — time-slicing, not silicon-slicing

This is the direct answer to "is there a further division possible
below one core": **not at the hardware level, but yes, at the
scheduling level** — by dividing *time* on a real core, not the core
itself.

Kubernetes' `cpu: "500m"` (500 millicpu = 0.5 CPU — the exact syntax
already sitting, unexplained, in
[Kubernetes & Infrastructure](./production/12-kubernetes-and-infrastructure.md)'s
resource-limits example) is implemented via **Linux cgroups CPU
quota** — this section covers just the `cpu` controller; see
[cgroups, Namespaces & What a Container Actually Is](./cgroups-and-containers.md)
for the general mechanism (memory/IO/process-count limits work the same
way) and how it combines with namespaces to form what "a container"
actually is:

```
cpu.max: 50000 100000
         ^^^^^ ^^^^^^
         quota  period   (both in microseconds)
```

A period of 100ms with a quota of 50ms means: this cgroup (container)
may consume **at most 50ms of real CPU time out of every 100ms
window**. Try to use more, and the kernel **throttles** it — blocks it
from running at all until the next period starts, regardless of whether
a core is sitting idle elsewhere.

**The critical, easy-to-misread part: the process runs at full clock
speed the entire time it's scheduled.** "500 millicpu" does not mean
"a core running at half frequency" — it means "a real, full-speed core
for half of any given wall-clock window, on average." This is precisely
[context switching](./asm/07-context-switching.md)'s save/restore
mechanism, just triggered by quota/period accounting instead of plain
round-robin fairness — the CPU itself never runs slower; it's handed to
someone else more often.

AWS ECS/Fargate's fractional vCPU units work the same way underneath —
Fargate tasks run on Linux, backed by the identical cgroups mechanism,
just exposed through AWS's own API/billing model instead of a raw
`cpu.max` file.

## 5. Doing this locally: yes, your 8 logical CPUs can become N vCPUs

Direct answer to "if I have 8 cores in logical, can it be divided into
8 vCPUs" — yes, and the number doesn't even have to be 8:

**a) A full VM, via QEMU** (as above):

```sh
qemu-system-x86_64 -smp 8 -accel kvm    # Linux host, hardware-accelerated
qemu-system-x86_64 -smp 8 -accel hvf    # macOS host (Apple Silicon or Intel)
```

`-smp` can be any number — fewer than your real logical CPUs (a
"downsized" guest), exactly matching them, or more (overcommit, per
above).

**b) Fractional CPU with no VM at all, via cgroups directly** — the
same mechanism Kubernetes uses, invoked locally:

```sh
docker run --cpus=0.5 my-image          # Docker: cgroups quota/period underneath
systemd-run --property=CPUQuota=50% ... # systemd: same cgroups mechanism directly
```

**c) On macOS specifically** — there's no native cgroups (that's a
Linux kernel feature). Docker Desktop for Mac works around this by
running a lightweight Linux VM in the background (via Apple's
Virtualization.framework) purely so it has a real Linux kernel to apply
cgroups-based limits *inside* — `docker run --cpus=0.5` on a Mac is
still, underneath, Linux cgroups running inside that hidden VM, not a
native macOS mechanism.

**d) A related but distinct concept worth not confusing with the
above: CPU affinity** (`taskset` on Linux) — pinning a process to
specific logical CPUs, controlling **which** ones it may run on, not
**how much time** it gets. Affinity and cgroups quota are independent
knobs — you can limit a process to 0.5 CPU worth of time *and* restrict
it to only ever run on logical CPUs 0–3, simultaneously.
