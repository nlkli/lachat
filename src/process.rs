use crate::Result;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

pub fn spawn_detached(cmd: &str, args: &[&str]) -> Result<u32> {
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

pub fn run_in_background<F>(job: F) -> Result<u32>
where
    F: FnOnce() + Send + 'static,
{
    unsafe {
        match libc::fork() {
            -1 => Result::Err(Box::new(io::Error::last_os_error())),

            0 => {
                if libc::setsid() == -1 {
                    libc::_exit(1);
                }

                job();

                libc::_exit(0);
            }

            pid => {
                Ok(pid as u32)
            }
        }
    }
}
