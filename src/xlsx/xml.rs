use std::collections::HashMap;

#[derive(Debug)]
pub struct Loc {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub enum XMLError {
    Expecting { what: &'static str, loc: Loc },
    EndOfFile { when: &'static str },
    Invalid { what: &'static str, loc: Loc },
    InternalError { what: &'static str },
}

impl std::fmt::Display for XMLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XMLError::Expecting { what, loc } => {
                write!(f, "{}:{}: expecting {what}", loc.line, loc.column)
            }
            XMLError::EndOfFile { when } => {
                write!(f, "end of file when {when}")
            }
            XMLError::Invalid { what, loc } => {
                write!(f, "{}:{}: invalid xml when {what}", loc.line, loc.column)
            }
            XMLError::InternalError { what } => {
                write!(f, "{what}")
            }
        }
    }
}

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
    column: usize,
    line: usize,
}

impl Reader {
    fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            cursor: 0,
            column: 1,
            line: 1,
        }
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

            self.advance();
        }
    }

    // Advances without checking eob
    fn advance(&mut self) {
        let c = self.data[self.cursor];

        if c == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        self.cursor += 1;
    }

    // Advances without checking lines and eob
    fn raw_advance_n(&mut self, n: usize) {
        self.cursor += n;
        self.column += n;
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
        self.advance();

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

    fn get_loc(&self) -> Loc {
        Loc {
            line: self.line,
            column: self.column,
        }
    }
}

#[derive(Debug)]
pub enum Component {
    Comment(String),
    Declaration {
        version: String,
        encoding: String,
        standalone: bool,
    },
    Element {
        name: String,
        attributes: HashMap<String, String>,
        children: Vec<Component>,
    },
    Text {
        value: String,
        is_cdata: bool,
    },

    // Used for conditional sections and DTD, it just saves the raw data
    Other(String),
}

impl Component {
    pub fn children(&self) -> Option<&Vec<Component>> {
        match self {
            Component::Element { children, .. } => Some(&children),
            _ => None,
        }
    }

    pub fn children_unchecked(&self) -> &Vec<Component> {
        match self {
            Component::Element { children, .. } => &children,
            _ => unreachable!(),
        }
    }

    pub fn filter_elements<'a>(
        &'a self,
        elem_name: &'a str,
    ) -> Box<dyn Iterator<Item = &'a Component> + 'a> {
        match self {
            Component::Element { children, .. } => {
                Box::new(children.iter().filter(move |item| match item {
                    Component::Element { name, .. } => name.as_str() == elem_name,
                    _ => false,
                }))
            }
            _ => Box::new(std::iter::empty()),
        }
    }

    pub fn find_element(&self, elem_name: &str) -> Option<&Component> {
        match self {
            Component::Element { children, .. } => children.iter().find(move |item| match item {
                Component::Element { name, .. } => name.as_str() == elem_name,
                _ => false,
            }),
            _ => None,
        }
    }
}

#[inline]
fn is_digit(c: u8) -> bool {
    b'0' <= c && c < b'9'
}

#[inline]
fn is_start_name_char(c: u8) -> bool {
    c > 128 || is_alpha(c) || c == b':' || c == b'_'
}

#[inline]
fn is_name_char(c: u8) -> bool {
    is_start_name_char(c) || is_digit(c) || c == b'-' || c == b'.'
}

fn parse_name(reader: &mut Reader) -> Result<String, XMLError> {
    if reader.eob() {
        return Err(XMLError::EndOfFile {
            when: "parsing a name",
        });
    }

    let start = reader.cursor;

    let c = reader.read_byte();

    if !is_start_name_char(c) {
        return Err(XMLError::Expecting {
            what: "':', '_', or a letter",
            loc: reader.get_loc(),
        });
    }

    while !reader.eob() {
        let c = reader.peek_byte();

        if is_white_space(c) {
            break;
        }

        if !is_name_char(c) {
            break;
        }

        reader.advance();
    }

    let v = str::from_utf8(reader.data[start..reader.cursor].into());

    Ok(v.unwrap().to_string())
}

static ENTITIES: [(&[u8], u8); 5] = [
    (b"&quot;", b'"'),
    (b"&amp;", b'&'),
    (b"&apos;", b'\''),
    (b"&lt;", b'<'),
    (b"&gt;", b'>'),
];

fn parse_text(reader: &mut Reader, end_marker: &[u8]) -> Result<String, XMLError> {
    if end_marker.len() == 0 {
        return Err(XMLError::EndOfFile {
            when: "parsing a text",
        });
    }

    let mut result = Vec::new();

    while !reader.eob() {
        if reader.sequece_match(end_marker) {
            let v = str::from_utf8(&result);

            return Ok(v.unwrap().to_string());
        }

        let b = reader.peek_byte();

        if b == b'&' {
            let mut found = false;

            for (value, replacement) in ENTITIES {
                if reader.sequece_match(value) {
                    result.push(replacement);

                    reader.raw_advance_n(value.len());
                    found = true;
                    break;
                }
            }

            if found {
                continue;
            }
            // NOTE: Maybe return an error if entity was not found
        }

        result.push(b);

        reader.advance();
    }

    Err(XMLError::EndOfFile {
        when: "parsing a text the marker was not found",
    })
}

fn parse_attribute(reader: &mut Reader) -> Result<(String, String), XMLError> {
    let name = parse_name(reader)?;

    reader.skip_white_spaces();

    if !reader.sequece_match(b"=") {
        return Err(XMLError::Expecting {
            what: "'='",
            loc: reader.get_loc(),
        });
    }

    reader.raw_advance_n(1);

    reader.skip_white_spaces();

    if reader.eob() {
        return Err(XMLError::EndOfFile {
            when: "parsing an attribute",
        });
    }

    let c = reader.read_byte();

    if c != b'"' && c != b'\'' {
        return Err(XMLError::Expecting {
            what: "'\"' or `'`",
            loc: reader.get_loc(),
        });
    }

    let value = parse_text(reader, &[c])?;

    reader.raw_advance_n(1);

    Ok((name, value))
}

// -- Document
pub struct Document {
    pub children: Vec<Component>,
}

impl Document {
    pub fn from_data(data: Vec<u8>) -> Result<Document, XMLError> {
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
    ) -> Result<(), XMLError> {
        let is_element = match c {
            Component::Element { .. } => true,
            _ => false,
        };

        if is_open && is_element {
            stack.push(c);

            return Ok(());
        }

        if let Some(e) = stack.as_mut_slice().last_mut() {
            match e {
                Component::Element { children, .. } => children.push(c),
                _ => {
                    return Err(XMLError::InternalError {
                        what: "component in stack is not an element!",
                    });
                }
            }
        } else {
            self.children.push(c);
        }

        Ok(())
    }

    fn parse(&mut self, data: Vec<u8>) -> Result<(), XMLError> {
        let mut reader = Reader::new(data);

        let mut stack: Vec<Component> = Vec::new();

        while !reader.eob() {
            reader.skip_white_spaces();

            if reader.sequece_match(b"<?") {
                reader.raw_advance_n(2);

                if !reader.sequece_match(b"xml") {
                    return Err(XMLError::Invalid {
                        what: "invalid declaration, expecting 'xml'",
                        loc: reader.get_loc(),
                    });
                }

                if self.children.len() > 0 {
                    return Err(XMLError::Invalid {
                        what: "it must have only one xml declaration and it must be the first element in file",
                        loc: reader.get_loc(),
                    });
                }

                reader.raw_advance_n(3);

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
                            return Err(XMLError::EndOfFile {
                                when: "reading tag and its attributes",
                            });
                        }
                        (b'?', b'>') => {
                            reader.raw_advance_n(2);
                            break;
                        }
                        _ => {
                            let loc0 = reader.get_loc();

                            let (name, value) = parse_attribute(&mut reader)?;

                            if attr_count == 0 {
                                if name != "version" {
                                    return Err(XMLError::Expecting {
                                        what: "'version' attribute in xml declation",
                                        loc: loc0,
                                    });
                                }

                                version = value;
                            } else if attr_count == 1 {
                                if name == "encoding" {
                                    encoding = value;
                                } else if name == "standalone" {
                                    standalone = value.as_str() == "yes";
                                    attr_count += 1;
                                } else {
                                    return Err(XMLError::Expecting {
                                        what:
                                            "'encoding' or 'standalone' attribute in xml declation",
                                        loc: loc0,
                                    });
                                }
                            } else if attr_count == 2 {
                                if name != "standalone" {
                                    return Err(XMLError::Expecting {
                                        what: "'standalone' attribute in xml declation",
                                        loc: loc0,
                                    });
                                }
                                standalone = value.as_str() == "yes";
                            } else {
                                return Err(XMLError::Invalid {
                                    what: "too many attributes in xml declation",
                                    loc: loc0,
                                });
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
                reader.raw_advance_n(4);

                let text = parse_text(&mut reader, b"-->")?;

                reader.raw_advance_n(3);

                self.add_component(Component::Comment(text), false, &mut stack)?;
            } else if reader.sequece_match(b"<![CDATA[") {
                reader.raw_advance_n(9);

                let text = parse_text(&mut reader, b"]]>")?;

                reader.raw_advance_n(3);

                self.add_component(
                    Component::Text {
                        value: text,
                        is_cdata: true,
                    },
                    false,
                    &mut stack,
                )?;
            } else if reader.sequece_match(b"<!") {
                reader.raw_advance_n(2);

                let text = parse_text(&mut reader, b">")?;

                reader.raw_advance_n(1);

                self.add_component(Component::Other(text), false, &mut stack)?;
            } else if reader.sequece_match(b"</") {
                let loc0 = reader.get_loc();

                reader.raw_advance_n(2);

                let name = parse_name(&mut reader)?;

                reader.skip_white_spaces();

                if reader.eob() {
                    return Err(XMLError::EndOfFile {
                        when: "reading a closing tag",
                    });
                }

                let c = reader.peek_byte();

                if c != b'>' {
                    return Err(XMLError::Expecting {
                        what: "'>' to be closing the tag end",
                        loc: reader.get_loc(),
                    });
                }

                reader.advance();

                let removed_element;

                if let Some(e) = stack.pop() {
                    match e {
                        Component::Element { name: ref n, .. } => {
                            if *n != name {
                                return Err(XMLError::Invalid {
                                    what: "unmatched tag end",
                                    loc: loc0,
                                });
                            }
                        }
                        _ => {
                            return Err(XMLError::InternalError {
                                what: "component in stack is not an element!",
                            });
                        }
                    }

                    removed_element = e;
                } else {
                    return Err(XMLError::Invalid {
                        what: "unexpected tag end found",
                        loc: loc0,
                    });
                }

                if stack.len() > 0 {
                    self.add_component(removed_element, false, &mut stack)?;
                } else {
                    self.children.push(removed_element);
                }
            } else if reader.sequece_match(b"<") {
                reader.raw_advance_n(1);

                let mut attributes = HashMap::new();
                let name = parse_name(&mut reader)?;

                loop {
                    reader.skip_white_spaces();

                    match reader.peek_2bytes() {
                        (0, 0) => {
                            return Err(XMLError::EndOfFile {
                                when: "reading tag closing or its attributes",
                            });
                        }
                        (b'>', _) => {
                            reader.raw_advance_n(1);

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
                            reader.raw_advance_n(2);

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
                            let (name, value) = parse_attribute(&mut reader)?;
                            let _ = attributes.insert(name, value);
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
