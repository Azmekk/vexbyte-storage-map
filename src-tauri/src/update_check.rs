//! Asks GitHub for the newest release so the UI can offer a download. The app never downloads or
//! installs anything itself.

use std::cmp::Ordering;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use specta::Type;

const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/Azmekk/vexbyte-storage-map/releases/latest";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const USER_AGENT: &str = concat!("VexbyteStorageMap/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, thiserror::Error)]
pub enum UpdateCheckError {
    #[error("could not read the latest release from GitHub")]
    RequestFailed {
        #[source]
        source: Box<ureq::Error>,
    },
    #[error("GitHub reported a release version that is not a version number: {tag_name}")]
    UnreadableReleaseVersion { tag_name: String },
    #[error("the version of this app is not a version number: {version}")]
    UnreadableCurrentVersion { version: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AvailableUpdate {
    pub latest_version: String,
}

#[derive(Debug, Deserialize)]
struct LatestReleaseResponse {
    tag_name: String,
}

/// Fetches the latest release and returns it when it is newer than this build. Blocks for up to
/// a few seconds; call it from a background thread.
///
/// # Errors
///
/// Fails when GitHub cannot be reached or reports a tag that is not a version number.
pub fn check_for_newer_release() -> Result<Option<AvailableUpdate>, UpdateCheckError> {
    let latest_tag_name = fetch_latest_release_tag_name()?;
    find_newer_version(env!("CARGO_PKG_VERSION"), &latest_tag_name)
}

fn fetch_latest_release_tag_name() -> Result<String, UpdateCheckError> {
    let request_failed = |source: ureq::Error| UpdateCheckError::RequestFailed {
        source: Box::new(source),
    };

    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .user_agent(USER_AGENT)
        .build()
        .into();

    let mut response = agent
        .get(LATEST_RELEASE_URL)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(request_failed)?;
    let release: LatestReleaseResponse = response.body_mut().read_json().map_err(request_failed)?;

    Ok(release.tag_name)
}

/// Compares the running version with a release tag such as `v0.2.0`.
fn find_newer_version(
    current_version: &str,
    latest_tag_name: &str,
) -> Result<Option<AvailableUpdate>, UpdateCheckError> {
    let latest_version = latest_tag_name.trim().trim_start_matches(['v', 'V']);

    let latest_components = parse_version_components(latest_version).ok_or_else(|| {
        UpdateCheckError::UnreadableReleaseVersion {
            tag_name: latest_tag_name.to_owned(),
        }
    })?;
    let current_components = parse_version_components(current_version).ok_or_else(|| {
        UpdateCheckError::UnreadableCurrentVersion {
            version: current_version.to_owned(),
        }
    })?;

    let is_newer =
        compare_version_components(&latest_components, &current_components) == Ordering::Greater;

    Ok(is_newer.then(|| AvailableUpdate {
        latest_version: latest_version.to_owned(),
    }))
}

/// Reads `1.2.3` as `[1, 2, 3]`. A pre-release or build suffix (`-beta.1`, `+abc`) is ignored.
fn parse_version_components(version: &str) -> Option<Vec<u64>> {
    let core_version = version.split(['-', '+']).next().unwrap_or_default();
    if core_version.is_empty() {
        return None;
    }

    core_version
        .split('.')
        .map(|component| component.parse::<u64>().ok())
        .collect()
}

/// Missing trailing components count as zero, so `1.2` equals `1.2.0`.
fn compare_version_components(left: &[u64], right: &[u64]) -> Ordering {
    let component_count = left.len().max(right.len());

    (0..component_count)
        .map(|position| {
            let left_component = left.get(position).copied().unwrap_or(0);
            let right_component = right.get(position).copied().unwrap_or(0);
            left_component.cmp(&right_component)
        })
        .find(|ordering| ordering.is_ne())
        .unwrap_or(Ordering::Equal)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn newer_version(current_version: &str, latest_tag_name: &str) -> Option<String> {
        find_newer_version(current_version, latest_tag_name)
            .unwrap()
            .map(|update| update.latest_version)
    }

    #[test]
    fn a_higher_patch_version_is_an_update() {
        assert_eq!(newer_version("0.1.3", "v0.1.4"), Some("0.1.4".to_owned()));
    }

    #[test]
    fn components_compare_as_numbers_not_text() {
        assert_eq!(newer_version("0.9.0", "v0.10.0"), Some("0.10.0".to_owned()));
        assert_eq!(newer_version("0.10.0", "v0.9.0"), None);
    }

    #[test]
    fn the_same_version_is_not_an_update() {
        assert_eq!(newer_version("0.1.3", "v0.1.3"), None);
        assert_eq!(newer_version("0.1.3", "0.1.3"), None);
    }

    #[test]
    fn an_older_release_is_not_an_update() {
        assert_eq!(newer_version("1.0.0", "v0.9.9"), None);
    }

    #[test]
    fn missing_components_count_as_zero() {
        assert_eq!(newer_version("1.2.0", "v1.2"), None);
        assert_eq!(newer_version("1.2", "v1.2.1"), Some("1.2.1".to_owned()));
    }

    #[test]
    fn pre_release_suffixes_are_ignored_when_comparing() {
        assert_eq!(
            newer_version("0.1.3", "v0.2.0-beta.1"),
            Some("0.2.0-beta.1".to_owned())
        );
    }

    #[test]
    fn a_tag_that_is_not_a_version_is_an_error() {
        assert!(matches!(
            find_newer_version("0.1.3", "nightly"),
            Err(UpdateCheckError::UnreadableReleaseVersion { .. })
        ));
        assert!(matches!(
            find_newer_version("0.1.3", "v"),
            Err(UpdateCheckError::UnreadableReleaseVersion { .. })
        ));
    }
}
