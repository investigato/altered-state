use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::marker::PhantomData;

pub type DiskStorageReader<T> = OxicodeIterator<T, BufReader<File>>;

/// Lazy iterator for oxicode-encoded, length-prefixed data
pub struct OxicodeIterator<T, R: Read> {
    reader: R,
    _phantom: PhantomData<T>,
}

impl<T> OxicodeIterator<T, BufReader<File>>
where
    T: oxicode::Decode<()>,
{
    /// Create a new iterator from a file path
    pub fn from_path(file_path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        Ok(Self {
            reader,
            _phantom: PhantomData,
        })
    }

    pub fn from_file(file: std::fs::File) -> Self {
        Self {
            reader: BufReader::new(file),
            _phantom: PhantomData,
        }
    }
}

impl<T, R: Read> OxicodeIterator<T, R>
where
    T: oxicode::Decode<()>,
{
    /// Create a new iterator from any reader
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            _phantom: PhantomData,
        }
    }
}

impl<T, R: Read> Iterator for OxicodeIterator<T, R>
where
    T: oxicode::Decode<()>,
{
    type Item = Result<T, Box<dyn Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Try to read length prefix
        let mut len_bytes = [0u8; 4];
        match self.reader.read_exact(&mut len_bytes) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Clean EOF - no more records
                return None;
            }
            Err(e) => return Some(Err(e.into())),
        }

        let len = u32::from_le_bytes(len_bytes) as usize;

        // Validate length to prevent excessive allocation
        // if len > 100_000_000 {
        //     // 100MB limit, adjust as needed
        //     return Some(Err(format!(
        //         "Item length {len} exceeds maximum allowed size"
        //     )
        //     .into()));
        // }

        // Read the exact amount of data for this item
        let mut data = vec![0u8; len];
        if let Err(e) = self.reader.read_exact(&mut data) {
            return Some(Err(format!("Failed to read {len} bytes: {e}").into()));
        }

        // Decode the item
        match oxicode::decode_from_slice::<T>(&data) {
            Ok((item, _)) => Some(Ok(item)),
            Err(e) => Some(Err(format!("Failed to decode item: {e:?}").into())),
        }
    }
}
