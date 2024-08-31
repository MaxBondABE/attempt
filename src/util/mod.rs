use std::process::Child;

use nix::{
    errno::Errno,
    sys::signal::{kill, Signal},
    unistd::Pid,
};

pub mod status;
pub mod value_parsing;
pub mod logger;
pub mod poll;
#[cfg(test)] // dev-dependencies
pub mod testing;

/// Send SIGKILL to a child process
pub fn force_kill(child: &Child) -> Result<(), Errno> {
    // Negative values have special meaning and kill many processes, so are unsafe to use.
    // See `man 2 kill`. If we simply cast a u32 to an i32, any value > i32::MAX would trigger
    // this behavior. This unlikely to ever happen, it would require unusual configuration or
    // exotic platforms.
    let id = i32::try_from(child.id()).expect("only positive values are safe for kill()");
    let pid = Pid::from_raw(id);
    kill(pid, Signal::SIGKILL)?;
    Ok(())
}
