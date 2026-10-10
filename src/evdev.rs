//! Linux input_event uses native C long timestamps, not a fixed 24-byte record.
//! ARMv7's kernel ABI is 16 bytes even when musl uses 64-bit userspace time_t.
use std::io;
use std::os::fd::RawFd;

pub const EVENT_SIZE: usize = 2 * std::mem::size_of::<libc::c_long>() + 8;

pub fn decode(bytes: &[u8]) -> io::Result<(u16, u16, i32)> {
    decode_at(bytes, EVENT_SIZE)
}

fn decode_at(bytes: &[u8], size: usize) -> io::Result<(u16, u16, i32)> {
    if !matches!(size, 16 | 24) || bytes.len() != size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid evdev record",
        ));
    }
    let i = size - 8;
    Ok((
        u16::from_ne_bytes(bytes[i..i + 2].try_into().unwrap()),
        u16::from_ne_bytes(bytes[i + 2..i + 4].try_into().unwrap()),
        i32::from_ne_bytes(bytes[i + 4..i + 8].try_into().unwrap()),
    ))
}

#[derive(Clone, Copy, Debug)]
pub struct Axis {
    pub min: i32,
    pub max: i32,
}

impl Axis {
    pub fn read(fd: RawFd, axis: u16) -> io::Result<Self> {
        // EVIOCGABS(axis): _IOR('E', 0x40 + axis, struct input_absinfo).
        let mut info = [0i32; 6];
        let request = 0x8018_4500u32 | (0x40 + u32::from(axis));
        if unsafe { libc::ioctl(fd, request as _, info.as_mut_ptr()) } < 0 {
            return Err(io::Error::last_os_error());
        }
        if info[2] <= info[1] {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "empty input axis range",
            ));
        }
        Ok(Self {
            min: info[1],
            max: info[2],
        })
    }

    pub fn scale(self, value: i32, extent: i32) -> i32 {
        let value = i64::from(value.clamp(self.min, self.max)) - i64::from(self.min);
        (value * i64::from(extent - 1) / (i64::from(self.max) - i64::from(self.min))) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decodes_both_kernel_abis_and_signed_values() {
        for size in [16, 24] {
            let mut b = vec![0xa5; size];
            b[size - 8..size - 6].copy_from_slice(&3u16.to_ne_bytes());
            b[size - 6..size - 4].copy_from_slice(&57u16.to_ne_bytes());
            b[size - 4..].copy_from_slice(&(-1i32).to_ne_bytes());
            assert_eq!(decode_at(&b, size).unwrap(), (3, 57, -1));
            assert!(decode_at(&b[..size - 1], size).is_err());
        }
    }
    #[test]
    fn scaling_clamps_and_honors_nonzero_minimum() {
        let a = Axis { min: 10, max: 110 };
        assert_eq!(a.scale(-100, 1404), 0);
        assert_eq!(a.scale(110, 1404), 1403);
        assert_eq!(a.scale(1000, 1404), 1403);
        assert_eq!(a.scale(60, 1404), 701);
    }
}
