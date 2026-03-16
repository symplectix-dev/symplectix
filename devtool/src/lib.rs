//! This crate `devtool` provides internal tooling.

use std::path::{
    Path,
    PathBuf,
};
use std::process::Stdio;
use std::{
    env,
    process,
};

use serde::{
    Deserialize,
    Serialize,
};

mod info;

trait DevTool {
    fn run(self, ctx: Context) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
struct Context {
    #[allow(dead_code)]
    cargo: PathBuf,
    workspace_status: WorkspaceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceStatus {
    year:      u32,
    week:      u32,
    rev_count: u32,
    rev_parse: String,
}

impl WorkspaceStatus {
    fn version(&self) -> String {
        let WorkspaceStatus { year, week, rev_count, rev_parse } = self;
        format!("{year}.{week}.{rev_count}+r{rev_parse}")
    }
}

/// Arguments for the devtool cli.
#[derive(Debug, Clone, clap::Parser)]
pub struct Cli {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum Command {
    /// Print buildinfo collected at compile time.
    Info(info::Info),
}

impl Cli {
    /// Run a tool and wait its result.
    pub fn run(self) -> anyhow::Result<()> {
        let project_root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/.."));
        assert!(project_root.join(".git").exists());
        // Always run tools from the project root for consistency.
        env::set_current_dir(project_root)?;

        let workspace_status: WorkspaceStatus = {
            let output = process::Command::new("uv")
                .arg("run")
                .arg("devtool/workspace_status.py")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
                .wait_with_output()?;
            if output.status.success() {
                serde_json::from_slice(&output.stdout)?
            } else {
                let err = str::from_utf8(&output.stderr)
                    .expect("workspace_status.py emits non-utf8 error string");
                println!("workspace_status.py failed: {}", err);
                process::exit(1);
            }
        };

        // Github Actions environment variables.
        // https://docs.github.com/en/actions/learn-github-actions/contexts#github-context
        //
        // * run_id: A unique number for each workflow run within a repository. This number does not
        //   change if you re-run the workflow run.
        // * run_number: A unique number for each run of a particular workflow in a repository. This
        //   number begins at 1 for the workflow's first run, and increments with each new run. This
        //   number does not change if you re-run the workflow run.
        // * run_attempt: A unique number for each attempt of a particular workflow run in a
        //   repository. This number begins at 1 for the workflow run's first attempt, and
        //   increments with each re-run.
        // let run_number = env::var("GITHUB_RUN_NUMBER").unwrap_or("0".to_owned());

        let ctx = Context { cargo: PathBuf::from(env!("CARGO")), workspace_status };
        match self.cmd {
            Command::Info(c) => c.run(ctx),
        }
    }
}
