// ext-core::state — working file status resolver.
//
// This module is pure domain logic. It does not perform filesystem/process IO.
// Callers provide the observed facts and this resolver returns the status using
// the defined precedence order.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkingFileStatus {
    Missing,
    OpenClean,
    OpenModified,
    Orphaned,
    // Deferred to Week 5-6: these states need live ETABS/sidecar signals and
    // are not inferred by the Week 3-4 pure resolver yet.
    Analyzed,
    Clean,
    Modified,
    Untracked,
    Locked,
}

impl std::fmt::Display for WorkingFileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Missing => "Missing",
            Self::OpenClean => "OpenClean",
            Self::OpenModified => "OpenModified",
            Self::Orphaned => "Orphaned",
            Self::Analyzed => "Analyzed",
            Self::Clean => "Clean",
            Self::Modified => "Modified",
            Self::Untracked => "Untracked",
            Self::Locked => "Locked",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone)]
pub struct ResolveInput {
    pub file_exists: bool,
    pub etabs_pid: Option<u32>,
    pub pid_alive: bool,
    pub based_on_version: Option<String>,
    pub last_known_mtime: Option<DateTime<Utc>>,
    pub current_mtime: Option<DateTime<Utc>>,
}

fn is_modified(
    last_known_mtime: Option<DateTime<Utc>>,
    current_mtime: Option<DateTime<Utc>>,
) -> bool {
    match (last_known_mtime, current_mtime) {
        (Some(last), Some(current)) => current > last,
        (None, Some(_)) => true,
        _ => false,
    }
}

pub fn resolve(input: ResolveInput) -> WorkingFileStatus {
    if !input.file_exists {
        return WorkingFileStatus::Missing;
    }

    if input.etabs_pid.is_some() {
        if input.pid_alive {
            if is_modified(input.last_known_mtime, input.current_mtime) {
                return WorkingFileStatus::OpenModified;
            }
            return WorkingFileStatus::OpenClean;
        }
        return WorkingFileStatus::Orphaned;
    }

    if input.based_on_version.is_none() {
        return WorkingFileStatus::Untracked;
    }

    if is_modified(input.last_known_mtime, input.current_mtime) {
        WorkingFileStatus::Modified
    } else {
        WorkingFileStatus::Clean
    }
}

#[cfg(test)]
mod tests {
    use super::{ResolveInput, WorkingFileStatus, resolve};
    use chrono::{Duration, Utc};

    fn base() -> ResolveInput {
        let now = chrono::DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap();
        ResolveInput {
            file_exists: true,
            etabs_pid: None,
            pid_alive: false,
            based_on_version: Some("v1".to_string()),
            last_known_mtime: Some(now),
            current_mtime: Some(now),
        }
    }

    #[test]
    fn missing_overrides_everything() {
        let mut input = base();
        input.file_exists = false;
        input.etabs_pid = Some(123);
        input.pid_alive = true;
        input.based_on_version = None;
        assert_eq!(resolve(input), WorkingFileStatus::Missing);
    }

    #[test]
    fn open_clean_when_pid_alive_and_not_modified() {
        let mut input = base();
        input.etabs_pid = Some(123);
        input.pid_alive = true;
        assert_eq!(resolve(input), WorkingFileStatus::OpenClean);
    }

    #[test]
    fn open_modified_when_pid_alive_and_mtime_increased() {
        let mut input = base();
        input.etabs_pid = Some(123);
        input.pid_alive = true;
        input.current_mtime = input.last_known_mtime.map(|t| t + Duration::seconds(1));
        assert_eq!(resolve(input), WorkingFileStatus::OpenModified);
    }

    #[test]
    fn orphaned_when_pid_present_but_dead() {
        let mut input = base();
        input.etabs_pid = Some(123);
        input.pid_alive = false;
        assert_eq!(resolve(input), WorkingFileStatus::Orphaned);
    }

    #[test]
    fn untracked_when_no_based_on_version() {
        let mut input = base();
        input.based_on_version = None;
        assert_eq!(resolve(input), WorkingFileStatus::Untracked);
    }

    #[test]
    fn modified_when_mtime_increased() {
        let mut input = base();
        input.current_mtime = input.last_known_mtime.map(|t| t + Duration::seconds(1));
        assert_eq!(resolve(input), WorkingFileStatus::Modified);
    }

    #[test]
    fn clean_when_mtime_not_increased() {
        let input = base();
        assert_eq!(resolve(input), WorkingFileStatus::Clean);
    }
}
