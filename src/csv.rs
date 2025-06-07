use std::collections::HashMap;
use std::fs;

#[derive(Copy, Clone, Debug)]
pub enum Delimiter {
    SingleQuote,
    DoubleQuote,
}

#[derive(Debug)]
pub struct CSV {
    pub filename: String,
    pub separator: char,
    pub delimiter: Delimiter,
    pub rows: Vec<Vec<String>>,
}

pub struct CSVMap {
    pub filename: String,
    pub separator: char,
    pub delimiter: Delimiter,
    pub map: HashMap<(usize, usize), String>,
}

pub fn read_csv_file(filename: &str, separator0: char, string_delimiter: Delimiter) -> CSV {
    let data0 = fs::read_to_string(filename).unwrap();
    let data = data0.as_bytes();

    let delimiter = match string_delimiter {
        Delimiter::SingleQuote => '\'' as u8,
        Delimiter::DoubleQuote => '"' as u8,
    };

    let separator = separator0 as u8;

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();

    let mut i = 0;
    let mut start_idx = 0;

    let mut reading_delimiter = false;

    while i < data.len() {
        let c = data[i];

        if c == b'\n' {
            if !reading_delimiter {
                let value = str::from_utf8(&data[start_idx..i]).unwrap();

                current_row.push(value.to_string());

                rows.push(current_row);
                current_row = Vec::new();

                start_idx = i + 1;
            }
        } else if c == b'\\' {
            if reading_delimiter {
                if i + 1 < data.len() {
                    let next_c = data[i + 1];
                    if next_c == delimiter {
                        i += 1;
                    }
                }
            }
        } else if c == delimiter {
            if i + 1 < data.len() {
                let next_c = data[i + 1];
                if next_c != delimiter {
                    reading_delimiter = !reading_delimiter;
                } else {
                    current_row.push(String::new());
                    i += 1;
                }
            }
        } else if c == separator {
            let value = str::from_utf8(&data[start_idx..i]).unwrap();

            current_row.push(value.to_string());

            start_idx = i + 1;
        }

        i += 1;
    }

    if start_idx + 1 < i {
        let value = str::from_utf8(&data[start_idx..i]).unwrap();

        current_row.push(value.to_string());

        rows.push(current_row);
    }

    CSV {
        filename: filename.to_owned(),
        separator: separator0,
        delimiter: string_delimiter,
        rows,
    }
}

pub fn read_csv_file_as_hashmap(
    filename: &str,
    separator: char,
    string_delimiter: Delimiter,
) -> CSVMap {
    let csv = read_csv_file(filename, separator, string_delimiter);

    let mut map: HashMap<(usize, usize), String> = HashMap::new();

    for (i, row) in csv.rows.iter().enumerate() {
        for (j, col) in row.iter().enumerate() {
            if col.len() > 0 {
                let _ = map.insert((i, j), col.to_string());
            }
        }
    }

    CSVMap {
        filename: csv.filename,
        separator: csv.separator,
        delimiter: csv.delimiter,
        map,
    }
}
