use anyhow::{Context, Result};
use fatfs::{FileSystem, FsOptions};
use gpt::{disk::LogicalBlockSize, mbr::ProtectiveMBR, partition_types, GptConfig};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

/// Simple view over a region of a file, constrained to [start, start+len).
struct PartitionSlice<F> {
    f: F,
    start: u64,
    len: u64,
    pos: u64, // position within the slice
}

impl<F: Read + Write + Seek> PartitionSlice<F> {
    fn new(mut f: F, start: u64, len: u64) -> Result<Self> {
        f.seek(SeekFrom::Start(start))?;
        Ok(Self { f, start, len, pos: 0 })
    }
}

impl<F: Read + Write + Seek> Read for PartitionSlice<F> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let remaining = (self.len - self.pos) as usize;
        if remaining == 0 {
            return Ok(0);
        }
        let to_read = remaining.min(buf.len());
        let n = self.f.read(&mut buf[..to_read])?;
        self.pos += n as u64;
        Ok(n)
    }
}

impl<F: Read + Write + Seek> Write for PartitionSlice<F> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let remaining = (self.len - self.pos) as usize;
        if remaining == 0 {
            return Ok(0);
        }
        let to_write = remaining.min(buf.len());
        let n = self.f.write(&buf[..to_write])?;
        self.pos += n as u64;
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.f.flush()
    }
}

impl<F: Read + Write + Seek> Seek for PartitionSlice<F> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(o) => o,
            SeekFrom::End(o) => {
                if o >= 0 {
                    self.len.saturating_add(o as u64)
                } else {
                    self.len.saturating_sub((-o) as u64)
                }
            }
            SeekFrom::Current(o) => {
                if o >= 0 {
                    self.pos.saturating_add(o as u64)
                } else {
                    self.pos.saturating_sub((-o) as u64)
                }
            }
        };
        let clamped = new_pos.min(self.len);
        self.f.seek(SeekFrom::Start(self.start + clamped))?;
        self.pos = clamped;
        Ok(self.pos)
    }
}

pub fn make_gpt_and_fat() -> Result<()> {
    // 1) Open (or create) the disk image
    let block_size = 512u64;
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("disk.img")
        .context("open disk.img")?;

    // Ensure the image has a size if newly created (e.g., 64 MiB)
    let size_bytes = f.seek(SeekFrom::End(0))?;
    if size_bytes == 0 {
        f.set_len(64 * 1024 * 1024)?; // 64 MiB
    }
    let size_bytes = f.seek(SeekFrom::End(0))?;
    let num_blocks = size_bytes / block_size;

    // 2) Protective MBR at LBA0 (fresh disks need this before GPT)
    let pmbr = ProtectiveMBR::with_lb_size(
        u32::try_from(num_blocks.saturating_sub(1)).unwrap_or(0xFFFF_FFFF),
    );
    f.seek(SeekFrom::Start(0))?;
    pmbr.overwrite_lba0(&mut f).context("write protective MBR")?;

    // 3) Create a new GPT
    let mut gdisk = GptConfig::default()
        .writable(true)
        .logical_block_size(LogicalBlockSize::Lb512)
        .create_from_device(f, None)
        .context("create GPT from device")?;

    // Add one partition using most of the disk.
    // gpt reserves 34 LBAs at head and 33 at tail; leave a small safety margin too.
    let safety_tail = 2048u64;
    let usable_lbas = num_blocks
        .saturating_sub(34) // head
        .saturating_sub(33) // tail
        .saturating_sub(safety_tail);
    let part_size_lbas = usable_lbas.max(1024);
    gdisk
        .add_partition("oxide", part_size_lbas, partition_types::BASIC, 0, None)
        .context("add partition")?;

    // Write GPT and get file back
    let mut f = gdisk.write().context("write GPT back to device")?;

    // 4) Reopen GPT to query the actual partition LBAs
    let gdisk = GptConfig::new()
        .open_from_device(&mut f)
        .context("reopen GPT")?;
    let (_idx, p) = gdisk
        .partitions()
        .iter()
        .find(|(_, p)| p.name == "oxide")
        .context("oxide partition not found")?;


    let first_lba = p.first_lba;
    let last_lba = p.last_lba;
    let part_start = first_lba * block_size;
    let part_len = (last_lba - first_lba + 1) * block_size;

    // 5) Format FAT and write HELLO.TXT
    {
        let ps = PartitionSlice::new(&mut f, part_start, part_len)?;
        fatfs::format_volume(ps, fatfs::FormatVolumeOptions::new())
            .context("format FAT volume")?;
    }
    {
        let ps = PartitionSlice::new(&mut f, part_start, part_len)?;
        let fs = FileSystem::new(ps, FsOptions::new()).context("mount FAT")?;
        let root = fs.root_dir();
        let mut file = root.create_file("HELLO.TXT")?;
        file.write_all(b"Hello from Oxide!\n")?;
    }

    Ok(())
}

