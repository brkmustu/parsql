//! Key binding definitions

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn is_quit_key(key: &KeyEvent) -> bool {
    matches!(
        key,
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
    )
}