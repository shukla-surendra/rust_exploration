# 5. The Boot Process: BIOS/MBR vs UEFI/GPT

## What makes a device "bootable"?

Nothing magical — it comes down to specific bytes in specific
locations, checked by the firmware (BIOS or UEFI) at power-on. Which
bytes, and where, depends on which of the two boot schemes below the
system uses.

## BIOS + MBR (the classic scheme)

1. BIOS loads the very first sector of the chosen disk — LBA 0, 512
   bytes (chapter 2's addressing) — into memory at a fixed address
   (`0x7C00`).
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
