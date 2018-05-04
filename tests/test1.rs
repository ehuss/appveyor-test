use std::io::Read;
use std::env;
use std::fs;
use std::path::PathBuf;

#[test]
fn testit1() {
    testit(1);
}
#[test]
fn testit2() {
    testit(2);
}
#[test]
fn testit3() {
    testit(3);
}
#[test]
fn testit4() {
    testit(4);
}

fn testit(n: u32) {
    let root = PathBuf::from("target").join(format!("t{}", n));
    if !root.exists() {
        fs::create_dir(&root).unwrap();
    }
    let dst = root.join(format!("appveyor-test{}", env::consts::EXE_SUFFIX));
    let bins = fs::read_dir("target/debug/deps")
        .unwrap()
        .map(|de| de.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("appveyor_test-")
        })
        .filter(|path| {
            if env::consts::EXE_SUFFIX.len() == 0 {
                path.extension().is_none()
            } else {
                path.extension().unwrap_or(::std::ffi::OsStr::new("")) == "exe"
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(bins.len(), 2);
    let bins = bins.iter()
        .map(|path| {
            let d = root.join(path.file_name().unwrap());
            fs::copy(path, &d).unwrap();
            d
        })
        .collect::<Vec<_>>();

    println!("bins={:?} dst={:?}", bins, dst);
    for n in 0..1000 {
        for src in &bins {
            let mut removed = false;
            if dst.exists() {
                if let Err(e) = fs::remove_file(&dst) {
                    panic!("{} Error removing dst: {}", n, e);
                }
                if dst.exists() {
                    panic!("{} Exists after remove!", n);
                }
                removed = true;
            }
            if let Err(e) = fs::hard_link(&src, &dst) {
                panic!("{} Failed to hard link: {} {}", n, e, removed);
            }
            let mut f = fs::File::open(&dst).unwrap();
            let mut contents = Vec::new();
            f.read_to_end(&mut contents).unwrap();
            drop(f);
            // let result = std::process::Command::new(&dst)
            //     .stdin(std::process::Stdio::null())
            //     .stdout(std::process::Stdio::null())
            //     .stderr(std::process::Stdio::null())
            //     .status();
            // if let Err(e) = result {
            //     panic!("failed to run {:?} {} {}", dst, e, removed);
            // }
        }
    }
}
