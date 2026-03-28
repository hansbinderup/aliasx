use anyhow::Context;
use nix::sys::select::{select, FdSet};
use nix::sys::signal::*;
use nix::sys::signalfd::{SfdFlags, SignalFd};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{setpgid, Pid};
use std::io::IsTerminal;
use std::os::fd::{AsFd, AsRawFd};
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::{Command, ExitStatus, Stdio};

pub struct Executor {
    command: String,
}

impl Executor {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }

    pub fn run(self) -> anyhow::Result<ExitStatus> {
        // Block SIGTSTP in parent so we can handle it via signalfd.
        // The child will unblock it in pre_exec so it suspends normally.
        let mut tstp_mask = SigSet::empty();
        tstp_mask.add(Signal::SIGTSTP);
        sigprocmask(SigmaskHow::SIG_BLOCK, Some(&tstp_mask), None)?;

        let sfd = SignalFd::with_flags(&tstp_mask, SfdFlags::SFD_CLOEXEC)?;

        let child = self.spawn(&tstp_mask)?;
        let child_pid = Pid::from_raw(child as i32);
        let child_pgid = child_pid;

        let stdin = std::io::stdin();
        let is_tty = stdin.is_terminal();
        let our_pgid = nix::unistd::getpgrp();

        ignore_signals()?;

        if is_tty {
            handoff_tty(stdin.as_fd(), child_pgid)
                .context("failed to hand TTY control to child")?;
        }

        let exit_status = self.event_loop(child_pid, child_pgid, our_pgid, &stdin, &sfd, is_tty)?;

        if is_tty {
            reclaim_tty(stdin.as_fd(), our_pgid).context("failed to reclaim TTY control")?;
        }

        restore_signals()?;
        sigprocmask(SigmaskHow::SIG_UNBLOCK, Some(&tstp_mask), None)?;

        Ok(exit_status)
    }

    fn spawn(&self, tstp_mask: &SigSet) -> anyhow::Result<u32> {
        let tstp_mask = *tstp_mask;
        let mut cmd = Command::new("sh");
        cmd.args(["-c", &self.command])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        unsafe {
            cmd.pre_exec(move || {
                // Own process group — Ctrl+C goes to child directly, not parent
                setpgid(Pid::from_raw(0), Pid::from_raw(0))
                    .map_err(|e| std::io::Error::from_raw_os_error(e as i32))?;
                // Unblock SIGTSTP so child can be suspended normally
                sigprocmask(SigmaskHow::SIG_UNBLOCK, Some(&tstp_mask), None)
                    .map_err(|e| std::io::Error::from_raw_os_error(e as i32))?;
                Ok(())
            });
        }

        let child = cmd.spawn().context("failed to spawn command")?;
        Ok(child.id())
    }

    fn event_loop(
        &self,
        child_pid: Pid,
        child_pgid: Pid,
        our_pgid: Pid,
        stdin: &std::io::Stdin,
        sfd: &SignalFd,
        is_tty: bool,
    ) -> anyhow::Result<ExitStatus> {
        let mut tstp_mask = SigSet::empty();
        tstp_mask.add(Signal::SIGTSTP);

        loop {
            // Check child state without blocking
            match waitpid(
                child_pid,
                Some(WaitPidFlag::WNOHANG | WaitPidFlag::WUNTRACED),
            )? {
                WaitStatus::Exited(_, code) => {
                    return Ok(ExitStatus::from_raw(code << 8));
                }
                WaitStatus::Signaled(_, sig, _) => {
                    return Ok(ExitStatus::from_raw(sig as i32));
                }
                WaitStatus::Stopped(_, _) => {
                    // Child was stopped via Ctrl+Z — suspend ourselves too
                    if is_tty {
                        reclaim_tty(stdin.as_fd(), our_pgid).ok();
                    }
                    restore_signals()?;
                    sigprocmask(SigmaskHow::SIG_UNBLOCK, Some(&tstp_mask), None)?;

                    // Suspend — shell takes over here until `fg`
                    nix::sys::signal::raise(Signal::SIGSTOP)?;

                    // Resumed via fg
                    sigprocmask(SigmaskHow::SIG_BLOCK, Some(&tstp_mask), None)?;
                    ignore_signals()?;
                    kill(child_pgid, Signal::SIGCONT)?;
                    if is_tty {
                        handoff_tty(stdin.as_fd(), child_pgid).ok();
                    }
                }
                _ => {
                    // Child still running — wait for SIGTSTP on signalfd
                    let sfd_fd = sfd.as_fd();
                    let mut fds = FdSet::new();
                    fds.insert(sfd_fd);
                    let mut tv = nix::sys::time::TimeVal::new(0, 50_000);
                    if select(
                        sfd_fd.as_raw_fd() + 1,
                        Some(&mut fds),
                        None,
                        None,
                        Some(&mut tv),
                    )? > 0
                    {
                        sfd.read_signal().ok();
                    }
                }
            }
        }
    }
}

fn ignore_signals() -> anyhow::Result<()> {
    let ign = SigAction::new(SigHandler::SigIgn, SaFlags::empty(), SigSet::empty());
    unsafe {
        sigaction(Signal::SIGINT, &ign)?;
        sigaction(Signal::SIGTTOU, &ign)?;
        sigaction(Signal::SIGTTIN, &ign)?;
    }
    Ok(())
}

fn restore_signals() -> anyhow::Result<()> {
    let dfl = SigAction::new(SigHandler::SigDfl, SaFlags::empty(), SigSet::empty());
    unsafe {
        sigaction(Signal::SIGINT, &dfl)?;
        sigaction(Signal::SIGTTOU, &dfl)?;
        sigaction(Signal::SIGTTIN, &dfl)?;
    }
    Ok(())
}

fn handoff_tty(fd: std::os::fd::BorrowedFd<'_>, pgid: Pid) -> anyhow::Result<()> {
    nix::unistd::tcsetpgrp(fd, pgid)?;
    Ok(())
}

fn reclaim_tty(fd: std::os::fd::BorrowedFd<'_>, pgid: Pid) -> anyhow::Result<()> {
    nix::unistd::tcsetpgrp(fd, pgid)?;
    Ok(())
}
