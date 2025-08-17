use std::collections::HashMap;
use std::str::FromStr;

pub mod xml;
pub mod zip;

#[inline]
fn is_digit(c: u8) -> bool {
    b'0' <= c && c < b'9'
}

#[inline]
fn is_alpha(c: u8) -> bool {
    (b'a' <= c && c <= b'z') || (b'A' <= c && c <= b'Z')
}

fn base26_letters_to_int(v: &str) -> usize {
    let mut result = 0usize;

    for c in v.bytes() {
        if is_alpha(c) {
            result = result * 26 + 1 + (c - b'A') as usize;
        } else {
            return 0;
        }
    }

    result - 1
}

fn cell_pos_to_tuple(cell_pos: &str) -> Option<(usize, usize)> {
    let mut digit_idx = -1;

    for (i, c) in cell_pos.bytes().enumerate() {
        if is_digit(c) {
            digit_idx = i as isize;
            break;
        }

        if !is_alpha(c) {
            return None;
        }
    }

    if digit_idx < 1 {
        return None;
    }

    let (letters, digits) = cell_pos.split_at(digit_idx as usize);

    let row = usize::from_str(digits).unwrap() - 1;
    let col = base26_letters_to_int(letters);

    Some((row, col))
}

fn element_by_name(item: &xml::Component, elem_name: &str) -> bool {
    match item {
        xml::Component::Element { name, .. } => name.as_str() == elem_name,
        _ => false,
    }
}

fn get_text(item: &xml::Component) -> Option<String> {
    match item {
        xml::Component::Text { value, .. } => Some(value.clone()),
        _ => None,
    }
}

fn read_shared_strings(content: Vec<u8>) -> Vec<String> {
    let doc = xml::Document::from_data(content).unwrap();

    let mut shared_strings = Vec::new();

    let sst = doc
        .children
        .iter()
        .find(|e| element_by_name(e, "sst"))
        .unwrap();

    for si in sst.filter_elements("si") {
        let text = 'si_txt: {
            if let Some(t) = si.find_element("t") {
                if let Some(txt) = t.children_unchecked().iter().find_map(get_text) {
                    break 'si_txt txt;
                }
            } else {
                if let Some(r) = si.find_element("r") {
                    if let Some(t) = r.find_element("t") {
                        if let Some(txt) = t.children_unchecked().iter().find_map(get_text) {
                            break 'si_txt txt;
                        }
                    }
                }
            }

            String::new()
        };

        shared_strings.push(text);
    }

    shared_strings
}

fn read_cells(content: Vec<u8>, shared_strings: Vec<String>) -> HashMap<(usize, usize), String> {
    let doc = xml::Document::from_data(content).unwrap();

    let worksheet = doc
        .children
        .iter()
        .find(|e| element_by_name(e, "worksheet"))
        .unwrap();

    let mut map = HashMap::new();

    for sheet_data in worksheet.filter_elements("sheetData") {
        for row in sheet_data.filter_elements("row") {
            for c in row.filter_elements("c") {
                if let Some(v) = c.find_element("v") {
                    match c {
                        xml::Component::Element { attributes, .. } => {
                            if attributes.contains_key("r") && attributes.contains_key("t") {
                                let text =
                                    v.children_unchecked().iter().find_map(get_text).unwrap_or(String::new());

                                let cell_value = if let Some(t) = attributes.get("t") {
                                    if t == "n" || t == "str" {
                                        text.clone()
                                    } else if t == "b" {
                                        if text == "1" {
                                            String::from("true")
                                        } else {
                                            String::from("false")
                                        }
                                    } else if t == "s" {
                                        let index = usize::from_str(text.as_str()).unwrap();

                                        if index < shared_strings.len() {
                                            shared_strings[index].clone()
                                        } else {
                                            String::from("???")
                                        }
                                    } else {
                                        String::from("??")
                                    }
                                } else {
                                    String::from("?")
                                };

                                if let Some(r) = attributes.get("r") {
                                    if let Some(tpos) = cell_pos_to_tuple(r.as_str()) {
                                        let _ = map.insert(tpos, cell_value);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    map
}

pub struct XLSXMap {
    pub filename: String,
    pub worksheet: String,
    pub map: HashMap<(usize, usize), String>,
}

pub fn read_xlsx_file_as_hashmap(
    filename: &str,
    worksheet_name: Option<&str>,
) -> Result<XLSXMap, ()> {
    let zip = zip::Zip::from_file(filename).unwrap();

    let files = zip.extract_files().map_err(|e| {
        eprintln!("{:?}", e);
    })?;

    let shared_strings = files
        .iter()
        .find(|f| f.name.as_str() == "xl/sharedStrings.xml")
        .map(|f| &f.content);

    let ss = if let Some(s) = shared_strings {
        read_shared_strings(s.as_str().into())
    } else {
        Vec::new()
    };

    let mut ws_name = String::new();

    let worksheet = if let Some(ws) = worksheet_name {
        ws_name = format!("xl/worksheets/{ws}.xml");

        files.iter().find(|f| f.name == ws_name).map(|f| &f.content)
    } else if let Some(f) = files.iter().find(|f| f.name.starts_with("xl/worksheets/")) {
        ws_name = f.name.clone();

        Some(&f.content)
    } else {
        None
    };

    if let Some(sheet) = worksheet {
        let map = read_cells(sheet.as_str().into(), ss);

        Ok(XLSXMap {
            filename: filename.to_string(),
            worksheet: ws_name,
            map,
        })
    } else {
        eprintln!("not able to read {} contents", filename);

        Err(())
    }
}
