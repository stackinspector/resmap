use std::{path::{PathBuf, Path}, fs, io, env};

fn assert_dir(entry: &fs::DirEntry) {
    assert!(entry.file_type().unwrap().is_dir());
}

fn assert_file(entry: &fs::DirEntry) {
    assert!(entry.file_type().unwrap().is_file());
}

fn assert_name_vaild(name: &[u8]) {
    for b in name {
        assert!(matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.'));
    }
}

fn vaild_name(entry: &fs::DirEntry) -> String {
    let name = entry.file_name().into_string().unwrap();
    assert_name_vaild(name.as_bytes());
    name
}

fn is_newest(name: &[u8]) -> bool {
    name[0].is_ascii_lowercase()
}

fn hash_file(file_path: &Path) -> String {
    use sha2::{Digest as _, Sha256};
    let mut hasher = Sha256::new();
    let mut file_handle = fs::OpenOptions::new().read(true).open(file_path).unwrap();
    io::copy(&mut file_handle, &mut hasher).unwrap();
    hex::encode(hasher.finalize())
}

fn main() {
    let path = PathBuf::from(env::args_os().next().unwrap());
    let blobs_path = path.join("blobs");
    for ty_entry in fs::read_dir(path).unwrap().map(Result::unwrap) {
        assert_dir(&ty_entry);
        let ty_name = vaild_name(&ty_entry);
        if ty_name == "blobs" { continue; }
        for file_entry in fs::read_dir(ty_entry.path()).unwrap().map(Result::unwrap) {
            assert_file(&file_entry);
            let file_name = vaild_name(&file_entry);
            let file_path = file_entry.path();
            let hash = hash_file(&file_path);
            let dst = if is_newest(file_name.as_bytes()) {
                println!("/{}/{} -> /blobs/{}", ty_name, file_name, hash);
                match fs::OpenOptions::new().create_new(true).write(true).open(blobs_path.join(&hash)) {
                    Ok(mut dst_file_handle) => {
                        let mut file_handle = fs::OpenOptions::new().read(true).open(&file_path).unwrap();
                        io::copy(&mut file_handle, &mut dst_file_handle).unwrap();
                        "created"
                    }
                    Err(err) => if err.kind() == io::ErrorKind::AlreadyExists {
                        "exists "
                    } else {
                        panic!("called `Result::unwrap()` on an `Err` value: {:?}", err);
                    }
                }
            } else {
                "ignored"
            };
            eprintln!("dst={} hash={} ty={} file={}", dst, hash, ty_name, file_name);
        }
    }
}
