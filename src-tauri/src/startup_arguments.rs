//! The command line arguments the app understands. Only the elevated relaunch passes any: it
//! hands the new instance the scan the user asked for, so that instance starts it right away.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use disk_scanner::{ScanMethod, ScanTarget};
use serde::Serialize;
use specta::Type;

const SCAN_VOLUME_ARGUMENT: &str = "--scan-volume";
const SCAN_FOLDER_ARGUMENT: &str = "--scan-folder";

/// A scan requested on the command line, for the frontend to start as soon as it loads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StartupScanRequest {
    pub target: ScanTarget,
    /// The relaunch exists to get the fast scan, so a requested scan always uses it.
    pub method: ScanMethod,
}

/// The arguments that make a new instance start a fast scan of `target`.
pub fn arguments_for_startup_scan(target: &ScanTarget) -> Vec<OsString> {
    let (flag, path) = match target {
        ScanTarget::Volume { mount_point } => (SCAN_VOLUME_ARGUMENT, mount_point),
        ScanTarget::Folder { path } => (SCAN_FOLDER_ARGUMENT, path),
    };

    vec![OsString::from(flag), path.clone().into_os_string()]
}

/// Finds a scan request in the arguments after the program name. Unknown arguments are ignored;
/// when several requests are given, the last one wins.
pub fn parse_startup_scan_request<Argument: AsRef<OsStr>>(
    arguments: impl IntoIterator<Item = Argument>,
) -> Option<StartupScanRequest> {
    let mut arguments = arguments.into_iter();
    let mut request = None;

    while let Some(argument) = arguments.next() {
        let argument = argument.as_ref();
        let target_for_path: fn(PathBuf) -> ScanTarget = if argument == SCAN_VOLUME_ARGUMENT {
            |mount_point| ScanTarget::Volume { mount_point }
        } else if argument == SCAN_FOLDER_ARGUMENT {
            |path| ScanTarget::Folder { path }
        } else {
            continue;
        };

        let Some(path) = arguments.next() else { break };
        let path = PathBuf::from(path.as_ref());
        if path.as_os_str().is_empty() {
            continue;
        }

        request = Some(StartupScanRequest {
            target: target_for_path(path),
            method: ScanMethod::MasterFileTable,
        });
    }

    request
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn volume_target(mount_point: &str) -> ScanTarget {
        ScanTarget::Volume {
            mount_point: PathBuf::from(mount_point),
        }
    }

    #[test]
    fn reads_a_volume_request() {
        let request = parse_startup_scan_request(["--scan-volume", r"C:\"]).unwrap();

        assert_eq!(request.target, volume_target(r"C:\"));
        assert_eq!(request.method, ScanMethod::MasterFileTable);
    }

    #[test]
    fn reads_a_folder_request_among_other_arguments() {
        let request =
            parse_startup_scan_request(["--unknown", "--scan-folder", r"D:\My Photos"]).unwrap();

        assert_eq!(
            request.target,
            ScanTarget::Folder {
                path: PathBuf::from(r"D:\My Photos")
            }
        );
    }

    #[test]
    fn ignores_missing_or_empty_paths() {
        assert_eq!(parse_startup_scan_request(["--scan-volume"]), None);
        assert_eq!(parse_startup_scan_request(["--scan-volume", ""]), None);
        assert_eq!(parse_startup_scan_request(Vec::<&str>::new()), None);
    }

    #[test]
    fn the_last_request_wins() {
        let request =
            parse_startup_scan_request(["--scan-volume", r"C:\", "--scan-volume", r"E:\"]).unwrap();

        assert_eq!(request.target, volume_target(r"E:\"));
    }

    #[test]
    fn round_trips_through_the_relaunch_arguments() {
        let targets = [
            volume_target(r"C:\"),
            ScanTarget::Folder {
                path: PathBuf::from(r"D:\Some Folder\"),
            },
        ];

        for target in targets {
            let arguments = arguments_for_startup_scan(&target);
            let request = parse_startup_scan_request(&arguments).unwrap();
            assert_eq!(request.target, target);
        }
    }
}
