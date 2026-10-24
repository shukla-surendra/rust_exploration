# 5. The Boot Process: BIOS/MBR vs UEFI/GPT

## What makes a device "bootable"?

Nothing magical — it comes down to specific bytes in specific
locations, checked by the firmware (BIOS or UEFI) at power-on. Which
bytes, and where, depends on which of the two boot schemes below the
system uses.

## BIOS + MBR (the classic scheme)

1. BIOS loads the very first sector of the chosen disk — LBA 0, 512
   bytes (chapter 2's addressing) — into memory at a fixed address
   (`0x7C00`). Before this step, BIOS already set up the **Interrupt
   Vector Table** at physical address `0x00000` — see
   [Interrupts & Exceptions](../asm/04-interrupts-and-exceptions.md#before-the-idt-the-real-mode-ivt)
   for what that table is and why the boot sector's `INT 0x13` disk
   calls (below) work at all.
2. It checks the **last two bytes** of that sector. They must be the
   signature `0x55AA`.
3. If valid → BIOS runs the bootloader code from that sector. If
   invalid → "no bootable device."

Additionally, the **MBR partition table** (four entries, stored at
offset `0x1BE` within that same first sector) has a 1-byte "bootable
flag" per entry: `0x80` means bootable ("active"), `0x00` means not.
Exactly one partition should be marked active at a time — BIOS-era MBR
bootloaders look for it to know which partition to hand control to
next.

This is genuinely just bytes — `use_cases/disk_exploration/src/main.rs`
writes this exact signature by hand, in Rust:

```rust
let mut boot = [0u8; SECTOR_SIZE as usize];
boot[0..3].copy_from_slice(&[0xEB, 0x3C, 0x90]); // JMP + NOP
boot[510] = 0x55; boot[511] = 0xAA;              // the 0x55AA signature
disk.write_sector(0, &boot);
```

`boot[510]`/`boot[511]` are the last two bytes of a 512-byte sector —
exactly the two bytes BIOS checks. A real BIOS, pointed at this
`disk.img`, would consider it bootable purely because of these two
bytes being correct; nothing else about the sector's contents matters
for that specific check.

## Stage 1 vs Stage 2 — why boot code is almost never just 446 bytes

The BIOS+MBR flow above only guarantees **one thing gets loaded**: 512
bytes, executed in 16-bit real mode, with no filesystem driver, no
`malloc`, nothing but raw BIOS interrupts. 446 bytes of actual code
(bytes `0`–`445`, before the partition table at `0x1BE`) is nowhere
near enough to understand a real filesystem and locate a
multi-megabyte kernel file on it by name. Real bootloaders solve this
by chaining two stages:

- **Stage 1** — exactly what's described above: lives in the MBR's 446
  code bytes, has one job, load a larger **Stage 2** image from a
  known disk location into memory, then jump to it.
- **Stage 2** — no longer size-constrained, so it can carry a real
  filesystem driver (FAT, ext2), present a boot menu, and load the
  actual OS kernel by filename rather than raw sector number.

GRUB is the most common Stage 2 loader in practice — installing it
writes a tiny Stage 1 into the MBR and a much larger Stage 2 (with
filesystem support) elsewhere on disk, which is why GRUB installs need
free space right after the MBR on BIOS-booted disks (the "BIOS boot
partition" convention, even on otherwise-GPT disks).

## A third loading mechanism: Multiboot2

BIOS+MBR (above) and UEFI+GPT (below) both answer "how does firmware
find and start *something*." **Multiboot2** answers a narrower
question one level up: once a Stage 2 loader like GRUB is already
running, how does *it* recognize a raw kernel binary as bootable and
know where to jump?

GRUB scans the **first 32 KiB** of whatever file it loads (typically
the kernel's own ELF) for a specific header:

```
magic:    0xE85250D6          (4 bytes, little-endian)
arch:     0  (x86, 32-bit protected mode) or 4 (MIPS)
len:      total header size in bytes, including all tags
checksum: computed so magic + arch + len + checksum ≡ 0 (mod 2^32)
...tags...
end tag:  type = 0, size = 8
```

If the magic and checksum check out, GRUB treats the file as
Multiboot2-capable and jumps to its entry point after setting up a
defined machine state (protected mode, a stack, a memory map handed to
the kernel) — no BIOS interrupts needed from that point on, since GRUB
already did the filesystem/loading work Stage 1 alone couldn't. In
Rust, this header is just **data**, emitted as a `static`, never
executed — three attributes make that placement correct:

```rust
#[unsafe(no_mangle)]
#[unsafe(link_section = ".multiboot2")]
#[used]
pub static MULTIBOOT2_HEADER: MbHeader = MbHeader { /* fields as above */ };
```

- `#[unsafe(no_mangle)]` — conventional here, though GRUB finds this
  particular static by scanning raw bytes for the magic number, not by
  symbol name.
- `#[unsafe(link_section = ".multiboot2")]` — without this, the linker
  is free to place the static anywhere, including well past the first
  32 KiB GRUB actually scans; a linker script `KEEP(*(.multiboot2))`
  placed before `.text` is what guarantees it lands early enough.
- `#[used]` — the header is never read by the kernel's own Rust code,
  only by GRUB externally, so without this the compiler would see an
  unreferenced `static` and legitimately optimize it away entirely.

This is the same `#[unsafe(no_mangle)]`/`#[unsafe(link_section)]`/
`#[used]` combination
[OxideOS's Limine boot process](../oxideos/oxide_cocepts/01_boot_process.md)
uses for its own, different bootloader request protocol — same
underlying problem (getting the linker and a scanning bootloader to
agree on where a data structure lives), different specific bootloader
and header format.

## UEFI + GPT (the modern scheme)

UEFI systems don't use an "active flag" at all. Instead:

- UEFI looks for a partition with the **EFI System Partition (ESP)**
  type — identified by a specific GUID (Globally Unique Identifier) in
  the partition table, not a boot flag.
- That partition must contain a **FAT32** filesystem (chapter 2's
  filesystem layer) with a specific file: `/EFI/BOOT/BOOTX64.EFI` (or a
  vendor-specific `.efi` loader).
- UEFI firmware loads and runs that `.efi` executable directly.

So under UEFI, "bootable" is defined by *partition type + a specific
file existing in a specific path on a FAT32 partition*, plus the
system's NVRAM boot entries (what `efibootmgr` manages on Linux, or the
Windows Boot Manager) recording which ESP to actually try.

**Fresh GPT disks still need a "protective MBR" at LBA 0** — a
backward-compatibility measure so old, GPT-unaware tools don't mistake
the disk for unpartitioned space and overwrite it. This is exactly what
`disk_exploration`'s `gpt_fat.rs` does before writing the real GPT:

```rust
let pmbr = ProtectiveMBR::with_lb_size(
    u32::try_from(num_blocks.saturating_sub(1)).unwrap_or(0xFFFF_FFFF),
);
f.seek(SeekFrom::Start(0))?;
pmbr.overwrite_lba0(&mut f).context("write protective MBR")?;
```

Note this runs in the *same* program, on the *same* `disk.img`, as the
raw `0x55AA` sector-0 write shown above from `main.rs` — in practice the
GPT-writing step (`make_gpt_and_fat()`) overwrites that earlier boot
sector experiment with a real protective MBR + GPT layout. Both are
shown here because both are real, and seeing them side by side is the
clearest way to see that "bootable" is just specific bytes, checked by
specific rules — not a property some files mysteriously have and others
don't.

## Removable devices (USB, DVD) — same rules, different tooling

- **BIOS**: the USB stick needs a valid MBR + boot sector, identical
  rules to a hard disk.
- **UEFI**: the USB stick needs an EFI System Partition with
  `/EFI/BOOT/BOOTX64.EFI`, same as above.

This is why bootable-USB-creation tools (`dd` on Linux, Rufus on
Windows) exist as distinct utilities — they're specifically writing
these boot sectors/EFI partition structures onto a drive, not doing
anything a plain file copy would accomplish.

## Summary

| Scheme | "Bootable" means... |
|---|---|
| BIOS + MBR | Sector 0 ends in `0x55AA`, and a partition table entry has its active flag (`0x80`) set |
| UEFI + GPT | A partition with the ESP type GUID exists, contains FAT32, and has `/EFI/BOOT/BOOTX64.EFI` |

Both are things you can inspect directly — chapter 6's CLI cheat-sheet
includes the exact commands (`dd`/`hexdump`, `gdisk -l`) to look at
either one on a real or virtual disk.
