use argh::FromArgs;

#[derive(FromArgs)]
/// A tui serial terminal
pub struct Args {
    #[argh(option, short = 'b')]
    /// baud rate
    pub baud: u32,

    /// serial port to connect to
    #[argh(option, short = 'p')]
    pub port: String,
}
