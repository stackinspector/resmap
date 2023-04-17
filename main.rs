use std::{path::{PathBuf, Path}, fs, io::{self, Write}, env};

const BLOB_PATH: &str = "namedblobs";
const EXT: &[u8] = b".zip";

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

fn file_name_without_tail(name: &[u8]) -> &str {
    let (name, tail) = name.split_at(name.len() - 4);
    assert_eq!(tail, EXT);
    std::str::from_utf8(name).unwrap()
}

fn file_name_insert_tail(name: &[u8], hash: &[u8]) -> String {
    let (name, tail) = name.split_at(name.len() - 4);
    assert_eq!(tail, EXT);
    String::from_utf8([name, b"-", hash, EXT].concat()).unwrap()
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
    use crc32fast::Hasher;
    struct CrcWriter(Hasher);
    impl io::Write for CrcWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.update(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut hasher = CrcWriter(Hasher::new());
    let mut file_handle = fs::OpenOptions::new().read(true).open(file_path).unwrap();
    io::copy(&mut file_handle, &mut hasher).unwrap();
    hex::encode(hasher.0.finalize().to_be_bytes())
}

fn main() {
    let mut args = env::args_os();
    let _ = args.next();
    let path = PathBuf::from(args.next().unwrap());
    let list_path = PathBuf::from(args.next().unwrap());
    let mut list1_file_handle = fs::OpenOptions::new().create_new(true).write(true).open(list_path.join("mirror-cn")).unwrap();
    let mut list2_file_handle = fs::OpenOptions::new().create_new(true).write(true).open(list_path.join("mirror-os")).unwrap();
    let blobs_path = path.join(BLOB_PATH);
    for ty_entry in fs::read_dir(path).unwrap().map(Result::unwrap) {
        assert_dir(&ty_entry);
        let ty_name = vaild_name(&ty_entry);
        if ty_name == BLOB_PATH || !is_newest(ty_name.as_bytes()) { continue; }
        for file_entry in fs::read_dir(ty_entry.path()).unwrap().map(Result::unwrap) {
            assert_file(&file_entry);
            let file_name = vaild_name(&file_entry);
            let file_path = file_entry.path();
            let hash = hash_file(&file_path);
            let dst = if is_newest(file_name.as_bytes()) {
                let dst_name = file_name_insert_tail(file_name.as_bytes(), hash.as_bytes());
                writeln!(list1_file_handle, "{} d1.ldtstore.com.cn/{}/{}", file_name_without_tail(file_name.as_bytes()), BLOB_PATH, dst_name).unwrap();
                writeln!(list2_file_handle, "{} d1.ldtstore.net/{}/{}", file_name_without_tail(file_name.as_bytes()), BLOB_PATH, dst_name).unwrap();
                match fs::OpenOptions::new().create_new(true).write(true).open(blobs_path.join(&dst_name)) {
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
