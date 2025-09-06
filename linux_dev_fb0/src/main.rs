// WARNING: conceptual only. Real code needs safe ioctl wrappers and
// parsing fb_var_screeninfo/fb_fix_screeninfo structures.

use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::slice;
use memmap2::MmapMut;

fn main() -> anyhow::Result<()> {
    let fb = OpenOptions::new().read(true).write(true).open("/dev/fb0")?;
    let fd = fb.as_raw_fd();

    // You must call ioctl to get screen info here (fb_var_screeninfo).
    // Suppose we know: width=800, height=600, bpp=32, line_length=3200
    let width = 800usize;
    let height = 600usize;
    let bpp = 32usize;
    let line_length = width * 4;

    let size = line_length * height;
    let mut mmap = unsafe { MmapMut::map_mut(&fb)? };

    // write a red rectangle
    for y in 100..200 {
        for x in 100..300 {
            let offset = y * line_length + x * 4;
            // endianness and format matters (assume ARGB or BGRA)
            mmap[offset] = 0x00;     // blue
            mmap[offset + 1] = 0x00; // green
            mmap[offset + 2] = 0xFF; // red
            mmap[offset + 3] = 0x00; // alpha / padding
        }
    }
    mmap.flush()?;
    Ok(())
}
