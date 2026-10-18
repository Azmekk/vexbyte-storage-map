/// True when the process runs as root.
pub(crate) fn is_process_elevated() -> bool {
    rustix::process::geteuid().is_root()
}
