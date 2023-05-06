use phf::phf_map;

pub const SWITCH_HEX: char = 'h';
pub const INSERT: char = 'i';
pub const NORMAL: &str = "esc";
pub const QUIT: char = 'q';

pub static BINDINGS: phf::Map<&'static str, &'static str> = phf_map! {
    "h" => "hex output",
    "q" => "quit",
    "i" => "insert mode",
    "ESC" => "normal mode",
};
