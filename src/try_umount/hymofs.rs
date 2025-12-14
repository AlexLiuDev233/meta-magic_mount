use std::{
    ffi::{CString, c_char, c_int},
    fs::OpenOptions,
    io,
    os::fd::{AsRawFd, RawFd},
    path::Path,
    sync::OnceLock,
};

use anyhow::Result;
use nix::ioctl_write_ptr;
use rustix::path::Arg;

const HYMO_IOC_MAGIC: u32 = 0xE0;
ioctl_write_ptr!(hymofs_hide, HYMO_IOC_MAGIC, 3, HymoHide);
pub(super) const HYMO_DEV: &[&str] = &["/dev/hymo_ctl", "/proc/hymo_ctl"];
static DRIVER_FD: OnceLock<RawFd> = OnceLock::new();

#[repr(C)]
struct HymoHide {
    src: *const c_char,
    target: *const c_char,
    r#type: c_int,
}

pub(super) fn send_hide_hymofs<P>(target: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let fd = *DRIVER_FD.get_or_init(|| {
        let mut fd = -1;
        for i in HYMO_DEV {
            if let Ok(dev) = OpenOptions::new().read(true).write(true).open(i) {
                fd = dev.as_raw_fd();
            }
        }
        fd
    });

    let path = CString::new(target.as_ref().as_str()?)?;
    let cmd = HymoHide {
        src: path.as_ptr(),
        target: std::ptr::null(),
        r#type: 0,
    };

    let ret = unsafe { hymofs_hide(fd, &raw const cmd) }?;
    if ret < 0 {
        log::error!(
            "umount {} failed: {}",
            target.as_ref().display(),
            io::Error::last_os_error()
        );

        return Ok(());
    }

    log::info!("umount {} successful!", target.as_ref().display());
    Ok(())
}
