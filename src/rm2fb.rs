//! Native client for ddvk/remarkable2-framebuffer. No LD_PRELOAD client shim.
//! IPC definitions checked against upstream 3ce4f8109a146edf5602ac1e434f052659756441.
use crate::fb::{SCREEN_H, SCREEN_W};
use crate::surface::{PixFmt, Surface};
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io;
use std::os::fd::AsRawFd;
use std::time::{Duration, Instant};

const QUEUE_KEY: libc::key_t = 0x2257c;
const BUFFER_BYTES: usize = SCREEN_W * SCREEN_H * 2;

// mxcfb_update_data: 18 32-bit fields, including the unused alternate buffer.
// Keep the System V message type a native C long on both host and ARMv7.
#[repr(C)]
struct Update {
    kind: libc::c_long,
    fields: [u32; 18],
}
#[repr(C)]
struct Wait {
    kind: libc::c_long,
    name: [u8; 512],
}

pub struct Rm2fb {
    queue: i32,
    ptr: *mut u8,
    failure: RefCell<Option<io::Error>>,
}

impl Rm2fb {
    pub fn open() -> io::Result<(Self, Surface)> {
        if std::fs::read_to_string("/sys/devices/soc0/machine")?.trim() != "reMarkable 2.0" {
            return Err(io::Error::other("rm2 build requires a reMarkable 2"));
        }
        // Never create a queue or resize the server's shared buffer.
        let queue = unsafe { libc::msgget(QUEUE_KEY, 0) };
        if queue < 0 {
            return Err(io::Error::last_os_error());
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/shm/swtfb.01")?;
        if file.metadata()?.len() != BUFFER_BYTES as u64 {
            return Err(io::Error::other("rm2fb framebuffer has unexpected size"));
        }
        // An old queue/file can survive a crash; require a live server reply.
        ping(queue)?;
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                BUFFER_BYTES,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                file.as_raw_fd(),
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            return Err(io::Error::last_os_error());
        }
        let client = Self {
            queue,
            ptr: ptr.cast(),
            failure: RefCell::new(None),
        };
        let surface = Surface::new(
            client.ptr,
            BUFFER_BYTES,
            SCREEN_W,
            SCREEN_H,
            SCREEN_W * 2,
            PixFmt::Rgb565,
        );
        Ok((client, surface))
    }

    pub fn update(&self, x: i32, y: i32, w: i32, h: i32, fast: bool, full: bool) {
        if self.failure.borrow().is_some() {
            return;
        }
        if let Some(msg) = update_message(x, y, w, h, fast, full) {
            if let Err(e) = send(self.queue, &msg, 72) {
                *self.failure.borrow_mut() = Some(e);
            }
        }
    }

    pub fn check(&self) -> io::Result<()> {
        if let Some(e) = self.failure.borrow_mut().take() {
            Err(e)
        } else {
            Ok(())
        }
    }
}

impl Drop for Rm2fb {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr.cast(), BUFFER_BYTES);
        }
    }
}

fn update_message(x: i32, y: i32, w: i32, h: i32, fast: bool, full: bool) -> Option<Update> {
    if w <= 0 || h <= 0 {
        return None;
    }
    let left = x.clamp(0, SCREEN_W as i32);
    let top = y.clamp(0, SCREEN_H as i32);
    let right = x.saturating_add(w).clamp(0, SCREEN_W as i32);
    let bottom = y.saturating_add(h).clamp(0, SCREEN_H as i32);
    if right <= left || bottom <= top {
        return None;
    }
    let mut fields = [0; 18];
    fields[..4].copy_from_slice(&[
        top as u32,
        left as u32,
        (right - left) as u32,
        (bottom - top) as u32,
    ]);
    fields[4] = if full {
        2
    } else if fast {
        1
    } else {
        3
    }; // GC16 / DU / GL16
    fields[5] = u32::from(full);
    fields[7] = 0x1000; // ambient temperature
    Some(Update { kind: 2, fields })
}

fn send<T>(queue: i32, msg: &T, payload_len: usize) -> io::Result<()> {
    let deadline = Instant::now() + Duration::from_millis(250);
    loop {
        let rc = unsafe {
            libc::msgsnd(
                queue,
                (msg as *const T).cast(),
                payload_len,
                libc::IPC_NOWAIT,
            )
        };
        if rc == 0 {
            return Ok(());
        }
        let e = io::Error::last_os_error();
        if !matches!(e.raw_os_error(), Some(libc::EAGAIN | libc::EINTR)) {
            return Err(e);
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "rm2fb queue stalled",
            ));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn ping(queue: i32) -> io::Result<()> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let name =
        std::ffi::CString::new(format!("/riddle-rm2-{}-{stamp}", std::process::id())).unwrap();
    let sem = unsafe {
        libc::sem_open(
            name.as_ptr(),
            libc::O_CREAT | libc::O_EXCL,
            0o600 as libc::mode_t,
            0u32,
        )
    };
    if sem == libc::SEM_FAILED {
        return Err(io::Error::last_os_error());
    }
    let result = (|| {
        let mut msg = Wait {
            kind: 4,
            name: [0; 512],
        };
        msg.name[..name.as_bytes_with_nul().len()].copy_from_slice(name.as_bytes_with_nul());
        send(queue, &msg, 512)?;
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if unsafe { libc::sem_trywait(sem) } == 0 {
                return Ok(());
            }
            let e = io::Error::last_os_error();
            if !matches!(e.raw_os_error(), Some(libc::EAGAIN | libc::EINTR)) {
                return Err(e);
            }
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "rm2fb server did not acknowledge probe",
                ));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    })();
    unsafe {
        libc::sem_close(sem);
        libc::sem_unlink(name.as_ptr());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clips_updates_and_selects_waveforms() {
        let m = update_message(-10, -20, 30, 40, true, false).unwrap();
        assert_eq!(&m.fields[..6], &[0, 0, 20, 20, 1, 0]);
        assert!(update_message(1404, 0, 10, 10, true, false).is_none());
        assert!(update_message(0, 0, -1, 10, true, false).is_none());
        let m = update_message(0, 0, i32::MAX, i32::MAX, false, true).unwrap();
        assert_eq!(&m.fields[..6], &[0, 0, 1404, 1872, 2, 1]);
    }
    #[test]
    fn real_message_queue_receives_exact_server_payload() {
        let q = unsafe { libc::msgget(libc::IPC_PRIVATE, libc::IPC_CREAT | 0o600) };
        assert!(q >= 0);
        struct Guard(i32);
        impl Drop for Guard {
            fn drop(&mut self) {
                unsafe {
                    libc::msgctl(self.0, libc::IPC_RMID, std::ptr::null_mut());
                }
            }
        }
        let _guard = Guard(q);
        let m = update_message(5, 7, 9, 11, false, true).unwrap();
        send(q, &m, 72).unwrap();
        let mut received = Update {
            kind: 0,
            fields: [0; 18],
        };
        let n = unsafe {
            libc::msgrcv(
                q,
                (&mut received as *mut Update).cast(),
                72,
                0,
                libc::IPC_NOWAIT,
            )
        };
        assert_eq!(n, 72);
        assert_eq!(received.kind, 2);
        assert_eq!(&received.fields[..6], &[7, 5, 9, 11, 2, 1]);
        assert_eq!(
            std::mem::offset_of!(Update, fields),
            std::mem::size_of::<libc::c_long>()
        );
        // Fill a private queue to exercise the bounded failure path, not the
        // tablet's real queue. An absent/stalled server must not freeze input.
        loop {
            let rc = unsafe { libc::msgsnd(q, (&m as *const Update).cast(), 72, libc::IPC_NOWAIT) };
            if rc < 0 {
                break;
            }
        }
        assert_eq!(
            io::Error::last_os_error().raw_os_error(),
            Some(libc::EAGAIN)
        );
        let started = Instant::now();
        assert_eq!(send(q, &m, 72).unwrap_err().kind(), io::ErrorKind::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(2));
    }
}
