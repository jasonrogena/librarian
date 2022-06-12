use crate::error;
use std::io::Read;
use std::path::Path;

#[cfg(test)]
mod tests;
// since you only need a few bytes in the start of the file to get
// the magic number used to determine the file type
#[allow(dead_code)]
const MAX_FILE_READ_BYTES: u64 = 10240;

#[derive(Debug)]
pub struct File<'a> {
    path: &'a Path,
}

impl<'a> File<'a> {
    #[allow(dead_code)]
    pub fn new(path: &'a Path) -> File {
        File { path }
    }

    #[allow(dead_code)]
    pub fn get_mime_type(&'a self) -> Result<String, error::Error> {
        let mut file_obj = match std::fs::File::open(self.path) {
            Ok(f) => f,
            Err(e) => {
                return Err(error::Error::new(format!(
                    "Unable to open file '{:?}' to test its type: {}",
                    self.path, e
                )))
            }
        };

        let mut buf = Vec::with_capacity(MAX_FILE_READ_BYTES as usize);
        if let Err(e) = file_obj
            .by_ref()
            .take(MAX_FILE_READ_BYTES)
            .read_to_end(&mut buf)
        {
            return Err(error::Error::new(format!(
                "Unable to read file '{:?}' to test its type: {}",
                self.path, e
            )));
        }

        let mime_type = tree_magic::from_u8(buf.as_slice());

        Ok(mime_type)
    }
}
