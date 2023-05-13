use crate::screen::{Rx, Tx};
use crossterm::event::{Event, KeyCode, KeyEvent};
use serialport::SerialPort;
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

#[derive(Debug, Clone)]
pub enum Mode {
    Normal,
    Insert,
    Config,
    WannaQuit,
    BaudInput(String),
}

impl Mode {
    pub fn is_insert(&self) -> bool {
        matches!(self, Self::Insert)
    }
    pub fn is_normal(&self) -> bool {
        matches!(self, Self::Normal)
    }
    pub fn is_config(&self) -> bool {
        matches!(self, Self::Config)
    }
    pub fn wanna_quit(&self) -> bool {
        matches!(self, Self::WannaQuit)
    }
}

pub struct App {
    pub serial: Box<dyn SerialPort>,
    pub tx: Tx,
    pub rx: Rx,
    pub mode: Mode,
    cursor: Cursor,
    connected: bool,
}

impl App {
    pub fn new(serial: Box<dyn SerialPort>) -> Self {
        Self {
            serial,
            tx: Tx::new(),
            rx: Rx::new(),
            mode: Mode::Normal,
            cursor: Cursor::Normal,
            connected: true,
        }
    }
    pub fn update(&mut self, event: Option<Event>) -> Result<Control, io::Error> {
        let mut ctl = Control::Continue;
        let mut key_pressed = false;
        if let Some(e) = event {
            match e {
                Event::Key(k) => {
                    ctl = is_connected(self.handle_key(k), &mut self.connected)?;
                    key_pressed = true
                }
                _ => (),
            }
        }
        is_connected(
            self.rx
                .recv(self.serial.as_mut())
                .map(|_| Control::Continue),
            &mut self.connected,
        )?;
        self.cursor.update(key_pressed);
        Ok(ctl)
    }
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    fn handle_key(&mut self, key: KeyEvent) -> Result<Control, io::Error> {
        use KeyCode as K;
        match &mut self.mode {
            Mode::Insert => match key.code {
                K::Esc => self.leave_insert(),
                K::Char(c) => {
                    let mut buf = [0; 4];
                    for &b in c.encode_utf8(&mut buf).as_bytes() {
                        self.tx.send(b, self.serial.as_mut())?;
                    }
                }
                K::Tab => self.tx.send(b'\t', self.serial.as_mut())?,
                K::Enter => self.tx.send(b'\n', self.serial.as_mut())?,
                _ => (),
            },
            Mode::Normal => match key.code {
                K::Esc | KeyCode::Char('q') => self.mode = Mode::WannaQuit,
                K::Char('i') => self.enter_insert(),
                K::Char('h') => self.rx.display.switch_hex(),
                K::Char('H') => self.tx.display.switch_hex(),
                K::Char('l') => self.tx.lf_crlf = !self.tx.lf_crlf,
                K::Char('c') => self.rx.display.clear(),
                K::Char('C') => self.tx.display.clear(),
                K::Char('b') => self.mode = Mode::BaudInput(String::with_capacity(8)),

                _ => (),
            },
            Mode::WannaQuit => match key.code {
                K::Esc | K::Char('n' | 'q') => self.mode = Mode::Normal,
                K::Char('y') => return Ok(Control::Exit),
                _ => (),
            },
            Mode::BaudInput(buf) => match key.code {
                K::Esc => self.mode = Mode::Normal,
                K::Char(c @ '0'..='9') => buf.push(c),
                K::Enter => {
                    self.serial.set_baud_rate(buf.parse().unwrap())?;
                    self.mode = Mode::Normal;
                }
                K::Backspace => {
                    buf.pop();
                }
                _ => (),
            },
            _ => (),
        }
        Ok(Control::Continue)
    }

    pub fn enter_insert(&mut self) {
        self.mode = Mode::Insert;
        self.cursor = Cursor::insert();
    }
    pub fn leave_insert(&mut self) {
        self.mode = Mode::Normal;
        self.cursor = Cursor::normal();
    }
    pub fn cursor(&self) -> char {
        self.cursor.cursor()
    }
}

fn is_connected(res: io::Result<Control>, connected: &mut bool) -> io::Result<Control> {
    match res {
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            *connected = false;
            return Ok(Control::Continue);
        }
        res => res,
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
