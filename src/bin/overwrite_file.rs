use std::fs::OpenOptions;
use std::io::{prelude::*, Seek, SeekFrom};

fn main() {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(false)
        .open("./test.txt")
        .unwrap();


    file.seek(SeekFrom::Start(0)).unwrap();
    file.write_all(b"That's overwrite!\nThanks you\n").unwrap();
    file.flush().unwrap();
}
