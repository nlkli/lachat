use crate::Result;
use std::ffi::OsStr;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

pub fn spawn_detached<I, S>(cmd: &str, args: I) -> Result<u32> 
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(cmd);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }

    let child = command.spawn()?;
    Ok(child.id())
}

pub fn kill_pid(pid: u32) -> Result<()> {
    let res = unsafe { libc::kill(pid as i32, libc::SIGTERM) };

    if res != 0 {
        return Err(Box::new(io::Error::last_os_error()));
    }

    Ok(())
}

