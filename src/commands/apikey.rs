use tinyharness_lib::config::{load_settings, save_settings};

use crate::style::*;

pub fn execute_set(key: &str) {
    let mut settings = load_settings();
    settings.ollama_api_key = Some(key.to_string());
    save_settings(&settings);
    println!("{}Ollama API key saved.{}", BOLD, RESET);
}

pub fn execute_show() {
    let settings = load_settings();
    match &settings.ollama_api_key {
        Some(key) => {
            let masked = if key.len() > 8 {
                format!("{}...{}", &key[..4], &key[key.len() - 4..])
            } else {
                "****".to_string()
            };
            println!("{}Ollama API key:{} {}", BOLD, RESET, masked);
        }
        None => println!(
            "{}No Ollama API key set.{} Use {}/apikey <key>{} to set one.",
            ORANGE, RESET, BLUE, RESET
        ),
    }
}

pub fn execute_clear() {
    let mut settings = load_settings();
    settings.ollama_api_key = None;
    save_settings(&settings);
    println!("{}Ollama API key cleared.{}", BOLD, RESET);
}
