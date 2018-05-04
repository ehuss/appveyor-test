extern crate rand;
use rand::Rng;
use std::env;
use std::fs;
use std::io;
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

            if let Err(e) = fs::hard_link(&src, &dst) {
                if e.kind() != io::ErrorKind::AlreadyExists {
                    panic!("{} Failed to hard link: {}", n, e);
                }
                let parent = dst.parent().unwrap();
                let mut tries = 100;
                let tmpname = loop {
                    let rname: String = rand::thread_rng().gen_ascii_chars().take(10).collect();
                    let tmpname = parent.join(rname);
                    match fs::hard_link(&src, &tmpname) {
                        Ok(_) => break tmpname,
                        Err(e) => {
                            if e.kind() != io::ErrorKind::AlreadyExists {
                                panic!("{} Failed to hard link: {}", n, e);
                            }
                        }
                    }
                    tries -= 1;
                    if tries == 0 {
                        panic!("Could not find random name.");
                    }
                };
                if let Err(e) = fs::rename(&tmpname, &dst) {
                    // This should unlink.
                    panic!("Failed to rename temp {}", e);
                }
                // attempt to unlink, even on success, in case tmp==dst?
            }
            // let mut removed = false;
            // if dst.exists() {
            //     if let Err(e) = fs::remove_file(&dst) {
            //         panic!("{} Error removing dst: {}", n, e);
            //     }
            //     if dst.exists() {
            //         panic!("{} Exists after remove!", n);
            //     }
            //     removed = true;
            // }
            let result = std::process::Command::new(&dst)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            if let Err(e) = result {
                panic!("failed to run {:?} {}", dst, e);
            }
        }
    }
}
