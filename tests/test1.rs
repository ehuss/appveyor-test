use std::env;
use std::fs;
use std::path::PathBuf;

#[test]
fn testit() {
    let dst = PathBuf::from("target/debug/appveyor-test");
    let bins = fs::read_dir("target/debug/deps")
        .unwrap()
        .map(|de| de.unwrap().path())
        .filter(|path| path.file_name().unwrap().to_str().unwrap().starts_with("appveyor_test-"))
        .filter(|path| {
            if env::consts::EXE_SUFFIX.len() == 0 {
                path.extension().is_none()
            } else {
                path.extension().unwrap_or(::std::ffi::OsStr::new("")) == "exe"
            }
        })
        .collect::<Vec<_>>();

    assert_eq!(bins.len(), 2);

    println!("bins={:?} dst={:?}", bins, dst);
    for n in 0..1000 {
        for src in &bins {
            let mut removed = false;
            if dst.exists() {
                if let Err(e) = fs::remove_file(&dst) {
                    panic!("{} Error removing dst: {}", n, e);
                }
                removed = true;
            }
            if let Err(e) = fs::hard_link(&src, &dst) {
                panic!("{} Failed to hard link: {} {}", n, e, removed);
            }
            let result = std::process::Command::new(&dst)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            if let Err(e) = result {
                panic!("failed to run {} {}", e, removed);
            }
        }
    }
}
