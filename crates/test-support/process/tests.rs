use super::*;
use std::process::{Child, Command};

#[cfg(target_os = "linux")]
#[path = "tests/linux.rs"]
mod linux;

fn sleep() -> Child {
    Command::new("/bin/sleep").arg("60").spawn().unwrap()
}

#[test]
fn reports_running_as_alive_and_zombie_and_reaped_as_dead() {
    let mut child = sleep();
    let pid = child.id();
    let running = is_dead(pid);

    child.kill().unwrap();
    assert!(!running.unwrap());
    // Blocks until the child is exited and then leaves it unreaped, so the
    // next check observes the zombie `is_dead` reads, and no poll guesses
    // when the exit becomes visible.
    // POSIX waitid with WNOWAIT: block until the child is exited, then leave
    // it unreaped, so the next check observes the zombie `is_dead` reads.
    // No poll guesses when the exit becomes visible.
    // pid_t is u32 on some targets, i32 on others; the call takes it as-is.
    let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
    let result =
        unsafe { libc::waitid(libc::P_PID, pid, &mut info, libc::WNOWAIT | libc::WEXITED) };
    assert_eq!(result, 0, "killed child did not exit");
    assert!(is_dead(pid).unwrap());

    child.wait().unwrap();
    assert!(is_dead(pid).unwrap());
}
