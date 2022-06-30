use std::ffi::OsString;
use std::fs;

pub fn file_names() -> Vec<OsString> {
    let roms = fs::read_dir("src/roms").expect("Directory exists");
    let mut files: Vec<OsString> = Vec::new();
    for x in roms {
        files.push(match x {
            Ok(T) => T.file_name(),
            _ => OsString::default(),
        });
    }
    return files;
}

pub fn get_contents(file_name: String) -> Vec<u8> {
    let path = "src/roms/".to_owned() + &*file_name;
    return fs::read(path).expect("File Exists.");
}
