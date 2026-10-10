//! Raw evdev pen input: the full digitizer, bypassing Qt's filtered view.
//! Gives us 0-4096 pressure, tilt, hover, and the eraser tip (BTN_TOOL_RUBBER),
//! at the hardware event rate.
//!
//! The device is grabbed (EVIOCGRAB) while the diary is open so xochitl
//! doesn't also react to the pen; released automatically on close/exit.

use std::io;
use std::os::fd::RawFd;

use crate::fb::{SCREEN_H, SCREEN_W};

// Digitizer axis ranges on the Paper Pro ("Elan marker input").
pub const MAX_PRESSURE: i32 = 4096;

const EV_SYN: u16 = 0;
const EV_KEY: u16 = 1;
const EV_ABS: u16 = 3;
const SYN_REPORT: u16 = 0;
const ABS_X: u16 = 0;
const ABS_Y: u16 = 1;
const ABS_PRESSURE: u16 = 24;
const BTN_TOOL_PEN: u16 = 320;
const BTN_TOOL_RUBBER: u16 = 321;
const BTN_TOUCH: u16 = 330;

const EVIOCGRAB: libc::c_ulong = 0x40044590;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Pen,
    Eraser,
}

#[derive(Debug, Clone, Copy)]
pub struct PenSample {
    /// Screen coordinates.
    pub x: i32,
    pub y: i32,
    /// 0..4096
    pub pressure: i32,
    pub tool: Tool,
    pub touching: bool,
    /// True from tool-in-range until the pen leaves the digitizer.
    pub proximity: bool,
}

pub struct PenDevice {
    fd: RawFd,
    x_axis: crate::evdev::Axis,
    y_axis: crate::evdev::Axis,
    pressure_axis: crate::evdev::Axis,
    // Accumulated state between SYN_REPORTs.
    raw_x: i32,
    raw_y: i32,
    pressure: i32,
    tool: Tool,
    touching: bool,
    pen_in_range: bool,
    rubber_in_range: bool,
    proximity: bool,
    dirty: bool,
}

impl PenDevice {
    /// Find and grab the marker input device.
    pub fn open() -> io::Result<Self> {
        let path = find_marker_device()?;
        let cpath = std::ffi::CString::new(path.clone()).unwrap();
        let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_RDONLY | libc::O_NONBLOCK) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let axes = (|| {
            Ok::<_, io::Error>((crate::evdev::Axis::read(fd, ABS_X)?, crate::evdev::Axis::read(fd, ABS_Y)?, crate::evdev::Axis::read(fd, ABS_PRESSURE)?))
        })();
        let (x_axis, y_axis, pressure_axis) = match axes {
            Ok(axes) => axes,
            Err(e) => { unsafe { libc::close(fd); } return Err(e); }
        };
        let grab = unsafe { libc::ioctl(fd, EVIOCGRAB as _, 1i32) };
        if grab != 0 && cfg!(feature = "rm2") {
            let e = io::Error::last_os_error();
            unsafe { libc::close(fd); }
            return Err(e);
        }
        if grab != 0 {
            eprintln!(
                "riddle: warning: EVIOCGRAB failed ({}) — xochitl will also see the pen",
                io::Error::last_os_error()
            );
        }
        eprintln!("riddle: pen device {path} opened (grabbed: {})", grab == 0);
        Ok(Self {
            fd,
            x_axis, y_axis, pressure_axis,
            raw_x: 0,
            raw_y: 0,
            pressure: 0,
            tool: Tool::Pen,
            touching: false,
            pen_in_range: false,
            rubber_in_range: false,
            proximity: false,
            dirty: false,
        })
    }

    fn screen_point(&self) -> (i32, i32) {
        map_point(self.raw_x, self.raw_y, self.x_axis, self.y_axis, cfg!(feature = "rm2"))
    }

    pub fn raw_fd(&self) -> RawFd {
        self.fd
    }

    /// Drain all pending events; returns one sample per SYN_REPORT frame
    /// that changed state.
    pub fn drain(&mut self) -> Vec<PenSample> {
        let mut out = Vec::new();
        // Kernel input_event is 16 bytes on ARMv7 and 24 on aarch64.
        let mut buf = [0u8; crate::evdev::EVENT_SIZE * 64];
        loop {
            let n =
                unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if n <= 0 {
                break;
            }
            for chunk in buf[..n as usize].chunks_exact(crate::evdev::EVENT_SIZE) {
                let (etype, code, value) = crate::evdev::decode(chunk).expect("complete input event");
                match (etype, code) {
                    (EV_ABS, ABS_X) => {
                        self.raw_x = value;
                        self.dirty = true;
                    }
                    (EV_ABS, ABS_Y) => {
                        self.raw_y = value;
                        self.dirty = true;
                    }
                    (EV_ABS, ABS_PRESSURE) => {
                        self.pressure = value;
                        self.dirty = true;
                    }
                    (EV_KEY, BTN_TOOL_PEN) => {
                        self.pen_in_range = value == 1;
                        if self.pen_in_range {
                            self.tool = Tool::Pen;
                        }
                        self.proximity = self.pen_in_range || self.rubber_in_range;
                        self.dirty = true;
                    }
                    (EV_KEY, BTN_TOOL_RUBBER) => {
                        self.rubber_in_range = value == 1;
                        if self.rubber_in_range {
                            self.tool = Tool::Eraser;
                        }
                        self.proximity = self.pen_in_range || self.rubber_in_range;
                        self.dirty = true;
                    }
                    (EV_KEY, BTN_TOUCH) => {
                        self.touching = value == 1;
                        self.dirty = true;
                    }
                    (EV_SYN, SYN_REPORT) => {
                        if self.dirty {
                            self.dirty = false;
                            out.push(PenSample {
                                x: self.screen_point().0,
                                y: self.screen_point().1,
                                pressure: self.pressure_axis.scale(self.pressure, MAX_PRESSURE + 1),
                                tool: self.tool,
                                touching: self.touching,
                                proximity: self.proximity,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
        out
    }
}

impl Drop for PenDevice {
    fn drop(&mut self) {
        unsafe {
            libc::ioctl(self.fd, EVIOCGRAB as _, 0i32);
            libc::close(self.fd);
        }
    }
}

fn find_marker_device() -> io::Result<String> {
    for i in 0..8 {
        let name_path = format!("/sys/class/input/event{i}/device/name");
        if let Ok(name) = std::fs::read_to_string(&name_path) {
            if name.to_lowercase().contains("marker") || name.to_lowercase().contains("wacom") {
                return Ok(format!("/dev/input/event{i}"));
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "no marker input device found",
    ))
}

fn map_point(x: i32, y: i32, xa: crate::evdev::Axis, ya: crate::evdev::Axis, rm2: bool) -> (i32, i32) {
    if rm2 {
        // Wacom is mounted at 270 degrees: raw Y -> screen X; inverted raw X -> screen Y.
        (ya.scale(y, SCREEN_W as i32), SCREEN_H as i32 - 1 - xa.scale(x, SCREEN_H as i32))
    } else { (xa.scale(x, SCREEN_W as i32), ya.scale(y, SCREEN_H as i32)) }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wacom_corners_map_to_portrait_without_off_by_one() {
        let x = crate::evdev::Axis { min: 0, max: 20967 };
        let y = crate::evdev::Axis { min: 0, max: 15725 };
        assert_eq!(map_point(20967, 0, x, y, true), (0, 0));
        assert_eq!(map_point(0, 15725, x, y, true), (SCREEN_W as i32-1, SCREEN_H as i32-1));
        assert_eq!(map_point(0, 0, x, y, false), (0, 0));
        assert_eq!(map_point(20967, 15725, x, y, false), (SCREEN_W as i32-1, SCREEN_H as i32-1));
    }
}
