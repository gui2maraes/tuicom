use crate::tx::Tx;
use crossterm::event::{Event, KeyCode, KeyEvent};
use serialport::SerialPort;
use std::fmt::Write;
use std::io;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub enum Control {
    Continue,
    Exit,
}
impl Control {
    pub fn exit(self) -> bool {
        matches!(self, Self::Exit)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Normal,
    Insert,
    Config,
    WannaQuit,
}

impl Mode {
    pub fn insert(self) -> bool {
        matches!(self, Self::Insert)
    }
    pub fn normal(self) -> bool {
        matches!(self, Self::Normal)
    }
    pub fn config(self) -> bool {
        matches!(self, Self::Config)
    }
    pub fn wanna_quit(self) -> bool {
        matches!(self, Self::WannaQuit)
    }
}

pub struct App {
    pub serial: Box<dyn SerialPort>,
    pub tx: Tx,
    pub rx_buf: Vec<u8>,
    pub rx_out: String,
    pub is_hex: bool,
    pub connected: bool,
    pub mode: Mode,
    cursor: Cursor,
}

impl App {
    pub fn new(serial: Box<dyn SerialPort>) -> Self {
        Self {
            serial,
            tx: Tx::new(),
            rx_buf: Vec::new(),
            rx_out: String::new(),
            is_hex: false,
            connected: true,
            mode: Mode::Normal,
            cursor: Cursor::Normal,
        }
    }
    pub fn update(&mut self, event: Option<Event>) -> Result<Control, io::Error> {
        let mut ctl = Control::Continue;
        let mut key_pressed = false;
        if let Some(e) = event {
            match e {
                Event::Key(k) => {
                    ctl = self.handle_key(k)?;
                    key_pressed = true
                }
                _ => (),
            }
        }
        self.get_data()?;
        self.cursor.update(key_pressed);
        Ok(ctl)
    }
    fn handle_key(&mut self, key: KeyEvent) -> Result<Control, io::Error> {
        use KeyCode as K;
        match self.mode {
            Mode::Insert => match key.code {
                K::Esc => self.leave_insert(),
                K::Char(c) => self.tx.send(c, self.serial.as_mut())?,
                K::Tab => self.tx.send_but_show('\t', "    ", self.serial.as_mut())?,
                K::Enter => self.tx.send('\n', self.serial.as_mut())?,
                _ => (),
            },
            Mode::Normal => match key.code {
                K::Esc | KeyCode::Char('q') => self.mode = Mode::WannaQuit,
                K::Char('i') => self.enter_insert(),
                K::Char('h') => self.switch_rx_hex(),
                K::Char('H') => self.tx.switch_hex(),
                _ => (),
            },
            Mode::WannaQuit => match key.code {
                K::Esc | K::Char('n' | 'q') => self.mode = Mode::Normal,
                K::Char('y') => return Ok(Control::Exit),
                _ => (),
            },
            _ => (),
        }
        Ok(Control::Continue)
    }

    pub fn switch_rx_hex(&mut self) {
        self.is_hex = !self.is_hex;
        self.rx_out.clear();
        if self.is_hex {
            for &b in &self.rx_buf {
                push_hex(&mut self.rx_out, b);
            }
        } else {
            for &b in &self.rx_buf {
                push_ascii(&mut self.rx_out, b);
            }
        }
    }
    pub fn enter_insert(&mut self) {
        self.mode = Mode::Insert;
        self.cursor = Cursor::insert();
    }
    pub fn leave_insert(&mut self) {
        self.mode = Mode::Normal;
        self.cursor = Cursor::normal();
    }
    pub fn get_data(&mut self) -> Result<(), io::Error> {
        use serialport::ErrorKind;
        let bytes = match self.serial.bytes_to_read() {
            Ok(b) => b as usize,
            Err(err) if err.kind() == ErrorKind::NoDevice => {
                self.connected = false;
                return Ok(());
            }
            Err(err) => return Err(err.into()),
        };
        let len = self.rx_buf.len();
        self.rx_buf.resize(len + bytes, 0);
        self.serial.read_exact(&mut self.rx_buf[len..])?;
        if self.is_hex {
            for &b in &self.rx_buf[len..] {
                push_hex(&mut self.rx_out, b);
            }
        } else {
            for &b in &self.rx_buf[len..] {
                push_ascii(&mut self.rx_out, b);
            }
        }
        Ok(())
    }
    pub fn cursor(&self) -> char {
        self.cursor.cursor()
    }
}

fn push_hex(s: &mut String, byte: u8) {
    write!(s, "{byte:02X} ").unwrap();
}
fn push_ascii(s: &mut String, byte: u8) {
    if byte == b'\t' {
        s.push_str("    ");
    } else {
        s.push(byte.into());
    }
}

enum Cursor {
    Normal,
    Insert {
        on: bool,
        timer: Duration,
        last: Instant,
    },
}

impl Cursor {
    const BLINK_SPEED: Duration = Duration::from_millis(500);
    pub const INSERT: char = '▎';
    pub const NORMAL: char = '▉';

    fn normal() -> Self {
        Self::Normal
    }
    fn insert() -> Self {
        Self::Insert {
            on: true,
            timer: Duration::ZERO,
            last: Instant::now(),
        }
    }

    fn update(&mut self, key_pressed: bool) {
        if let Self::Insert { on, timer, last } = self {
            if key_pressed {
                *on = true;
                *timer = Duration::ZERO;
                return;
            }
            let now = Instant::now();
            *timer += now - *last;
            *last = now;
            if *timer > Self::BLINK_SPEED {
                *on = !*on;
                *timer -= Self::BLINK_SPEED;
            }
        }
    }
    fn cursor(&self) -> char {
        match *self {
            Self::Normal => Self::NORMAL,
            Self::Insert { on: true, .. } => Self::INSERT,
            Self::Insert { on: false, .. } => ' ',
        }
    }
}
