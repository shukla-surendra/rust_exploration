# 2. Disks, Sectors & Addressing

## What is a sector?

Think of a disk (HDD, SSD, or USB stick) as a notebook. A page in that
notebook is a **sector** — the smallest unit of data the disk can read
or write in one go. Even writing a single byte still consumes the whole
sector; there's no such thing as writing "half a sector" to a disk.

Historically sectors were 512 bytes. Modern disks often use 4096 bytes
(4 KB, called "Advanced Format") because larger sectors are more
efficient — less per-sector bookkeeping overhead relative to the data
stored — but many drives still *report* 512-byte sectors to stay
compatible with old software that assumes that size.

This is a real, working constant in this repo, not just a fact to
memorize — `use_cases/disk_exploration/src/main.rs` defines exactly
this:

```rust
pub const SECTOR_SIZE: u64 = 512;

pub trait BlockDevice {
    fn size(&self) -> u64;                        // total bytes
    fn read_sector(&self, lba: u64, buf: &mut [u8]);
    fn write_sector(&mut self, lba: u64, buf: &[u8]);
}
```

`BlockDevice` is the Rust encoding of "a disk is something you read and
write one fixed-size sector at a time, addressed by a number" — every
concept in this chapter is what that trait's `lba: u64` parameter
actually means, and chapter 6 walks through a real implementation of it
end to end.

## What is a boot sector?

Imagine the notebook's first page is reserved for instructions on how
to read the rest of the book. That's the boot sector — the very first
sector on a disk, which tells the computer "here's how to start the
operating system." Without it, the computer doesn't know how to start,
even if every other page is full of data. Chapter 5 covers exactly what
bytes have to be in that first sector for it to actually count as
bootable.

## Physical vs. logical addressing: CHS → LBA

Disks *do* have a physical location for every sector, but you almost
never deal with it directly anymore — there have been two addressing
schemes, historically:

- **CHS (Cylinder, Head, Sector)** — the old scheme, addressing a spot
  by which physical track ("cylinder"), which platter surface ("head"),
  and which slice of that track ("sector") it lives on. Rarely used
  today.
- **LBA (Logical Block Address)** — the modern scheme. Instead of
  "cylinder 2, head 3, sector 10," the computer just says "give me block
  number 123456," and the disk's own firmware maps that number to
  wherever the data actually lives physically — including transparently
  remapping around bad sectors, which CHS addressing couldn't hide.

**Analogy:** CHS is asking a librarian for "Row 3, Shelf 2, Book 5." LBA
is just asking for "Book #123456" and letting the librarian (the disk's
firmware) figure out where that physically is. SSDs and USB flash
drives take this even further — they have no physical platters at all,
just flash cells grouped into pages and blocks (chapter 4), but the
controller chip still presents the same simple "linear array of
sector numbers" interface, silently handling wear-leveling and
remapping underneath.

`BlockDevice::read_sector(&self, lba: u64, buf: &mut [u8])` above is
exactly LBA addressing in code — `lba` is a plain number, and whatever
implements the trait is responsible for translating it into wherever
the data actually lives (for `disk_exploration`'s `FileDisk`, that's a
byte offset into a regular file — chapter 6 shows the exact line that
does this).

## Who defines the addresses: the drive, not the OS

Disks come with predefined logical addresses built in at the factory —
when a drive is manufactured, it's organized into fixed-size sectors,
each assigned an LBA number: 0, 1, 2, ... up to the drive's capacity.
That numbering lives in the drive's own firmware.

The OS's role is narrower than it might seem: it asks the drive "how
many logical blocks do you have," gets back a count, and builds a
**filesystem** (FAT32, NTFS, ext4, ...) on top of those LBAs — deciding
which sectors hold file metadata, which hold actual file contents, and
so on. Opening a file is this whole chain, end to end:

```
File name → Filesystem table → Sector numbers (LBAs) → Disk firmware → Physical spot
```

The three layers, kept distinct:

| Layer | What it is | Who defines it |
|---|---|---|
| Physical address | the real magnetic spot / flash cell | hardware, hidden from you |
| Logical address (LBA) | sector numbers 0, 1, 2, ... | the drive's own firmware, fixed at manufacture |
| Filesystem | files, directories, free space, built on top of LBAs | the OS, and it's the *only* one of the three that varies between machines |

Plug the same USB drive into ten different computers, and every one
sees the identical logical block addresses — because those numbers are
defined by the device itself, not by whichever OS happens to read it.
What differs between operating systems is the *filesystem* layered on
top, never the underlying sector numbering.

## Virtual disks work exactly the same way

A virtual disk (`.vhd`, `.vdi`, `.qcow2`, and friends) is just a large
regular file, which virtualization software (VirtualBox, VMware, QEMU)
presents to a guest OS as if it were real hardware. Inside that file,
the same LBA numbering scheme applies — the file is divided into
virtual sectors (typically 512 B or 4 KB, same as a physical disk),
numbered 0 through N−1. When the guest OS says "write to sector 12345,"
the hypervisor translates that into a byte offset inside the big file
on the *host's* real disk.

| Layer | Physical Disk | Virtual Disk |
|---|---|---|
| Physical spots | Magnetic platter / flash cells | Bytes inside a big file |
| Logical addresses (LBA) | Provided by disk firmware | Simulated by the hypervisor |
| Filesystem | NTFS, FAT32, ext4, ... | Same — the guest OS installs one inside the virtual disk |

This is also exactly what `disk_exploration`'s `disk.img` *is* —
chapter 6 shows a real Rust program creating one from scratch: a plain
file on your real disk, addressed sector by sector, that an OS (or a
GPT/FAT-aware tool) can treat as if it were a real drive.
