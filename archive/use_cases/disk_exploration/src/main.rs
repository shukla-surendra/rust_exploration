pub const SECTOR_SIZE: u64 = 512;

pub trait BlockDevice {
    fn size(&self) -> u64;                        // total bytes
    fn read_sector(&self, lba: u64, buf: &mut [u8]);
    fn write_sector(&mut self, lba: u64, buf: &[u8]);
}

mod gpt_fat;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;
use gpt_fat::{make_gpt_and_fat};

pub struct FileDisk {
    file: File,
    len: u64,
}

impl FileDisk {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
        if file.metadata()?.len() == 0 {
            file.set_len(1 * 1024 * 1024)?; // give it 1 MiB by default if empty
        }
        let len = file.metadata()?.len();
        Ok(Self { file, len })
    }
}

impl BlockDevice for FileDisk {
    fn size(&self) -> u64 { self.len }
    fn read_sector(&self, lba: u64, buf: &mut [u8]) {
        assert_eq!(buf.len(), SECTOR_SIZE as usize);
        let mut f = &self.file;
        let _ = f.seek(SeekFrom::Start(lba * SECTOR_SIZE));
        let _ = (&self.file).read_exact(buf);
    }
    fn write_sector(&mut self, lba: u64, buf: &[u8]) {
        assert_eq!(buf.len(), SECTOR_SIZE as usize);
        let _ = self.file.seek(SeekFrom::Start(lba * SECTOR_SIZE));
        let _ = self.file.write_all(buf);
        let _ = self.file.flush();
    }
}

fn main() -> anyhow::Result<()> {
    // Minimal smoke test on sector 0
    let path = Path::new("disk.img");
    let mut disk = FileDisk::open(path).expect("open disk.img");

    let mut boot = [0u8; SECTOR_SIZE as usize];
    boot[0..3].copy_from_slice(&[0xEB, 0x3C, 0x90]); // JMP + NOP
    boot[510] = 0x55; boot[511] = 0xAA;              // 0xAA55
    disk.write_sector(0, &boot);

    let mut readback = [0u8; SECTOR_SIZE as usize];
    disk.read_sector(0, &mut readback);
    assert_eq!(readback[510], 0x55);
    assert_eq!(readback[511], 0xAA);

    // Now create GPT, add partition, format FAT, and write HELLO.TXT
    make_gpt_and_fat()?;


    Ok(())
}
