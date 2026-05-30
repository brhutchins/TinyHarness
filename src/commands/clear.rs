use std::io::Write;

use tinyharness_ui::output::Output;

use tinyharness_ui::style::CLEAR_SCREEN;

pub fn execute(out: &mut Output) {
    let _ = write!(out, "{CLEAR_SCREEN}");
    let _ = out.flush();
}
