/*
struct Loc {
    line: usize,
    column: usize,
}

enum XMLError {
    Expecting { what: &str, loc: Loc },
    EndOfFile { when: &str },
    Invalid { what: &str, loc: Loc },
}
*/

fn is_utf8_continuation(c: u8) -> bool {
    c & 0x80 != 0
}

fn is_white_space(c: u8) -> bool {
    if !is_utf8_continuation(c) {
        match c {
            b' ' | 0x0c | b'\n' | b'\r' | b'\t' | 0x0b => true,
            _ => false,
        }
    } else {
        false
    }
}

#[inline]
fn is_alpha(c: u8) -> bool {
    (b'a' <= c && c <= b'z') || (b'A' <= c && c <= b'Z')
}

struct Reader {
    data: Vec<u8>,
    cursor: usize,
}

impl Reader {
    fn new(data: Vec<u8>) -> Self {
        Self { data, cursor: 0 }
    }

    // End of bytes
    fn eob(&self) -> bool {
        self.cursor >= self.data.len()
    }

    fn skip_white_spaces(&mut self) {
        while !self.eob() {
            let c = self.data[self.cursor];

            if !is_white_space(c) {
                break;
            }

            self.cursor += 1;
        }
    }

    // Test if the next bytes match the input sequence
    fn sequece_match(&self, seq: &[u8]) -> bool {
        if seq.len() > self.data.len() - self.cursor {
            return false;
        }

        for (i, &c) in seq.iter().enumerate() {
            if c != self.data[self.cursor + i] {
                return false;
            }
        }

        true
    }

    fn read_byte(&mut self) -> u8 {
        self.cursor += 1;

        self.data[self.cursor - 1]
    }

    fn peek_byte(&mut self) -> u8 {
        self.data[self.cursor]
    }

    fn peek_2bytes(&mut self) -> (u8, u8) {
        let d = self.data.len() - self.cursor;

        if d >= 2 {
            (self.data[self.cursor], self.data[self.cursor + 1])
        } else if d == 1 {
            (self.data[self.cursor], 0)
        } else {
            (0, 0)
        }
    }
}

// -- Attribute
#[derive(Debug)]
struct Attribute {
    name: String,
    value: String,
}

impl Attribute {
    fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

#[derive(Debug)]
enum Component {
    Comment(String),
    Declaration {
        version: String,
        encoding: String,
        standalone: bool,
    },
    Element {
        name: String,
        attributes: Vec<Attribute>,
        children: Vec<Component>,
    },
    Text {
        value: String,
        is_cdata: bool,
    },

    // Used for conditional sections and DTD, it just saves the raw data
    Other(String),
}

#[inline]
fn is_start_name_char(c: u8) -> bool {
    c > 128 || is_alpha(c) || c == b':' || c == b'_'
}

#[inline]
fn is_name_char(c: u8) -> bool {
    is_start_name_char(c) || c == b'-' || c == b'.'
}

fn parse_name(reader: &mut Reader) -> Result<String, String> {
    if reader.eob() {
        return Err(String::from("parse_name: Reader EOB!"));
    }

    let start = reader.cursor;

    let c = reader.read_byte();

    if !is_start_name_char(c) {
        return Err(String::from("parse_name: Element name start not found!"));
    }

    while !reader.eob() {
        let c = reader.peek_byte();

        if is_white_space(c) {
            break;
        }

        if !is_name_char(c) {
            break;
        }

        reader.cursor += 1;
    }

    let v = str::from_utf8(reader.data[start..reader.cursor].into());

    Ok(v.unwrap().to_string())
}

fn parse_text(reader: &mut Reader, end_marker: &[u8]) -> Result<String, String> {
    if end_marker.len() == 0 {
        return Err(String::from("parse_text: Reader EOB!"));
    }

    let start = reader.cursor;

    // println!("marker: {}", end_marker[0] as char);
    while !reader.eob() {
        if reader.sequece_match(end_marker) {
            let v = str::from_utf8(reader.data[start..reader.cursor].into());

            return Ok(v.unwrap().to_string());
        }

        reader.cursor += 1;
    }

    Err(String::from("parse_text: Reader EOB 2!"))
}

fn parse_attribute(reader: &mut Reader) -> Result<Attribute, String> {
    let name = parse_name(reader)?;

    reader.skip_white_spaces();

    if !reader.sequece_match(b"=") {
        return Err(String::from("parse_attribute: expecting '='"));
    }

    reader.cursor += 1;

    reader.skip_white_spaces();

    if reader.eob() {
        return Err(String::from("parse_attribute: unexpected EOB!"));
    }

    let c = reader.read_byte();

    if c != b'"' && c != b'\'' {
        return Err(String::from("parse_attribute: expectin \" or '."));
    }

    let value = parse_text(reader, &[c])?;

    reader.cursor += 1;

    Ok(Attribute::new(name, value))
}

// -- Document
struct Document {
    children: Vec<Component>,
}

impl Document {
    pub fn from_data(data: Vec<u8>) -> Result<Document, String> {
        let mut doc = Document {
            children: Vec::new(),
        };

        doc.parse(data)?;

        Ok(doc)
    }

    pub fn print_components(&self) {
        for c in self.children.iter() {
            println!("{:?}", c);
        }
    }

    // Adds compoment to list of components, also including to an element children if
    // there is any in stack
    fn add_component(
        &mut self,
        c: Component,
        is_open: bool,
        stack: &mut Vec<Component>,
    ) -> Result<(), String> {
        let is_element = match c {
            Component::Element { .. } => true,
            _ => false,
        };

        if is_open && is_element {
            stack.push(c);

            return Ok(())
        }

        if let Some(e) = stack.as_mut_slice().last_mut() {
            match e {
                Component::Element { children, .. } => children.push(c),
                _ => {
                    return Err(String::from("parse: Component in stack is not an element!"));
                }
            }
        } else {
            self.children.push(c);
        }

        Ok(())
    }

    fn parse(&mut self, data: Vec<u8>) -> Result<(), String> {
        let mut reader = Reader::new(data);

        let mut stack: Vec<Component> = Vec::new();

        while !reader.eob() {
            reader.skip_white_spaces();

            if reader.sequece_match(b"<?") {
                reader.cursor += 2;

                if !reader.sequece_match(b"xml") {
                    return Err(String::from("parse: Invalid declaration!"));
                }

                reader.cursor += 3;

                if self.children.len() > 0 {
                    return Err(String::from(
                    "parse: There must only one declaration in xml file, and must be the first element!"
                ));
                }

                reader.skip_white_spaces();

                let mut version = String::from("1.0");
                let mut encoding = String::from("UTF-8");
                let mut standalone = true;

                // Must read version, encoding? and standalone? in this order
                let mut attr_count = 0;

                loop {
                    reader.skip_white_spaces();

                    match reader.peek_2bytes() {
                        (0, 0) => {
                            return Err(String::from("parse: Reader EOB 3!"));
                        }
                        (b'?', b'>') => {
                            reader.cursor += 2;
                            break;
                        }
                        _ => {
                            let attr = parse_attribute(&mut reader)?;

                            if attr_count == 0 {
                                if attr.name != "version" {
                                    return Err(String::from(
                                        "parse: expecting 'version' attribute in xml declation!",
                                    ));
                                }

                                version = attr.value;
                            } else if attr_count == 1 {
                                if attr.name == "encoding" {
                                    encoding = attr.value;
                                } else if attr.name == "standalone" {
                                    standalone = attr.value == "yes";
                                    attr_count += 1;
                                } else {
                                    return Err(String::from(
                                    "parse: expecting 'encoding' or 'standalone' attribute in xml declation!",
                                ));
                                }
                            } else if attr_count == 2 {
                                if attr.name != "standalone" {
                                    return Err(String::from(
                                        "parse: expecting 'standalone' attribute in xml declation!",
                                    ));
                                }
                                standalone = attr.value == "yes";
                            } else {
                                return Err(String::from(
                                    "parse: reading too many attributes in xml declation!",
                                ));
                            }

                            attr_count += 1;
                        }
                    }
                }

                self.add_component(
                    Component::Declaration {
                        version,
                        encoding,
                        standalone,
                    },
                    false,
                    &mut stack,
                )?;
            } else if reader.sequece_match(b"<!--") {
                reader.cursor += 4;

                let text = parse_text(&mut reader, b"-->")?;

                reader.cursor += 3;

                self.add_component(Component::Comment(text), false, &mut stack)?;
            } else if reader.sequece_match(b"<![CDATA[") {
                reader.cursor += 9;

                let text = parse_text(&mut reader, b"]]>")?;

                reader.cursor += 3;

                self.add_component(
                    Component::Text {
                        value: text,
                        is_cdata: true,
                    },
                    false,
                    &mut stack,
                )?;
            } else if reader.sequece_match(b"<!") {
                reader.cursor += 2;

                let text = parse_text(&mut reader, b">")?;

                reader.cursor += 1;

                self.add_component(Component::Other(text), false, &mut stack)?;
            } else if reader.sequece_match(b"</") {
                reader.cursor += 2;

                let name = parse_name(&mut reader)?;

                reader.skip_white_spaces();

                if reader.eob() {
                    return Err(String::from("parse: Reader EOB 2!"));
                }

                let c = reader.read_byte();

                if c != b'>' {
                    return Err(String::from("parse: Element tag end closenot found!"));
                }

                if let Some(e) = stack.pop() {
                    match e {
                        Component::Element { name: ref n, .. } => {
                            if *n != name {
                                return Err(String::from("parse: Unmatched tag end"));
                            }
                        }
                        _ => {
                            return Err(String::from(
                                "parse: Component in stack is not an element!",
                            ));
                        }
                    }

                    self.children.push(e);
                } else {
                    return Err(String::from("parse: Unexpected tag end found"));
                }
            } else if reader.sequece_match(b"<") {
                reader.cursor += 1;

                let mut attributes = Vec::new();
                let name = parse_name(&mut reader)?;

                loop {
                    reader.skip_white_spaces();

                    match reader.peek_2bytes() {
                        (0, 0) => {
                            return Err(String::from("parse: Reader EOB 3!"));
                        }
                        (b'>', _) => {
                            reader.cursor += 1;

                            self.add_component(
                                Component::Element {
                                    name,
                                    attributes,
                                    children: vec![],
                                },
                                true,
                                &mut stack,
                            )?;

                            break;
                        }
                        (b'/', b'>') => {
                            reader.cursor += 2;

                            self.add_component(
                                Component::Element {
                                    name,
                                    attributes,
                                    children: vec![],
                                },
                                false,
                                &mut stack,
                            )?;

                            break;
                        }
                        _ => {
                            let attr = parse_attribute(&mut reader)?;
                            attributes.push(attr);
                        }
                    }
                }
            } else if stack.len() > 0 {
                let text = parse_text(&mut reader, b"<")?;

                self.add_component(
                    Component::Text {
                        value: text,
                        is_cdata: false,
                    },
                    false,
                    &mut stack,
                )?;
            }
        }

        Ok(())
    }
}

fn main() {
    let data = b"
        <?xml version='1.1' encoding='UTF-16' standalone='yes' ?>
        <!ENTITY   rights \"All rights reserved\" >
        <!-- input -->
        <input />
        <tag abra=\"cadabra\">text</tag>
        <![CDATA[<greeting>Hello, world!</greeting>]]>
    ";

    println!("data size: {}", data.len());

    match Document::from_data(data.into()) {
        Ok(doc) => doc.print_components(),
        Err(err) => {
            println!("{err}");
        }
    }
}
