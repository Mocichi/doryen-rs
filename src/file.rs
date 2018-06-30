use std;
use std::collections::HashMap;

use uni_app;

struct AsyncFile(String, uni_app::fs::File, Option<Vec<u8>>);

pub struct FileLoader {
    files_to_load: HashMap<usize, AsyncFile>,
    seq: usize,
}

impl FileLoader {
    pub fn new() -> Self {
        Self {
            files_to_load: HashMap::new(),
            seq: 0,
        }
    }
    pub fn load_file(&mut self, path: &str) -> usize {
        uni_app::App::print(format!("loading file {}\n", path));
        match open_file(path) {
            Ok(mut f) => {
                if f.is_ready() {
                    match f.read_binary() {
                        Ok(buf) => {
                            self.files_to_load
                                .insert(self.seq, AsyncFile(path.to_owned(), f, Some(buf)));
                            self.seq += 1;
                            return self.seq - 1;
                        }
                        Err(e) => panic!("Could not read file {} : {}\n", path, e),
                    }
                } else {
                    uni_app::App::print(format!("loading async file {}\n", path));
                    self.files_to_load
                        .insert(self.seq, AsyncFile(path.to_owned(), f, None));
                    self.seq += 1;
                    return self.seq - 1;
                }
            }
            Err(e) => panic!("Could not open file {} : {}\n", path, e),
        }
    }

    fn load_file_async(&mut self) -> bool {
        for (_, f) in self.files_to_load.iter_mut() {
            if f.1.is_ready() && f.2.is_none() {
                match f.1.read_binary() {
                    Ok(buf) => {
                        f.2 = Some(buf);
                    }
                    Err(e) => panic!("could not load async file {} : {}", f.0, e),
                }
            }
        }
        return true;
    }

    pub fn is_file_ready(&mut self, id: usize) -> bool {
        self.load_file_async();
        if let Some(f) = self.files_to_load.get(&id) {
            return f.2.is_some();
        }
        false
    }

    pub fn get_file_content(&mut self, id: usize) -> Vec<u8> {
        let mut f = self.files_to_load.remove(&id).unwrap();
        f.2.take().unwrap()
    }
}

fn open_file(filename: &str) -> Result<uni_app::fs::File, std::io::Error> {
    let ffilename =
        if cfg!(not(target_arch = "wasm32")) && &filename[0..1] != "/" && &filename[1..2] != ":" {
            "static/".to_owned() + filename
        } else {
            filename.to_owned()
        };
    uni_app::fs::FileSystem::open(&ffilename)
}