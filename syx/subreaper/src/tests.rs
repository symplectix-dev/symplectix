use std::fs;
use std::path::{
    Path,
    PathBuf,
};

use testing::TempDirExt;
use tokio::task;

use super::{
    SpawnError,
    wait_for,
};

fn create_files<P, T>(temp_dir: &testing::TempDir, paths: T) -> Vec<PathBuf>
where
    P: AsRef<Path>,
    T: AsRef<[P]>,
{
    paths
        .as_ref()
        .iter()
        .map(|path| {
            let (_file, path) = temp_dir
                .create_file(fs::OpenOptions::new().create(true).read(true).write(true), path)
                .expect("create a temporary file");
            path
        })
        .collect()
}

#[tokio::test]
async fn wait_for_files() {
    let temp_dir = testing::tempdir();

    wait_for(&[]).await.expect("wait for nothing");

    let mut oks = create_files(&temp_dir, vec!["東/新宿/ok", "柏/の/葉/ok", "秋/葉/原/ok"]);
    wait_for(&oks).await.expect("waiting for files created just before");

    let err = create_files(&temp_dir, vec!["0.err"]);
    wait_for(&oks).await.expect("affected by an error file not waiting for");

    let more_oks = create_files(&temp_dir, vec!["0"]);
    oks.extend_from_slice(&more_oks);

    match wait_for(&oks)
        .await
        .expect_err("should be an error if '0' and '0.err' exist at the same time")
    {
        SpawnError::FoundErrFile(p) => {
            assert_eq!(p, err[0]);
        }
        others => {
            panic!("unexpected error: {others:?}")
        }
    }

    fs::remove_file(&more_oks[0]).unwrap();
    match wait_for(&oks)
        .await
        .expect_err("should be an error because the error file '0.err' present")
    {
        SpawnError::FoundErrFile(p) => {
            assert_eq!(p, err[0]);
        }
        others => {
            panic!("unexpected error: {others:?}")
        }
    }

    fs::remove_file(&err[0]).unwrap();
    // `wait` does not finish until the file "0" is created.
    let h = task::spawn(async move { wait_for(&oks).await });
    create_files(&temp_dir, vec!["0"]);
    h.await.unwrap().expect("should be ok");

    temp_dir.close().unwrap();
}
