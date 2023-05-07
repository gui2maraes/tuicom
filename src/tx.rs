use serialport::SerialPort;
use std::io;
pub struct Tx {
    pub display: String,
    output_mode: OutputMode,
    buffer: Vec<u8>,
}
impl Tx {
    pub fn new() -> Self {
        Self {
            output_mode: OutputMode::Ascii,
            buffer: Vec::new(),
            display: String::new(),
        }
    }
    pub fn with_cursor<'a>(&'a mut self, cursor: char) -> WithCursor<'a> {
        WithCursor::new(&mut self.display, cursor)
    }
    pub fn switch_hex(&mut self) {
        self.output_mode = match self.output_mode {
            OutputMode::Ascii => OutputMode::Hex(ByteBuffer { buf: None }),
            OutputMode::Hex(_) => OutputMode::Ascii,
        };
        self.display.clear();

        if let OutputMode::Ascii = self.output_mode {
            for &b in &self.buffer {
                push_ascii(&mut self.display, b);
            }
        } else {
            for &b in &self.buffer {
                push_hex(&mut self.display, b);
            }
        }
    }
    pub fn send(&mut self, ch: char, port: &mut dyn SerialPort) -> Result<(), io::Error> {
        match &mut self.output_mode {
            OutputMode::Ascii => {
                self.send_char(ch, port)?;
                self.buffer
                    .extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes());
                self.display.push(ch);
            }
            OutputMode::Hex(byte_buf) => {
                if let Some(d) = ch.to_digit(16) {
                    let mut complete = false;
                    if let Some(b) = byte_buf.push(d as u8) {
                        port.write_all(&[b])?;
                        self.buffer.push(b);
                        complete = true;
                    }
                    self.display.push(ch);
                    if complete {
                        self.display.push(' ');
                    }
                }
            }
        }
        Ok(())
    }
    pub fn send_but_show(
        &mut self,
        ch: char,
        show: &str,
        port: &mut dyn SerialPort,
    ) -> Result<(), io::Error> {
        match &mut self.output_mode {
            OutputMode::Ascii => {
                self.send_char(ch, port)?;
                self.buffer
                    .extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes());
                self.display.push_str(show);
            }
            OutputMode::Hex(byte_buf) => {
                if let Some(d) = ch.to_digit(16) {
                    let mut complete = false;
                    if let Some(b) = byte_buf.push(d as u8) {
                        port.write_all(&[b])?;
                        self.buffer.push(b);
                        complete = true;
                    }
                    self.display.push(ch);
                    if complete {
                        self.display.push(' ');
                    }
                }
            }
        }
        Ok(())
    }
    fn send_char(&mut self, ch: char, port: &mut dyn SerialPort) -> Result<(), io::Error> {
        let mut buf = [0; 4];
        let bytes = ch.encode_utf8(&mut buf).as_bytes();
        port.write_all(bytes)?;

        Ok(())
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
enum OutputMode {
    Ascii,
    Hex(ByteBuffer),
}

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
