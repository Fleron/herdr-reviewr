//! A manual probe against a real worktree and `gh`, never part of the suite:
//! `REVIEWR_LIVE_REPO=<worktree> cargo test --test pr_live -- --ignored --nocapture`.

use herdr_reviewr::config::PluginConfig;
use herdr_reviewr::forge::{fetch, fetch_input};

#[test]
#[ignore = "live network; run with REVIEWR_LIVE_REPO set"]
fn resolve_one_live_worktree() {
    let repo = std::env::var("REVIEWR_LIVE_REPO").expect("set REVIEWR_LIVE_REPO");
    let repo = std::path::PathBuf::from(repo);
    let input = fetch_input(&repo, None, &PluginConfig::default()).expect("fetch input");
    eprintln!("input: {input:#?}");
    let view = fetch(&repo, &input);
    eprintln!("view: {view:#?}");
}

#[test]
#[ignore = "live network; set REVIEWR_LIVE_REPO and REVIEWR_LIVE_PR"]
fn resolve_explicit_live_pr() {
    use herdr_reviewr::{
        app::{App, Tab},
        forge, git,
        model::Scope,
        world,
    };
    use std::sync::atomic::AtomicBool;

    let repo = std::path::PathBuf::from(
        std::env::var("REVIEWR_LIVE_REPO").expect("set REVIEWR_LIVE_REPO"),
    );
    let number = std::env::var("REVIEWR_LIVE_PR").expect("set REVIEWR_LIVE_PR").parse().unwrap();
    let input = fetch_input(&repo, None, &PluginConfig::default()).expect("fetch input");
    let git::RepositoryIdentity::Repository(target) = input.repository else {
        panic!("repository identity");
    };
    let cancelled = AtomicBool::new(false);
    let rows = forge::list_open_prs(&repo, &target, &cancelled).expect("list open PRs");
    assert!(rows.iter().any(|row| row.number == number), "selected open PR is listed");
    let prepared = forge::prepare_pr(&repo, &target, number, &cancelled).expect("prepare PR");
    let mut app = App::new(repo, Scope::Branch, None);
    let mut input = app.world_input();
    input.pr_revision = Some(prepared.revision.clone());
    input.tab = Tab::Changes;
    let changes = world::build(&input).expect("PR changes");
    input.tab = Tab::AllFiles;
    let files = world::build(&input).expect("PR files");
    assert!(!files.entries.is_empty(), "PR head tree has files");
    let head = prepared.revision.head_oid.clone();
    app.install_pr_target(prepared, changes, files);
    for tab in [Tab::Changes, Tab::AllFiles, Tab::Pr] {
        app.set_tab(tab).unwrap();
        assert_eq!(app.pr_target.as_ref().unwrap().revision.head_oid, head);
    }
    eprintln!(
        "PR #{number}: {} open PRs listed; pinned head {head}; all three tabs loaded",
        rows.len()
    );
}
