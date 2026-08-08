# 6. Hands-On: Building a Disk Image in Rust

Every concept in chapters 2 and 5 — sectors, LBAs, boot sectors, GPT,
filesystems — is implemented, in real Rust, in
[`use_cases/disk_exploration`](../../../use_cases/disk_exploration) in
this repo. This chapter walks through running it and inspecting what it
actually produced, byte for byte. This is the "first principles, not
just theory" payoff of this whole section.

## What the program does, end to end

```sh
cd use_cases/disk_exploration
cargo run
```

Reading `src/main.rs` and `src/gpt_fat.rs` top to bottom, in the order
they actually execute:

1. **Opens (or creates) `disk.img`** as a plain file, sized 1 MiB if new
   — this file *is* the "disk" for the rest of the program. There's no
   real hardware involved at all; it's chapter 2's "a virtual disk is
   just a big file" made completely literal.
2. **Writes a hand-built boot sector to LBA 0** — the `0xEB 0x3C 0x90`
   / `0x55 0xAA` bytes from chapter 5, written via the `BlockDevice`
   trait's `write_sector`. Reads it back and asserts the signature bytes
   round-tripped correctly.
3. **Calls `make_gpt_and_fat()`**, which:
   - Resizes the image to 64 MiB.
   - Writes a **protective MBR** at LBA 0 (overwriting the hand-built
     boot sector from step 2 — real MBR/GPT bytes now live there
     instead).
   - Creates a **GPT** (GUID Partition Table) and adds one partition
     named `"oxide"`, sized to use most of the 64 MiB.
   - Reopens the GPT to read back the *actual* start/end LBAs the `gpt`
     crate assigned the partition (GPT reserves some LBAs at the head
     and tail of the disk for its own metadata — chapter 2's "the
     firmware/format defines the layout, not you").
   - **Formats a FAT filesystem** inside just that partition's LBA
     range, using a `PartitionSlice` wrapper (in `gpt_fat.rs`) that
     makes a sub-range of the file behave like a self-contained,
     independently-seekable disk — this is chapter 2's "filesystem
     lives on top of a range of LBAs" made concrete in code.
   - **Writes a real file**, `HELLO.TXT`, containing `Hello from
     Oxide!\n`, into that freshly-formatted FAT filesystem.

The end result: `disk.img` is now a genuine, byte-for-byte valid disk
image — protective MBR, a real GPT partition table, one partition,
formatted FAT32, containing one file — built entirely by a ~150-line
Rust program with no OS-level disk APIs, no root privileges, just plain
file I/O.

## Inspecting the result — proving it's real

Everything below works because `disk.img` is a completely ordinary
file — any tool that understands MBR/GPT/FAT can read it, without
knowing or caring it was built by a Rust program instead of real
hardware.

**Look at the raw bytes directly:**

```sh
hexdump -C disk.img | less
strings disk.img | grep HELLO   # find HELLO.TXT's actual text content in the raw bytes
```

**Look at just the first sector (the protective MBR from step 3):**

```sh
dd if=disk.img bs=512 count=1 | hexdump -C
# offset 0x1BE..0x1FD = the 4 MBR partition entries
# offset 0x1FE..0x1FF = should be 55 AA — chapter 5's boot signature
```

**Inspect the GPT partition table:**

```sh
gdisk -l disk.img     # human-readable GPT partition listing
```

**Mount it as a real filesystem and see `HELLO.TXT` (Linux):**

```sh
sudo losetup -Pf disk.img       # attach as /dev/loop0, exposing /dev/loop0p1 for the partition
sudo mkdir -p /mnt/diskimg
sudo mount /dev/loop0p1 /mnt/diskimg
ls -l /mnt/diskimg               # HELLO.TXT should be right there
cat /mnt/diskimg/HELLO.TXT       # Hello from Oxide!

sudo umount /mnt/diskimg
sudo losetup -d /dev/loop0
```

Seeing `HELLO.TXT` show up in a normal `ls`, mounted through the
regular Linux filesystem stack, on a file a small Rust program built
from nothing, is the concrete version of everything chapters 2 and 5
described in prose — there really is no magic between "bytes in a file"
and "a mounted, browsable disk."

## Broader CLI cheat-sheet, for any disk image or real device

The commands above are specific to this project's image; these are the
general-purpose versions worth keeping handy for any future
system-level exploration in this section.

**Quick info (size, sector size, partitions) — Linux, real or
loop-mounted devices:**

```sh
lsblk -o NAME,SIZE,TYPE,ROTA,PHY-SECT,LOG-SEC,MODEL
fdisk -l /dev/sdX          # or on a loop device, see below
parted -l                   # more human-friendly
blockdev --getss /dev/sdX   # logical sector size only
cat /sys/block/sdX/queue/hw_sector_size
```

**Raw image → loop device, so partition-aware tools can read it:**

```sh
sudo losetup --find --show -P disk.img   # prints /dev/loopN, exposes /dev/loopNp1, p2, ...
sudo fdisk -l /dev/loopN                  # Start/End LBAs, sector size
sudo partx -a /dev/loopN                  # add partition mappings if needed
sudo losetup -d /dev/loopN                # detach when done
```

**Other virtual-disk formats:**

```sh
qemu-img info disk.qcow2            # virtual size, cluster size, backing file
VBoxManage showhdinfo disk.vdi
vmware-vdiskmanager -R disk.vmdk
```

**Windows equivalents:**

```powershell
Get-Disk
Get-Partition
Mount-VHD -Path .\disk.vhdx
Get-Volume
diskpart   # then: list disk / select disk N / list partition
```

**macOS equivalents:**

```sh
hdiutil imageinfo disk.dmg
hdiutil attach disk.img
diskutil list
fdisk /dev/diskN
```

Whether the target is a physical device or a virtual image, once the
OS "sees" it as a block device, `fdisk -l`/`gdisk -l` (Linux) or
`diskutil` (macOS) will report Start LBA, End LBA, and sector sizes —
the same LBA vocabulary from chapter 2, on real hardware.

## Extending this yourself

A few natural next experiments, in roughly increasing difficulty, if
you want to keep building on this crate rather than just reading about
it:

- Write a second file into the FAT filesystem and confirm it shows up
  alongside `HELLO.TXT`.
- Change `SECTOR_SIZE` reasoning: try formatting with 4096-byte sectors
  (chapter 2's "Advanced Format") instead of 512, and see what the `gpt`
  crate does differently with the reserved head/tail LBA counts.
- Add a second partition, and inspect with `gdisk -l` to see two
  entries instead of one.
- Read `BOOTX64.EFI`-style UEFI booting (chapter 5) and try building an
  ESP-type partition instead of a plain FAT partition — this is a
  meaningfully bigger jump (needs an actual `.efi` binary to place in
  it), a good candidate for a future chapter in this section once you've
  worked through it.
