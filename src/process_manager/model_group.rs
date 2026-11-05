//! The model engine must not outlive Kit, even if its stdin is held open or it is
//! wedged in native code. On Unix a monitor-only Kit process leads a private group.
//! The engine joins it and handshakes *before exec*. The monitor never executes
//! user input, scans a PID registry, or signals a PID supplied by its caller.

use super::ChildRegistration;
use std::io;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

pub(crate) struct ModelGroup {
    #[cfg(unix)]
    monitor: Option<unix::Monitor>,
    // Drop only after stopping the group; the registry retains any unverified survivor.
    _registration: ChildRegistration,
}

impl ModelGroup {
    pub(crate) fn spawn(command: Command) -> io::Result<(Child, Self)> {
        Self::launch(command, |mut c| {
            c.spawn().map(|p| {
                let id = p.id();
                (p, Some(id))
            })
        })
    }

    pub(crate) fn spawn_async(command: Command) -> io::Result<(tokio::process::Child, Self)> {
        Self::launch(command, |c| {
            tokio::process::Command::from(c)
                .kill_on_drop(true)
                .spawn()
                .map(|p| {
                    let id = p.id();
                    (p, id)
                })
        })
    }

    fn launch<C>(
        command: Command,
        spawn: impl FnOnce(Command) -> io::Result<(C, Option<u32>)>,
    ) -> io::Result<(C, Self)> {
        use crate::runtime_policy::{check, ExternalEffect};
        for effect in [ExternalEffect::Process, ExternalEffect::Provider] {
            check(effect).map_err(io::Error::other)?;
        }
        let program = command.get_program().to_owned();
        #[cfg(unix)]
        let (child, group) = {
            let mut command = command;
            let monitor = unix::Monitor::prepare(&program)?;
            // No exec is allowed until the child is in this specific group and its monitor is armed.
            monitor.bind(&mut command);
            let (child, model_pid) = spawn(command)?;
            let registration =
                ChildRegistration::for_group(monitor.pid(), &program.to_string_lossy(), model_pid);
            (
                child,
                Self {
                    monitor: Some(monitor),
                    _registration: registration,
                },
            )
        };
        #[cfg(not(unix))]
        let (child, group) = {
            let (child, pid) = spawn(command)?;
            let pid = pid.ok_or_else(|| io::Error::other("model process has no id"))?;
            (
                child,
                Self {
                    _registration: ChildRegistration::register(pid, &program.to_string_lossy()),
                },
            )
        };
        Ok((child, group))
    }

    /// The caller has closed stdin. Give a cooperative helper time to exit on EOF,
    /// but never block Kit on writing a shutdown message or waiting for a wedged model.
    pub(crate) fn finish_after_eof(&mut self, mut child: Child) {
        let finished = wait_briefly(&mut child, Duration::from_millis(150));
        #[cfg(unix)]
        drop(self.monitor.take());
        if !finished {
            // Unix stops the owned group through the private lease, not a cached PID.
            #[cfg(not(unix))]
            let _ = child.kill();
            reap(child);
        }
    }
}

fn wait_briefly(child: &mut Child, grace: Duration) -> bool {
    let deadline = Instant::now() + grace;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(2)),
            _ => return false,
        }
    }
}
fn reap(mut child: Child) {
    if !wait_briefly(&mut child, Duration::from_millis(80)) {
        // The kill was requested. Reap the original direct child, never a later PID.
        if std::thread::Builder::new()
            .name("kit-model-reaper".into())
            .spawn(move || {
                let _ = child.wait();
            })
            .is_err()
        {
            tracing::warn!("model_child_reaping_unverified");
        }
    }
}

#[cfg(unix)]
mod unix {
    use std::ffi::{OsStr, OsString};
    use std::io::{self, Read, Write};
    use std::net::Shutdown;
    use std::os::fd::{AsFd, AsRawFd, OwnedFd};
    use std::os::unix::net::UnixStream;
    use std::os::unix::process::CommandExt;
    use std::process::{Child, Command, Stdio};
    use std::time::Duration;

    const FLAG: &str = "--watch-model-process-group";
    const ARM: u8 = 0x4b;
    const ACK: u8 = 0x47;
    const HANDSHAKE: Duration = Duration::from_secs(8);

    pub(super) struct Monitor {
        child: Option<Child>,
        control: UnixStream,
        pid: i32,
    }
    impl Monitor {
        pub(super) fn prepare(label: &OsStr) -> io::Result<Self> {
            let (control, input) = UnixStream::pair()?;
            control.set_read_timeout(Some(HANDSHAKE))?;
            control.set_write_timeout(Some(HANDSHAKE))?;
            let mut cmd = Command::new(std::env::current_exe()?);
            cmd.env_clear()
                .current_dir("/")
                .process_group(0)
                .stdin(Stdio::from(OwnedFd::from(input)))
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            let parent = std::process::id().to_string();
            #[cfg(not(test))]
            cmd.arg(FLAG).arg(parent).arg(label);
            #[cfg(test)]
            {
                let _ = label;
                cmd.args([
                    "--exact",
                    "process_manager::model_group::unix::tests::monitor_entry",
                    "--test-threads=1",
                ])
                .env("KIT_TEST_MONITOR_PARENT", parent);
            }
            let child = cmd.spawn()?;
            Ok(Self {
                pid: child.id() as i32,
                child: Some(child),
                control,
            })
        }
        pub(super) fn pid(&self) -> u32 {
            self.pid as u32
        }
        pub(super) fn bind(&self, cmd: &mut Command) {
            let group = self.pid;
            let owner = std::process::id() as i32;
            let fd = self.control.as_raw_fd();
            cmd.process_group(group);
            // SAFETY: only async-signal-safe syscalls and stack bytes after fork. launch()
            // consumes the Command before the Monitor (and its CLOEXEC descriptor) can drop.
            unsafe {
                cmd.pre_exec(move || {
                    let mut reply = 0u8;
                    let mut arm = [ARM, 0, 0, 0, 0];
                    arm[1..].copy_from_slice(&libc::getpid().to_be_bytes());
                    if libc::getpgrp() != group
                        || libc::getppid() != owner
                        || libc::send(fd, arm.as_ptr().cast(), arm.len(), libc::MSG_NOSIGNAL)
                            != arm.len() as isize
                        || libc::read(fd, (&mut reply as *mut u8).cast(), 1) != 1
                        || reply != ACK
                    {
                        return Err(io::Error::from_raw_os_error(libc::ECANCELED));
                    }
                    Ok(())
                });
            }
        }
    }
    impl Drop for Monitor {
        fn drop(&mut self) {
            let _ = self.control.shutdown(Shutdown::Both);
            if let Some(child) = self.child.take() {
                // Only the monitor signals its *own* group. Don't issue a second signal
                // from Kit against a numerical ID after the monitor may have exited.
                super::reap(child);
            }
        }
    }

    pub(crate) fn entry(
        first: Option<OsString>,
        mut rest: impl Iterator<Item = OsString>,
    ) -> Option<i32> {
        if first.as_deref() != Some(OsStr::new(FLAG)) {
            return None;
        }
        let parent = rest.next().and_then(|s| s.to_str()?.parse::<i32>().ok());
        // The inert label is solely for the existing Process Manager identity/display.
        // It is never used as a path or a kill target. A malformed invocation never starts Kit.
        match (parent, rest.next(), rest.next()) {
            (Some(p), Some(label), None) if !label.is_empty() => {
                Some(if watch(p).is_ok() { 0 } else { 70 })
            }
            _ => Some(64),
        }
    }
    fn watch(parent: i32) -> io::Result<()> {
        let me = std::process::id() as i32;
        if parent <= 1
            || me <= 1
            || unsafe { libc::getpgrp() } != me
            || unsafe { libc::getppid() } != parent
        {
            return Err(io::Error::from_raw_os_error(libc::EPERM));
        }
        let mut socket = UnixStream::from(std::io::stdin().as_fd().try_clone_to_owned()?);
        socket.peer_addr()?;
        socket.set_read_timeout(Some(HANDSHAKE))?;
        // A graceful stop of the entire group must reach the engine but must not disarm
        // its monitor early. Nothing here changes the engine's own signal dispositions.
        for signal in [libc::SIGTERM, libc::SIGHUP, libc::SIGINT] {
            if unsafe { libc::signal(signal, libc::SIG_IGN) } == libc::SIG_ERR {
                return Err(io::Error::last_os_error());
            }
        }
        let mut arm = [0; 5];
        socket.read_exact(&mut arm)?;
        let payload = i32::from_be_bytes([arm[1], arm[2], arm[3], arm[4]]);
        if arm[0] != ARM
            || payload <= 1
            || payload == me
            || unsafe { libc::getpgid(payload) } != me
            || unsafe { libc::getppid() } != parent
        {
            return Err(io::Error::from_raw_os_error(libc::ECANCELED));
        }
        #[cfg(target_os = "macos")]
        let events = register_exit(parent, payload, socket.as_raw_fd())?;
        socket.write_all(&[ACK])?;
        // From ACK onward there is a payload in our group. Any observation error also
        // closes THIS group. We don't depend on stdin of the model, paths on a mounted
        // volume, Kit destructors, or another launch to clean it up.
        #[cfg(target_os = "macos")]
        loop {
            let mut event = std::mem::MaybeUninit::<libc::kevent>::uninit();
            let rc = unsafe {
                libc::kevent(
                    events.as_raw_fd(),
                    std::ptr::null(),
                    0,
                    event.as_mut_ptr(),
                    1,
                    std::ptr::null(),
                )
            };
            if rc >= 0 || io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
                break;
            }
        }
        #[cfg(not(target_os = "macos"))]
        while unsafe { libc::getppid() } == parent && unsafe { libc::getpgid(payload) } == me {
            let mut pfd = libc::pollfd {
                fd: socket.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            let rc = unsafe { libc::poll(&mut pfd, 1, 1000) };
            if rc > 0 || (rc < 0 && io::Error::last_os_error().kind() != io::ErrorKind::Interrupted)
            {
                break;
            }
        }
        // The monitor is the group leader and has never left its group. It accepts no arbitrary PID.
        if unsafe { libc::getpgrp() } == me {
            unsafe {
                libc::killpg(me, libc::SIGKILL);
            }
        }
        Ok(())
    }
    #[cfg(target_os = "macos")]
    fn register_exit(parent: i32, payload: i32, fd: i32) -> io::Result<OwnedFd> {
        use std::os::fd::FromRawFd;
        let raw = unsafe { libc::kqueue() };
        if raw < 0 {
            return Err(io::Error::last_os_error());
        }
        let queue = unsafe { OwnedFd::from_raw_fd(raw) };
        let watch = |ident, filter, fflags| libc::kevent {
            ident,
            filter,
            flags: libc::EV_ADD | libc::EV_ONESHOT,
            fflags,
            data: 0,
            udata: std::ptr::null_mut(),
        };
        let changes = [
            watch(parent as usize, libc::EVFILT_PROC, libc::NOTE_EXIT),
            watch(payload as usize, libc::EVFILT_PROC, libc::NOTE_EXIT),
            watch(fd as usize, libc::EVFILT_READ, 0),
        ];
        let rc = unsafe {
            libc::kevent(
                raw,
                changes.as_ptr(),
                changes.len() as i32,
                std::ptr::null_mut(),
                0,
                std::ptr::null(),
            )
        };
        if rc < 0 {
            return Err(io::Error::last_os_error());
        }
        if unsafe { libc::getppid() } != parent {
            return Err(io::Error::from_raw_os_error(libc::ECANCELED));
        }
        Ok(queue)
    }
    #[cfg(test)]
    mod tests {
        #[test]
        fn monitor_entry() {
            if let Ok(parent) = std::env::var("KIT_TEST_MONITOR_PARENT") {
                // The same monitor implementation runs inside a re-executed libtest binary.
                // Callers running the ordinary suite do not become a monitor.
                super::watch(parent.parse().unwrap()).unwrap();
            }
        }
    }
}

#[cfg(unix)]
/// Handle Kit’s private monitor entry before app startup, without reading user configuration.
pub fn run_if_requested() -> Option<i32> {
    let mut args = std::env::args_os().skip(1);
    unix::entry(args.next(), args)
}
