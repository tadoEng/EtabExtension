// ext-api::guards — centralised state-guard permission matrix.
//
// Every ext-api function calls `check_state_guard` at entry before doing any
// work.  This encodes the full permission matrix from workflow.md §15 in one
// testable place.  No guard logic is duplicated across commands.
//
// GuardOutcome::Block  → bail! immediately, do not proceed.
// GuardOutcome::Warn   → proceed, attach warning to result struct.
// GuardOutcome::Allow  → proceed unconditionally.

use ext_core::state::WorkingFileStatus;

// ── Command enum ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Commit,
    CommitAnalyze,
    Switch,
    Checkout,
    StashSave,
    StashPop,
    Analyze,
    EtabsOpen,
    // Always-allowed read-only commands.
    Status,
    Log,
    Show,
    Diff,
    Push,
    Report,
    ConfigGet,
    ConfigList,
}

// ── GuardOutcome ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardOutcome {
    Allow,
    Warn(String),
    Block(String),
}

// ── Permission matrix ─────────────────────────────────────────────────────────

pub fn check_state_guard(command: Command, status: &WorkingFileStatus) -> GuardOutcome {
    use Command::*;
    use GuardOutcome::*;
    use WorkingFileStatus::*;

    match (command, status) {
        // ── Commit ────────────────────────────────────────────────────────
        (Commit, OpenClean | OpenModified) => {
            Block("✗ Close ETABS before committing\n  Run: ext etabs close".into())
        }
        (Commit, Orphaned) => {
            Block("✗ Working file state unknown\n  Run: ext etabs recover".into())
        }
        (Commit, Missing) => Block("✗ Working file missing\n  Run: ext checkout vN".into()),
        (Commit, Analyzed | Locked) => Warn(
            "⚠ Working file has analysis results. Consider: ext commit --analyze to capture them."
                .into(),
        ),
        (Commit, _) => Allow,

        (CommitAnalyze, OpenClean | OpenModified) => {
            Block("✗ Close ETABS before committing\n  Run: ext etabs close".into())
        }
        (CommitAnalyze, Orphaned) => {
            Block("✗ Working file state unknown\n  Run: ext etabs recover".into())
        }
        (CommitAnalyze, Missing) => Block("✗ Working file missing\n  Run: ext checkout vN".into()),
        (CommitAnalyze, Locked) => {
            Block("✗ Model is locked after analysis\n  Run: ext etabs unlock".into())
        }
        (CommitAnalyze, _) => Allow,

        // ── Switch ────────────────────────────────────────────────────────
        (Switch, OpenClean | OpenModified) => {
            Block("✗ Close ETABS before switching branches\n  Run: ext etabs close".into())
        }
        (Switch, Orphaned) => {
            Block("✗ Working file state unknown\n  Run: ext etabs recover".into())
        }
        (Switch, Modified | Analyzed | Locked) => {
            Warn("⚠ Leaving branch with uncommitted changes".into())
        }
        (Switch, Missing) => Warn("⚠ Working file is missing on this branch".into()),
        (Switch, _) => Allow,

        // ── Checkout ─────────────────────────────────────────────────────
        // MODIFIED is handled via CheckoutConflictResolution — not blocked here.
        (Checkout, OpenClean | OpenModified) => {
            Block("✗ Close ETABS before checking out\n  Run: ext etabs close".into())
        }
        (Checkout, Analyzed | Locked) => Block(
            "✗ Close ETABS and commit analysis results first\n  Run: ext commit --analyze".into(),
        ),
        (Checkout, Orphaned) => {
            Block("✗ Working file state unknown\n  Run: ext etabs recover".into())
        }
        (Checkout, _) => Allow,

        // ── StashSave ─────────────────────────────────────────────────────
        (StashSave, Untracked | Clean | Analyzed | Locked | Missing) => {
            Block("✗ Nothing to stash (working file is not modified)".into())
        }
        (StashSave, OpenClean | OpenModified) => {
            Block("✗ Close ETABS before stashing\n  Run: ext etabs close".into())
        }
        (StashSave, _) => Allow,

        // ── StashPop ──────────────────────────────────────────────────────
        (StashPop, OpenClean | OpenModified) => {
            Block("✗ Close ETABS before restoring stash\n  Run: ext etabs close".into())
        }
        (StashPop, Orphaned) => {
            Block("✗ Working file state unknown\n  Run: ext etabs recover".into())
        }
        (StashPop, Analyzed | Locked) => {
            Block("✗ Commit or discard analysis results before restoring stash".into())
        }
        (StashPop, Untracked) => Block("✗ Cannot pop stash onto an untracked working file".into()),
        (StashPop, _) => Allow,

        // ── Analyze ───────────────────────────────────────────────────────
        // Analyze operates on a committed snapshot, never the working file.
        (Analyze, OpenClean | OpenModified) => {
            Block("✗ Close ETABS before running analysis\n  Run: ext etabs close".into())
        }
        (Analyze, _) => Allow,

        // ── EtabsOpen ────────────────────────────────────────────────────
        (EtabsOpen, OpenClean | OpenModified) => {
            Block("✗ ETABS is already running\n  Run: ext etabs close".into())
        }
        (EtabsOpen, Missing) => Block("✗ Working file missing\n  Run: ext checkout vN".into()),
        (EtabsOpen, Orphaned) => {
            Block("✗ ETABS crashed previously\n  Run: ext etabs recover".into())
        }
        (EtabsOpen, _) => Allow,

        // ── Always allowed ────────────────────────────────────────────────
        (Status | Log | Show | Diff | Push | Report | ConfigGet | ConfigList, _) => Allow,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use WorkingFileStatus::*;

    fn is_block(cmd: Command, st: WorkingFileStatus) -> bool {
        matches!(check_state_guard(cmd, &st), GuardOutcome::Block(_))
    }
    fn is_warn(cmd: Command, st: WorkingFileStatus) -> bool {
        matches!(check_state_guard(cmd, &st), GuardOutcome::Warn(_))
    }
    fn is_allow(cmd: Command, st: WorkingFileStatus) -> bool {
        matches!(check_state_guard(cmd, &st), GuardOutcome::Allow)
    }

    // Commit
    #[test]
    fn commit_blocked_when_open_clean() {
        assert!(is_block(Command::Commit, OpenClean));
    }
    #[test]
    fn commit_blocked_when_open_modified() {
        assert!(is_block(Command::Commit, OpenModified));
    }
    #[test]
    fn commit_blocked_when_orphaned() {
        assert!(is_block(Command::Commit, Orphaned));
    }
    #[test]
    fn commit_blocked_when_missing() {
        assert!(is_block(Command::Commit, Missing));
    }
    #[test]
    fn commit_warns_when_analyzed() {
        assert!(is_warn(Command::Commit, Analyzed));
    }
    #[test]
    fn commit_warns_when_locked() {
        assert!(is_warn(Command::Commit, Locked));
    }
    #[test]
    fn commit_allowed_when_clean() {
        assert!(is_allow(Command::Commit, Clean));
    }
    #[test]
    fn commit_allowed_when_modified() {
        assert!(is_allow(Command::Commit, Modified));
    }
    #[test]
    fn commit_allowed_when_untracked() {
        assert!(is_allow(Command::Commit, Untracked));
    }

    // CommitAnalyze
    #[test]
    fn commit_analyze_blocked_when_open_clean() {
        assert!(is_block(Command::CommitAnalyze, OpenClean));
    }
    #[test]
    fn commit_analyze_blocked_when_open_modified() {
        assert!(is_block(Command::CommitAnalyze, OpenModified));
    }
    #[test]
    fn commit_analyze_blocked_when_orphaned() {
        assert!(is_block(Command::CommitAnalyze, Orphaned));
    }
    #[test]
    fn commit_analyze_blocked_when_missing() {
        assert!(is_block(Command::CommitAnalyze, Missing));
    }
    #[test]
    fn commit_analyze_blocked_when_locked() {
        assert!(is_block(Command::CommitAnalyze, Locked));
    }
    #[test]
    fn commit_analyze_allowed_when_clean() {
        assert!(is_allow(Command::CommitAnalyze, Clean));
    }
    #[test]
    fn commit_analyze_allowed_when_modified() {
        assert!(is_allow(Command::CommitAnalyze, Modified));
    }
    #[test]
    fn commit_analyze_allowed_when_untracked() {
        assert!(is_allow(Command::CommitAnalyze, Untracked));
    }
    #[test]
    fn commit_analyze_allowed_when_analyzed() {
        assert!(is_allow(Command::CommitAnalyze, Analyzed));
    }

    // Switch
    #[test]
    fn switch_blocked_when_open_clean() {
        assert!(is_block(Command::Switch, OpenClean));
    }
    #[test]
    fn switch_blocked_when_open_modified() {
        assert!(is_block(Command::Switch, OpenModified));
    }
    #[test]
    fn switch_blocked_when_orphaned() {
        assert!(is_block(Command::Switch, Orphaned));
    }
    #[test]
    fn switch_warns_when_modified() {
        assert!(is_warn(Command::Switch, Modified));
    }
    #[test]
    fn switch_warns_when_analyzed() {
        assert!(is_warn(Command::Switch, Analyzed));
    }
    #[test]
    fn switch_warns_when_missing() {
        assert!(is_warn(Command::Switch, Missing));
    }
    #[test]
    fn switch_allowed_when_clean() {
        assert!(is_allow(Command::Switch, Clean));
    }
    #[test]
    fn switch_allowed_when_untracked() {
        assert!(is_allow(Command::Switch, Untracked));
    }

    // Checkout
    #[test]
    fn checkout_blocked_when_open_clean() {
        assert!(is_block(Command::Checkout, OpenClean));
    }
    #[test]
    fn checkout_blocked_when_analyzed() {
        assert!(is_block(Command::Checkout, Analyzed));
    }
    #[test]
    fn checkout_blocked_when_locked() {
        assert!(is_block(Command::Checkout, Locked));
    }
    #[test]
    fn checkout_blocked_when_orphaned() {
        assert!(is_block(Command::Checkout, Orphaned));
    }
    #[test]
    fn checkout_allowed_when_modified() {
        assert!(is_allow(Command::Checkout, Modified));
    }
    #[test]
    fn checkout_allowed_when_clean() {
        assert!(is_allow(Command::Checkout, Clean));
    }
    #[test]
    fn checkout_allowed_when_missing() {
        assert!(is_allow(Command::Checkout, Missing));
    }

    // StashSave
    #[test]
    fn stash_save_blocked_when_clean() {
        assert!(is_block(Command::StashSave, Clean));
    }
    #[test]
    fn stash_save_blocked_when_untracked() {
        assert!(is_block(Command::StashSave, Untracked));
    }
    #[test]
    fn stash_save_blocked_when_analyzed() {
        assert!(is_block(Command::StashSave, Analyzed));
    }
    #[test]
    fn stash_save_blocked_when_missing() {
        assert!(is_block(Command::StashSave, Missing));
    }
    #[test]
    fn stash_save_blocked_when_open_clean() {
        assert!(is_block(Command::StashSave, OpenClean));
    }
    #[test]
    fn stash_save_blocked_when_open_modified() {
        assert!(is_block(Command::StashSave, OpenModified));
    }
    #[test]
    fn stash_save_allowed_when_modified() {
        assert!(is_allow(Command::StashSave, Modified));
    }

    // StashPop
    #[test]
    fn stash_pop_blocked_when_open_clean() {
        assert!(is_block(Command::StashPop, OpenClean));
    }
    #[test]
    fn stash_pop_blocked_when_orphaned() {
        assert!(is_block(Command::StashPop, Orphaned));
    }
    #[test]
    fn stash_pop_blocked_when_untracked() {
        assert!(is_block(Command::StashPop, Untracked));
    }
    #[test]
    fn stash_pop_blocked_when_analyzed() {
        assert!(is_block(Command::StashPop, Analyzed));
    }
    #[test]
    fn stash_pop_allowed_when_clean() {
        assert!(is_allow(Command::StashPop, Clean));
    }
    #[test]
    fn stash_pop_allowed_when_modified() {
        assert!(is_allow(Command::StashPop, Modified));
    }

    // EtabsOpen
    #[test]
    fn etabs_open_blocked_when_already_open() {
        assert!(is_block(Command::EtabsOpen, OpenClean));
    }
    #[test]
    fn etabs_open_blocked_when_open_modified() {
        assert!(is_block(Command::EtabsOpen, OpenModified));
    }
    #[test]
    fn etabs_open_blocked_when_missing() {
        assert!(is_block(Command::EtabsOpen, Missing));
    }
    #[test]
    fn etabs_open_blocked_when_orphaned() {
        assert!(is_block(Command::EtabsOpen, Orphaned));
    }
    #[test]
    fn etabs_open_allowed_when_clean() {
        assert!(is_allow(Command::EtabsOpen, Clean));
    }
    #[test]
    fn etabs_open_allowed_when_modified() {
        assert!(is_allow(Command::EtabsOpen, Modified));
    }

    // Always-allowed commands
    #[test]
    fn log_always_allowed_in_any_state() {
        for st in [
            Clean,
            Modified,
            Missing,
            OpenClean,
            OpenModified,
            Orphaned,
            Analyzed,
            Locked,
            Untracked,
        ] {
            assert!(is_allow(Command::Log, st), "Log blocked in {st:?}");
        }
    }
    #[test]
    fn diff_always_allowed_in_any_state() {
        for st in [
            Clean,
            Modified,
            Missing,
            OpenClean,
            OpenModified,
            Orphaned,
            Analyzed,
            Locked,
            Untracked,
        ] {
            assert!(is_allow(Command::Diff, st), "Diff blocked in {st:?}");
        }
    }
    #[test]
    fn show_always_allowed_in_any_state() {
        assert!(is_allow(Command::Show, Missing));
        assert!(is_allow(Command::Show, OpenModified));
    }
}
