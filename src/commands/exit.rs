use std::io::Write;

use tinyharness_ui::output::Output;

use tinyharness_ui::style::*;

pub fn execute(out: &mut Output) {
    let _ = writeln!(out, "{ORANGE}Goodbye!{RESET}");
}
