# cgroups, Namespaces & What a Container Actually Is

[Cores, Threads, vCPUs & Fractional CPU](./cpu-division.md) covered
*one* cgroups controller (`cpu.max`) in depth. This page is the general
mechanism — what cgroups actually are, the other resources they
control, and the sibling kernel feature (namespaces) that, together
with cgroups, is the *entire* substance of what Docker/Kubernetes call
a "container." No VM, no separate kernel, no magic — two Linux kernel
features and a filesystem trick.

## What cgroups actually is

**"cgroups"** = **control groups**, a Linux kernel feature (not a
library, not something Docker invented) for grouping processes together
and limiting/accounting for the resources that group can use. It's
exposed directly as a special filesystem — literally
`/sys/fs/cgroup/...` — no separate command-line tool required to use
it at the lowest level, though `docker`, `systemd`, and `kubelet` all
provide friendlier interfaces on top.

**In layman's terms, and answering the natural first question directly:
no, a CPU cgroup limit is *not* "how many cores or threads you get."**
Picture a CPU core as one very fast cashier who serves one customer at
a time but switches customers so quickly it feels simultaneous. A
cgroup CPU limit is a **ticket for how much of the cashier's time
you're allowed** — not which cashier, and not how many. `cpu: "500m"`
is "you get 5 minutes of the cashier's attention out of every 10-minute
window"; when it *is* your turn, you're served at completely normal,
full speed — you're only capped on how *often*/how *much total time*
you get called up. Even a limit *above* 1.0 (`cpu: "2"`) is still a
time budget, not a core assignment: "up to 2 cores' worth of CPU-time
per second, total, spread across however many threads you run" — the
kernel is still free to move those threads between any physical
cores/threads it likes. **Picking specific cores** is a different,
separate cgroups feature entirely — the `cpuset` controller, covered
below — which almost nothing uses by default. See
[Cores, Threads, vCPUs & Fractional CPU](./cpu-division.md) for the
full "why it's time-slicing, not silicon-slicing" explanation this
analogy is standing in for.

- A **cgroup** is a directory in that filesystem.
- **Adding a process to a cgroup** means writing its PID into that
  directory's `cgroup.procs` file.
- Cgroups form a **hierarchy** (a tree) — child cgroups inherit and are
  bounded by whatever limits their parent sets, the same nesting shape
  Kubernetes' own pod → container resource limits reflect.

```sh
mkdir /sys/fs/cgroup/mygroup
echo "50000 100000" > /sys/fs/cgroup/mygroup/cpu.max   # 0.5 CPU, per Cores, Threads & vCPUs
echo $$ > /sys/fs/cgroup/mygroup/cgroup.procs           # move the current shell into it
```

From that point on, *every command run from that shell* (and anything
it forks) is capped at 0.5 CPU — no container, no Docker, just raw
kernel filesystem writes. This is genuinely what `docker run --cpus=0.5`
and Kubernetes' `resources.limits.cpu` do underneath, through a friendlier
API.

## Controllers — one per resource type

Each resource cgroups can limit is governed by a separate **controller**
(also called a subsystem), each with its own set of files inside a
cgroup directory:

| Controller | Governs | Example file |
|---|---|---|
| `cpu` | CPU time — quota/period, or relative shares | `cpu.max` (see [Cores, Threads, vCPUs & Fractional CPU](./cpu-division.md)) |
| `memory` | RAM usage limit, swap, OOM-kill behavior | `memory.max` |
| `io` (`blkio` in the older interface) | Disk I/O bandwidth/IOPS throttling | `io.max` |
| `pids` | Maximum number of processes/threads | `pids.max` — the direct defense against fork bombs |
| `cpuset` | *Which specific* logical CPUs/NUMA nodes a group may run on | `cpuset.cpus` — this is cgroups' version of the CPU-affinity concept ([Cores, Threads, vCPUs & Fractional CPU, §5](./cpu-division.md#5-doing-this-locally-yes-your-8-logical-cpus-can-become-n-vcpus)'s `taskset`, done per-group instead of per-process) |
| `devices` | Which device nodes (`/dev/*`) are accessible | `devices.allow`/`devices.deny` |
| `freezer` | Pause/resume every process in the group at once | `cgroup.freeze` |

`memory.max`, `pids.max`, and `io.max` are exactly why a misbehaving
container can be killed for using too much RAM, or blocked from
forking unboundedly, or throttled on disk I/O — the same quota
mechanism as `cpu.max`, one controller per resource.

## cgroup v1 vs v2 — worth knowing which one you're looking at

- **v1** (older): each controller gets its **own separate** filesystem
  hierarchy — a process could belong to one cgroup for CPU limits and a
  *completely different* cgroup tree for memory limits. Flexible, but
  confusing and inconsistent — CPU quota/period were even split across
  two separate files (`cpu.cfs_quota_us`, `cpu.cfs_period_us`).
- **v2** (current default on most distros since ~2021 — current
  Ubuntu/Fedora/RHEL 9): **one unified hierarchy** — every controller
  applies to the same tree of groups, and related settings are
  consolidated (`cpu.max` holds both quota *and* period in one file,
  space-separated, exactly the format shown above).

If you're inspecting a real system and the file names don't match what
you expect, checking which version is mounted (`mount | grep cgroup`)
is the first thing to check.

## Namespaces — the other half of "container," and a genuinely separate mechanism

Cgroups answer **"how much can this process use."** They say nothing
about **"what can this process see"** — that's a distinct Linux kernel
feature, **namespaces**, and conflating the two is the single most
common source of confusion about how containers work.

| Namespace | Isolates |
|---|---|
| PID | Its own process tree — a container's "process 1" has no visibility into host processes |
| Mount | Its own filesystem view — what paths exist, what's mounted where |
| Network | Its own network interfaces, routing table, and port space |
| UTS | Its own hostname |
| IPC | Its own shared memory segments, semaphores |
| User | Its own UID/GID mapping — lets "root" *inside* the container map to an unprivileged, ordinary user on the host |
| Cgroup | Its own view of the cgroup hierarchy itself, so a process inside can't see or manipulate cgroups above its own |

A process gets put into a new namespace via `unshare` (a command and a
syscall) or by passing `CLONE_NEWPID`/`CLONE_NEWNET`/etc. flags to the
`clone()` syscall when it's created — mechanically unrelated to
anything cgroups does.

## Putting it together: what a container actually is

**A container is an ordinary Linux process**, given three things, none
of which is a new kind of process or a new kernel:

1. **Its own set of namespaces** — so it can't see the host's other
   processes, network interfaces, or (with a mount namespace + an
   overlay filesystem) even the host's real file tree.
2. **Membership in a cgroup** — so it's capped on CPU, memory, I/O,
   process count, exactly as described above.
3. **A different root filesystem** (via a mount namespace plus
   something like `overlayfs`, which is what makes Docker image layers
   work — each layer is a read-only filesystem stacked under a
   writable one).

Docker, `containerd`, and `runc` don't invent any new isolation
mechanism — they're orchestration layers that create the namespaces,
write the cgroup files, and set up the layered filesystem, all using
kernel features that predate Docker itself by years.

## The container "C-word" alphabet soup: containerd, CRI, runc, OCI & the EKS-specific ones

The one-line "Docker, `containerd`, and `runc` don't invent any new
isolation mechanism" above is worth unpacking — these, plus a handful
of similarly-named things you'll run into on EKS specifically, form a
real *stack*, each layer talking to the one below it. Here's the actual
call chain, top to bottom, for a pod starting on an EKS worker node:

```
kubelet  (the per-node Kubernetes agent)
   ↓ speaks CRI
containerd  (the container runtime daemon actually running on the node)
   ↓ shells out to, per the OCI Runtime Spec
runc  (the low-level tool that actually does the work)
   ↓ performs
namespaces + cgroups + overlayfs  (everything this page already covered)
   ↓ produces
your container's process, running
```

- **`containerd`** — a **container runtime**: the daemon that pulls
  images, unpacks their layers, and manages container lifecycle
  (start/stop/list) on a single machine. It was originally *part of*
  Docker, then extracted and donated to the CNCF as its own project —
  Docker itself is built on top of it (below). **This is EKS's default
  node runtime** — since Kubernetes 1.24 removed built-in Docker
  support ("dockershim"), EKS worker nodes run `containerd` directly.
  You still *build* images with `docker build` on your laptop, but the
  cluster running them typically never has Docker Engine installed at
  all — only `containerd`.
- **CRI (Container Runtime Interface)** — not a program, a **protocol**:
  the API `kubelet` speaks to *whatever* container runtime is
  installed, so Kubernetes itself doesn't need to know or care whether
  that runtime is `containerd`, `CRI-O`, or something else. This is
  exactly why swapping the underlying runtime (e.g., the dockershim
  removal above) didn't require rewriting Kubernetes itself — only
  runtimes implementing CRI needed to exist.
- **`CRI-O`** — a *different* container runtime, built from scratch
  specifically to implement CRI and nothing else (no standalone
  `docker`-style CLI, no extra features) — `containerd`'s main
  competitor in this slot. OpenShift defaults to it; EKS defaults to
  `containerd` instead, but it's worth recognizing the name as "the
  other one" rather than a typo of `containerd`.
- **`runc`** — the actual tool that does the low-level work this whole
  page has been describing: creating the namespaces, writing the
  cgroup files, setting up the root filesystem, and finally executing
  the container's process. `containerd` doesn't do this part itself —
  it invokes `runc` (or another compliant runtime) as a subprocess for
  every container start. `runc` is a direct, literal implementation of
  the **OCI Runtime Spec**, next.
- **OCI (Open Container Initiative)** — not a program either, a
  **specification body**. Two specs matter here: the **OCI Image
  Spec** (the standard format for container images and their layers —
  what `docker build` produces, and what makes an image built with
  Docker runnable by `containerd`/`CRI-O`/Podman interchangeably) and
  the **OCI Runtime Spec** (the standard `runc` implements — meaning
  other tools, like `crun` or Kata Containers' runtime, can be swapped
  in for `runc` and still work with `containerd`).
- **Docker itself** — worth clarifying it isn't one monolithic thing:
  "Docker" is a whole platform (`dockerd`, the Docker CLI, Docker
  Compose, ...) that, underneath, **runs `containerd`, which runs
  `runc`** — the exact same stack as the diagram above. Building an
  image with `docker build` on your laptop and running that same image
  under bare `containerd` on an EKS node isn't a coincidence — they
  share the OCI Image Spec and, several layers down, the same `runc`.

## EKS-specific "C" plugin interfaces — a related but distinct pair

Two more, easy to lump in with the above but solving a different
problem (networking and storage, not process isolation):

- **CNI (Container Network Interface)** — the plugin spec for how a
  pod *gets* network connectivity (an IP address, routes) when it
  starts — the networking equivalent of CRI. **EKS uses the "Amazon
  VPC CNI"** specifically, which gives each pod a real IP address from
  your actual VPC's address space (rather than an overlay network),
  a deliberate AWS design choice that makes pods directly routable
  within the VPC.
- **CSI (Container Storage Interface)** — the plugin spec for how a
  persistent volume gets attached to a pod. **EKS commonly uses the
  "Amazon EBS CSI driver"** to back Kubernetes `PersistentVolume`s with
  real EBS volumes.

Both are, structurally, the same idea as CRI: a standard plugin
interface so Kubernetes doesn't hardcode support for any one vendor's
networking or storage implementation.

## Nearby "C" names that are *not* part of the runtime stack

A few more you'll see in the same EKS/Kubernetes contexts that are
worth *not* folding into the above — different layer entirely, config/
cluster-management vocabulary rather than container-runtime mechanics
(see [Kubernetes & Infrastructure](./production/12-kubernetes-and-infrastructure.md)
for where these actually belong):

| Term | What it actually is |
|---|---|
| **Cluster** | The whole EKS deployment — control plane + worker nodes together |
| **Control plane** | The Kubernetes management components (API server, `etcd`, scheduler) — in EKS, AWS runs and manages this for you; you never `ssh` into it |
| **ConfigMap** | A Kubernetes object for injecting configuration into pods — application config, not container isolation |
| **ClusterIP** | A Kubernetes `Service` type — an internal-only virtual IP load-balancing across a set of pods |
| **CoreDNS** | The default DNS server *addon* running inside the cluster, resolving service names to `ClusterIP`s |
| **cAdvisor** | Built into `kubelet` — collects per-container resource usage by reading the exact cgroup files described earlier on this page, feeding Kubernetes' metrics pipeline |

`cAdvisor` is the one genuinely worth double-clicking: it's not a
separate concept at all, just a piece of software that reads
`memory.max`/`cpu.max`/etc. (from [Controllers, above](#controllers--one-per-resource-type))
on your behalf and turns them into the metrics `kubectl top pod` shows
you.

## The contrast worth being explicit about: container vs. VM

Directly against [what QEMU/KVM actually virtualizes](./qemu.md):

| | Container (cgroups + namespaces) | VM (QEMU/KVM, per [QEMU](./qemu.md)) |
|---|---|---|
| Kernel | **Shared** with the host — one kernel, many processes | Its own, separate, fully emulated/virtualized kernel |
| Isolation mechanism | Namespaces (visibility) + cgroups (resource limits) | Hardware virtualization — [privilege level EL2/VT-x](./cpu-division.md#3-vcpu--what-a-hypervisor-presents-to-a-guest), a real vCPU |
| Startup time | Milliseconds — it's just a process starting | Seconds — a whole kernel has to boot |
| Overhead | Very low — no duplicated kernel, no emulated hardware | Higher — a full guest OS, its own memory management, its own drivers |
| Can run a different OS than the host? | No — same kernel, so effectively the same OS family (Linux container needs a Linux host kernel) | Yes — genuinely arbitrary guest OS, which is exactly how [`hello-kernel`](./systems/07-hello-kernel-overview.md) and [OxideOS](./oxideos/00-overview.md) run under QEMU on any host |

Neither is "better" — they solve different problems. A container is
cheap isolation for trusted-ish workloads sharing one kernel (the
normal shape of a Kubernetes cluster, see
[Kubernetes & Infrastructure](./production/12-kubernetes-and-infrastructure.md));
a VM is strong isolation (a genuinely separate kernel, a real
hardware-virtualization boundary) at a real cost in overhead and
startup time — and it's the *only* option when the guest needs to be a
different OS/architecture than the host at all, which is precisely why
this whole book's OS-dev examples run in QEMU rather than a container.

## systemd's relationship to cgroups

Every service `systemd` manages gets its own cgroup automatically —
this is *why* `systemd-run --property=CPUQuota=50%`
([Cores, Threads, vCPUs & Fractional CPU](./cpu-division.md#5-doing-this-locally-yes-your-8-logical-cpus-can-become-n-vcpus))
works: it's writing to the exact same `cpu.max`-style files described
above, just through `systemd`'s own management layer instead of by
hand. `systemctl status <service>` shows the cgroup path it's running
in directly.

## Inspecting cgroups on a real system

```sh
cat /proc/self/cgroup           # which cgroup(s) the current process is in
systemd-cgls                     # visualize the whole cgroup tree
cat /sys/fs/cgroup/<path>/cpu.max         # a specific group's CPU limit
docker inspect <container> | grep -i cpu  # Docker's view of the same underlying limits
```

None of these need a container runtime installed — they work on any
modern Linux system, since cgroups is a kernel feature Docker merely
exposes a friendlier interface to.
