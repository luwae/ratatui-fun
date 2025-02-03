use lazy_static::lazy_static;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

lazy_static! {
    static ref FILE: Mutex<File> = Mutex::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("debug.txt")
            .unwrap()
    );
}

pub fn debug_print(s: String) {
    write!(FILE.lock().unwrap(), "{}", s).unwrap();
}

pub fn debug_println(s: String) {
    writeln!(FILE.lock().unwrap(), "{}", s).unwrap();
}
