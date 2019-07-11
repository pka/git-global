//! Git repository representation for git-global.

use chrono::{TimeZone, Utc};
use std::fmt;
use std::path::PathBuf;

use git2;

/// A git repository, represented by the full path to its base directory.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Repo {
    path: PathBuf,
}

impl Repo {
    pub fn new(path: String) -> Repo {
        Repo {
            path: PathBuf::from(path),
        }
    }

    /// Returns the `git2::Repository` equivalent of this repo.
    pub fn as_git2_repo(&self) -> git2::Repository {
        git2::Repository::open(&self.path).ok().expect(
            "Could not open {} as a git repo. Perhaps you should run \
             `git global scan` again.",
        )
    }

    /// Returns the full path to the repo as a `String`.
    pub fn path(&self) -> String {
        self.path.to_str().unwrap().to_string()
    }

    /// Returns the age of the last commit in hours.
    pub fn num_hours_since_last_commit(&self) -> i64 {
        // dbg!(&self.path);
        let git2_repo = self.as_git2_repo();
        // dbg!(git2_repo.state());
        if let Ok(head) = git2_repo.head() {
            if let Some(oid) = head.target() {
                if let Ok(commit) = git2_repo.find_commit(oid) {
                    let commit_time = Utc.timestamp(commit.time().seconds(), 0);
                    let age_h = Utc::now()
                        .signed_duration_since(commit_time)
                        .num_hours();
                    return age_h;
                }
            }
        }
        i64::max_value()
    }

    pub fn get_status(&self) -> Vec<String> {
        let mut status_opts = git2::StatusOptions::new();
        status_opts
            .show(git2::StatusShow::IndexAndWorkdir)
            .include_untracked(true)
            .include_ignored(false);
        self.get_status_lines(status_opts)
    }

    pub fn get_short_status(&self) -> String {
        let git2_repo = self.as_git2_repo();
        let mut status_opts = git2::StatusOptions::new();
        status_opts.show(git2::StatusShow::Workdir);
        let statuses = git2_repo
            .statuses(Some(&mut status_opts))
            .expect(&format!("Could not get statuses for {}.", self));
        if let Some(status) = statuses.get(0) {
            get_short_format_status(status.status())
        } else {
            "?".to_string()
        }
    }

    /// Returns "short format" status output.
    pub fn get_status_lines(
        &self,
        mut status_opts: git2::StatusOptions,
    ) -> Vec<String> {
        let git2_repo = self.as_git2_repo();
        let statuses = git2_repo
            .statuses(Some(&mut status_opts))
            .expect(&format!("Could not get statuses for {}.", self));
        statuses
            .iter()
            .map(|entry| {
                let path = entry.path().unwrap();
                let status = entry.status();
                let status_for_path = get_short_format_status(status);
                format!("{} {}", status_for_path, path)
            })
            .collect()
    }

    /// Returns the list of stash entries for the repo.
    pub fn get_stash_list(&self) -> Vec<String> {
        let mut stash = vec![];
        self.as_git2_repo()
            .stash_foreach(|index, name, _oid| {
                stash.push(format!("stash@{{{}}}: {}", index, name));
                true
            })
            .unwrap();
        stash
    }
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path())
    }
}

/// Translates a file's status flags to their "short format" representation.
///
/// Follows an example in the git2-rs crate's `examples/status.rs`.
fn get_short_format_status(status: git2::Status) -> String {
    let mut istatus = match status {
        s if s.is_index_new() => 'A',
        s if s.is_index_modified() => 'M',
        s if s.is_index_deleted() => 'D',
        s if s.is_index_renamed() => 'R',
        s if s.is_index_typechange() => 'T',
        _ => ' ',
    };
    let mut wstatus = match status {
        s if s.is_wt_new() => {
            if istatus == ' ' {
                istatus = '?';
            }
            '?'
        }
        s if s.is_wt_modified() => 'M',
        s if s.is_wt_deleted() => 'D',
        s if s.is_wt_renamed() => 'R',
        s if s.is_wt_typechange() => 'T',
        _ => ' ',
    };
    if status.is_ignored() {
        istatus = '!';
        wstatus = '!';
    }
    if status.is_conflicted() {
        istatus = 'C';
        wstatus = 'C';
    }
    // TODO: handle submodule statuses?
    format!("{}{}", istatus, wstatus)
}
