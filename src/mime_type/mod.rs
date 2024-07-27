use std::io::Read;
use std::path::Path;

#[cfg(test)]
mod tests;
// since you only need a few bytes in the start of the file to get
// the magic number used to determine the file type
#[allow(dead_code)]
const MAX_FILE_READ_BYTES: u64 = 10240;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An IO error was thrown while trying to determine the MIME type of a file")]
    Io(#[from] std::io::Error),
}

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
    pub fn get_mime_type(&'a self) -> Result<String, Error> {
        let mut file_obj = std::fs::File::open(self.path)?;

        let mut buf = Vec::with_capacity(MAX_FILE_READ_BYTES as usize);
        file_obj
            .by_ref()
            .take(MAX_FILE_READ_BYTES)
            .read_to_end(&mut buf)?;

        let mime_type = tree_magic::from_u8(buf.as_slice());

        Ok(mime_type)
    }
}
