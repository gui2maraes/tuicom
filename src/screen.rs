use serialport::SerialPort;
use std::io;
pub struct Tx {
    pub display: Display,
    pub lf_crlf: bool,
}
impl Tx {
    pub fn new() -> Self {
        Self {
            display: Display::new(),
            lf_crlf: false,
        }
    }
    pub fn with_cursor<'a>(&'a mut self, cursor: char) -> WithCursor<'a> {
        WithCursor::new(&mut self.display.show, cursor)
    }
    pub fn send(&mut self, ch: u8, port: &mut dyn SerialPort) -> Result<(), io::Error> {
        let Some(c) = self.display.push_char(ch) else {return Ok(())};

        let res = if self.lf_crlf && c == b'\n' {
            port.write_all(b"\n\r")
        } else {
            port.write_all(&[c])
        };
        if let Err(_) = &res {
            self.display.pop();
        }
        res
    }
}

pub struct Rx {
    pub display: Display,
    recv_buf: Vec<u8>,
}

impl Rx {
    pub fn new() -> Self {
        Self {
            display: Display::new(),
            recv_buf: Vec::new(),
        }
    }
    pub fn with_cursor<'a>(&'a mut self, cursor: char) -> WithCursor<'a> {
        WithCursor::new(&mut self.display.show, cursor)
    }
    pub fn recv(&mut self, port: &mut dyn SerialPort) -> Result<(), io::Error> {
        let bytes = port.bytes_to_read()? as usize;
        self.recv_buf.resize(bytes as usize, 0);
        port.read_exact(&mut self.recv_buf[..])?;
        for &b in &self.recv_buf {
            self.display.push_byte(b);
        }

        Ok(())
    }
}

pub struct Display {
    buffer: Vec<u8>,
    show: String,
    display_mode: DisplayMode,
}

impl Display {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            show: String::new(),
            display_mode: DisplayMode::Ascii,
        }
    }
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.show.clear();
        self.display_mode.clear();
    }
    pub fn switch_hex(&mut self) {
        self.display_mode = match self.display_mode {
            DisplayMode::Ascii => DisplayMode::Hex(ByteBuffer { buf: None }),
            DisplayMode::Hex(_) => DisplayMode::Ascii,
        };
        self.show.clear();

        if let DisplayMode::Ascii = self.display_mode {
            for &b in &self.buffer {
                push_ascii(&mut self.show, b);
            }
        } else {
            for &b in &self.buffer {
                push_hex(&mut self.show, b);
            }
        }
    }
    pub fn pop(&mut self) -> Option<u8> {
        match self.display_mode {
            DisplayMode::Hex(_) => {
                for _ in 0..3 {
                    self.show.pop();
                }
            }
            DisplayMode::Ascii => {
                self.show.pop();
            }
        }
        self.buffer.pop()
    }
    // pushes and ASCII digit to buffer and display, accounting for HEX mode
    pub fn push_char(&mut self, ch: u8) -> Option<u8> {
        let mut out = None;
        match &mut self.display_mode {
            DisplayMode::Ascii => {
                self.buffer.push(ch);
                push_ascii(&mut self.show, ch);
                out = Some(ch);
            }
            DisplayMode::Hex(byte_buf) => {
                if let Some(c) = (ch as char).to_digit(16) {
                    let mut complete = false;
                    if let Some(b) = byte_buf.push(c as u8) {
                        self.buffer.push(b);
                        out = Some(b);
                        complete = true;
                    }
                    self.show.push(ch.to_ascii_uppercase().into());
                    if complete {
                        self.show.push(' ');
                    }
                }
            }
        }
        out
    }
    pub fn push_byte(&mut self, byte: u8) {
        self.buffer.push(byte);
        self.show_push(byte);
    }
    fn show_push(&mut self, byte: u8) {
        match &mut self.display_mode {
            DisplayMode::Ascii => push_ascii(&mut self.show, byte),
            DisplayMode::Hex(_) => push_hex(&mut self.show, byte),
        }
    }
}
fn push_hex(s: &mut String, byte: u8) {
    use std::fmt::Write;
    write!(s, "{byte:02X} ").unwrap();
}
fn push_ascii(s: &mut String, byte: u8) {
    if byte == b'\t' {
        s.push_str("    ");
    } else {
        s.push(byte.into());
    }
}

#[derive(Debug, Clone, Copy)]
enum DisplayMode {
    Ascii,
    Hex(ByteBuffer),
}

impl DisplayMode {
    fn clear(&mut self) {
        if let DisplayMode::Hex(b) = self {
            b.buf = None;
        }
    }
}

/// Struct for buffering nibbles to output bytes
#[derive(Debug, Clone, Copy)]
struct ByteBuffer {
    buf: Option<u8>,
}
impl ByteBuffer {
    fn push(&mut self, hex: u8) -> Option<u8> {
        if let Some(b) = self.buf {
            self.buf = None;
            Some((b << 4) | hex)
        } else {
            self.buf = Some(hex);
            None
        }
    }
}

pub struct WithCursor<'a> {
    s: &'a mut String,
}

impl<'a> WithCursor<'a> {
    pub fn new(s: &'a mut String, cursor: char) -> Self {
        s.push(cursor);
        Self { s }
    }
}
impl<'a> Drop for WithCursor<'a> {
    fn drop(&mut self) {
        self.s.pop();
    }
}

impl<'a> AsRef<String> for WithCursor<'a> {
    fn as_ref(&self) -> &String {
        self.s
    }
}
impl<'a> AsRef<str> for WithCursor<'a> {
    fn as_ref(&self) -> &str {
        self.s
    }
}
impl<'a> AsMut<String> for WithCursor<'a> {
    fn as_mut(&mut self) -> &mut String {
        self.s
    }
}
impl<'a> AsMut<str> for WithCursor<'a> {
    fn as_mut(&mut self) -> &mut str {
        self.s
    }
}
