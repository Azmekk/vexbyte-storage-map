use std::io;
use std::process::{Command, Stdio};

/// Starts `program` with `arguments` and no inherited input or output, without waiting for it to
/// finish.
pub(super) fn spawn_detached(program: &str, arguments: &[&std::ffi::OsStr]) -> io::Result<()> {
    let mut child = Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    // An exited child stays a zombie until it is waited on; a short-lived thread reaps it so the
    // caller does not have to block.
    let reaper = std::thread::Builder::new()
        .name("vsm-reap-child".to_owned())
        .spawn(move || child.wait());
    if let Err(error) = reaper {
        tracing::debug!(%error, program, "could not start a thread to reap the child process");
    }
    Ok(())
}
