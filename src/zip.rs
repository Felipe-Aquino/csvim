use std::convert::TryInto;
use std::fs;

mod inflate;

struct Reader {
    data: Vec<u8>,
    cursor: usize,
}

impl Reader {
    fn new(data: Vec<u8>) -> Self {
        Self { data, cursor: 0 }
    }

    fn find_prefix(&self, prefix: &[u8]) -> Option<usize> {
        for i in self.cursor..(self.data.len() - prefix.len()) {
            let mut found = true;

            for (j, v) in prefix.iter().enumerate() {
                if *v != self.data[i + j] {
                    found = false;
                    break;
                }
            }

            if found {
                return Some(i);
            }
        }

        None
    }

    // Finds the first occurence of the prefix in the data, starting from the end until
    // the cursor.
    fn find_prefix_rev(&self, prefix: &[u8]) -> Option<usize> {
        for i in (self.cursor..(self.data.len() - prefix.len())).rev() {
            let mut found = true;

            for (j, v) in prefix.iter().enumerate() {
                if *v != self.data[i + j] {
                    found = false;
                    break;
                }
            }

            if found {
                return Some(i);
            }
        }

        None
    }

    fn read_byte(&mut self) -> u8 {
        self.cursor += 1;

        self.data[self.cursor - 1]
    }

    fn read_u16(&mut self) -> Option<u16> {
        let r: Result<[u8; 2], _> = self.data[self.cursor..(self.cursor + 2)].try_into();

        match r {
            Ok(bytes) => {
                self.cursor += 2;

                Some(u16::from_le_bytes(bytes))
            }
            Err(e) => None,
        }
    }

    fn read_u32(&mut self) -> Option<u32> {
        let r: Result<[u8; 4], _> = self.data[self.cursor..(self.cursor + 4)].try_into();

        match r {
            Ok(bytes) => {
                self.cursor += 4;

                Some(u32::from_le_bytes(bytes))
            }
            Err(_) => None,
        }
    }

    fn peek_u32(&mut self) -> Option<u32> {
        let r: Result<[u8; 4], _> = self.data[self.cursor..(self.cursor + 4)].try_into();

        match r {
            Ok(bytes) => Some(u32::from_le_bytes(bytes)),
            Err(_) => None,
        }
    }

    fn read_string(&mut self, size: usize) -> Option<String> {
        let v = str::from_utf8(self.data[self.cursor..(self.cursor + size)].into());

        self.cursor += size;

        v.map(|d| d.to_string()).ok()
    }

    fn read_vec(&mut self, size: usize) -> Option<Vec<u8>> {
        if self.cursor + size >= self.data.len() {
            return None;
        }

        let v: Vec<u8> = self.data[self.cursor..(self.cursor + size)].to_owned();

        self.cursor += size;

        Some(v)
    }
}

#[derive(Debug)]
struct LocalFileHeader {
    // size
    // 4  Local file header signature. Must be 50 4B 03 04 (PK♥♦ or "PK\3\4").
    min_version: u16, // 2  Version needed to extract (minimum).
    flag: u16,        // 2  General purpose bit flag.
    compression: u16, // 2  Compression method; e.g. none = 0, DEFLATE = 8 (or "\0x08\0x00").
    // 2  File last modification time.
    // 2  File last modification date.
    crc32: u32,             // 4  CRC-32 of uncompressed data.
    compressed_size: u32,   // 4  Compressed size
    uncompressed_size: u32, // 4  Uncompressed size
    // 2  File name length (n).
    // 2  Extra field length (m).
    // n  File name
    // m  Extra field.
    data: Vec<u8>,
}

impl LocalFileHeader {
    fn from_reader(reader: &mut Reader) -> Option<Self> {
        let signature = reader.read_u32()?;

        if signature != 0x04034b50 {
            return None;
        }

        let min_version = reader.read_u16()?;
        let flag = reader.read_u16()?;
        let compression = reader.read_u16()?;

        reader.cursor += 4;

        // TODO: Test if data's CRC-32 matches
        let crc32 = reader.read_u32()?;
        let compressed_size = reader.read_u32()?;
        let uncompressed_size = reader.read_u32()?;

        let file_name_len = reader.read_u16()?;
        let extra_len = reader.read_u16()?;

        // let file_name = reader.read_string(file_name_len as usize).unwrap();

        reader.cursor += file_name_len as usize;
        reader.cursor += extra_len as usize;

        let data = if compression != 8 {
            reader.read_vec(compressed_size as usize)?
        } else {
            let content = reader.read_vec(compressed_size as usize)?;
            inflate::decompress(&content).ok()?
        };

        Some(Self {
            min_version,
            flag,
            compression,
            crc32,
            compressed_size,
            uncompressed_size,
            data,
        })
    }
}

#[derive(Debug)]
struct CentralDirectoryHeader {
    // off  size
    // 0       4   Central directory file header signature. Must be 50 4B 01 02.
    version: u16,     // 4       2   Version made by.
    min_version: u16, // 6       2   Version needed to extract (minimum).
    flag: u16,        // 8       2   General purpose bit flag.
    compression: u16, // 10      2   Compression method.
    // 12      2   File last modification time.
    // 14      2   File last modification date.
    crc32: u32,             // 16      4   CRC-32 of uncompressed data.
    compressed_size: u32,   // 20      4   Compressed size
    uncompressed_size: u32, // 24      4   Uncompressed size
    // 28      2   File name length (n).
    // 30      2   Extra field length (m).
    // 32      2   File comment length (k).
    // 34      2   Disk number where file starts
    // 36      2   Internal file attributes.
    // 38      4   External file attributes.
    offset: u32, // 42      4   Relative offset of local file header.
    file_name: String, // 46      n   File name.
                 // 46+n    m   Extra field.
                 // 46+n+m  k   File comment.
}

impl CentralDirectoryHeader {
    fn from_reader(reader: &mut Reader) -> Option<Self> {
        let signature = reader.read_u32()?;

        if signature != 0x02014b50 {
            return None;
        }

        let version = reader.read_u16()?;
        let min_version = reader.read_u16()?;
        let flag = reader.read_u16()?;
        let compression = reader.read_u16()?;

        reader.cursor += 4;

        let crc32 = reader.read_u32()?;
        let compressed_size = reader.read_u32()?;
        let uncompressed_size = reader.read_u32()?;

        let file_name_len = reader.read_u16()?;
        let extra_len = reader.read_u16()?;
        let comment_len = reader.read_u16()?;

        reader.cursor += 8;

        let offset = reader.read_u32()?;

        let file_name = reader.read_string(file_name_len as usize)?;

        reader.cursor += (extra_len + comment_len) as usize;

        Some(Self {
            version,
            min_version,
            flag,
            compression,
            crc32,
            compressed_size,
            uncompressed_size,
            offset,
            file_name,
        })
    }
}

#[derive(Debug)]
struct EndOfCentralDirectoryHeader {
    // off  size
    disk_number: u16,       // 0       2   Disk number where file starts
    disk_number_start: u16, // 2       2   Disk number where file starts
    num_central_directories_on_disk: u16, // 4       2   Number of central directory records on this disk
    total_num_central_directories: u16,   // 6       2   Total number of central directory records
    central_directory_size: u32,          // 8       4   Size of central directory (bytes)
    offset_to_start_of_central_directory: u32, // 12      4   Offset to start of central directory, relative to start of archive
                                               // 16      2   Comment length
                                               // 18      k   Comment
}

impl EndOfCentralDirectoryHeader {
    fn from_reader(reader: &mut Reader) -> Option<Self> {
        let signature = reader.read_u32()?;

        if signature != 0x06054b50 {
            return None;
        }

        let disk_number = reader.read_u16()?;
        let disk_number_start = reader.read_u16()?;
        let num_central_directories_on_disk = reader.read_u16()?;
        let total_num_central_directories = reader.read_u16()?;
        let central_directory_size = reader.read_u32()?;
        let offset_to_start_of_central_directory = reader.read_u32()?;
        let comment_len = reader.read_u16()?;

        reader.cursor += comment_len as usize;

        Some(Self {
            disk_number,
            disk_number_start,
            num_central_directories_on_disk,
            total_num_central_directories,
            central_directory_size,
            offset_to_start_of_central_directory,
        })
    }
}

pub struct Zip {
    eocd: EndOfCentralDirectoryHeader,
    central_directory_headers: Vec<CentralDirectoryHeader>,
    local_file_headers: Vec<LocalFileHeader>,
}

#[derive(Debug)]
pub struct ZipFile {
    name: String,
    content: String,
}

impl Zip {
    pub fn from_file(filepath: &str) -> Option<Zip> {
        let data = fs::read(filepath).ok()?;

        let mut reader = Reader::new(data);

        let pidx = reader.find_prefix_rev(&[0x50, 0x4b, 0x05, 0x06])?;

        reader.cursor = pidx;

        let eocd = EndOfCentralDirectoryHeader::from_reader(&mut reader)?;

        reader.cursor = eocd.offset_to_start_of_central_directory as usize;

        let mut central_directory_headers = Vec::new();
        let mut local_file_headers = Vec::new();

        for _ in 0..eocd.total_num_central_directories {
            let cdh = CentralDirectoryHeader::from_reader(&mut reader)?;

            let saved_cursor = reader.cursor;

            reader.cursor = cdh.offset as usize;
            let lfh = LocalFileHeader::from_reader(&mut reader)?;

            reader.cursor = saved_cursor;

            central_directory_headers.push(cdh);
            local_file_headers.push(lfh);
        }

        Some(Zip {
            eocd,
            central_directory_headers,
            local_file_headers,
        })
    }

    fn extract_files(&self) -> Result<Vec<ZipFile>, std::str::Utf8Error> {
        let mut files = Vec::new();

        for i in 0..self.central_directory_headers.len() {
            let name = self.central_directory_headers[i].file_name.clone();

            let lfh = &self.local_file_headers[i];
            let content = str::from_utf8(&lfh.data[..])?;

            files.push(ZipFile {
                name,
                content: content.to_string(),
            });
        }

        Ok(files)
    }
}

/*
fn main() {
    // let data = fs::read("file.xlsx").unwrap();
    // let data = fs::read("test2.zip").unwrap();

    let zip = Zip::from_file("file.xlsx").unwrap();

    let files = zip.extract_files().ok().unwrap();

    for file in files {
        println!("{:?}\n", file);
    }
}
*/
