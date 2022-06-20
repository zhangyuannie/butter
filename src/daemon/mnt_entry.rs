use std::{
    io::{self, Error, ErrorKind},
    path::PathBuf,
};

pub struct MntEntries<R: io::BufRead> {
    reader: R,
}

impl<R: io::BufRead> MntEntries<R> {
    pub fn new(reader: R) -> MntEntries<R> {
        MntEntries { reader }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MntEntry {
    pub spec: String,
    pub target: PathBuf,
    pub fs_type: String,
    pub options: String,
    pub dump_freq: i32,
    pub pass: i32,
}

impl<R: io::BufRead> Iterator for MntEntries<R> {
    type Item = io::Result<MntEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        loop {
            match self.reader.read_line(&mut buf) {
                Ok(0) => return None,
                Ok(_) => {
                    if buf.len() == 0 || buf.starts_with("#") {
                        // skip comment and empty line
                        continue;
                    } else {
                        let mut words = buf.split_ascii_whitespace();

                        return Some(Ok(MntEntry {
                            spec: match words.next() {
                                Some(s) => s.to_string(),
                                None => return Some(Err(Error::from(ErrorKind::InvalidData))),
                            },
                            target: match words.next() {
                                Some(s) => PathBuf::from(s),
                                None => return Some(Err(Error::from(ErrorKind::InvalidData))),
                            },
                            fs_type: match words.next() {
                                Some(s) => s.to_string(),
                                None => return Some(Err(Error::from(ErrorKind::InvalidData))),
                            },
                            options: match words.next() {
                                Some(s) => s.to_string(),
                                None => return Some(Err(Error::from(ErrorKind::InvalidData))),
                            },
                            dump_freq: match words.next() {
                                Some(s) => s.parse().unwrap_or(0),
                                None => 0,
                            },
                            pass: match words.next() {
                                Some(s) => s.parse().unwrap_or(0),
                                None => 0,
                            },
                        }));
                    }
                }
                Err(e) => return Some(Err(e)),
            };
        }
    }
}
