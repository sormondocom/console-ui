use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

/// Simplified, cross-platform key abstraction over `crossterm::event::Event`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Up, Down, Left, Right,
    PageUp, PageDown,
    Home, End,
    Enter,
    Escape,
    Backspace,
    Delete,
    Tab,
    BackTab,
    F(u8),
    Ctrl(char),
    Alt(char),
    Resize(u16, u16),
    Unknown,
}

/// Block until the next key or resize event arrives.
pub fn read_key() -> std::io::Result<Key> {
    loop {
        let ev = event::read()?;
        if let Some(k) = translate(ev) {
            return Ok(k);
        }
    }
}

/// Non-blocking poll — returns `None` if no event arrives within `timeout`.
pub fn poll_key(timeout: Duration) -> std::io::Result<Option<Key>> {
    if event::poll(timeout)? {
        let ev = event::read()?;
        return Ok(translate(ev));
    }
    Ok(None)
}

fn translate(ev: Event) -> Option<Key> {
    match ev {
        Event::Key(KeyEvent { code, modifiers, kind: KeyEventKind::Press, .. }) => Some(translate_key(code, modifiers)),
        Event::Resize(cols, rows) => Some(Key::Resize(cols, rows)),
        _ => None,
    }
}

fn translate_key(code: KeyCode, modifiers: KeyModifiers) -> Key {
    let ctrl  = modifiers.contains(KeyModifiers::CONTROL);
    let alt   = modifiers.contains(KeyModifiers::ALT);

    match code {
        KeyCode::Char(c) if ctrl => Key::Ctrl(c),
        KeyCode::Char(c) if alt  => Key::Alt(c),
        KeyCode::Char(c)         => Key::Char(c),
        KeyCode::Up              => Key::Up,
        KeyCode::Down            => Key::Down,
        KeyCode::Left            => Key::Left,
        KeyCode::Right           => Key::Right,
        KeyCode::PageUp          => Key::PageUp,
        KeyCode::PageDown        => Key::PageDown,
        KeyCode::Home            => Key::Home,
        KeyCode::End             => Key::End,
        KeyCode::Enter           => Key::Enter,
        KeyCode::Esc             => Key::Escape,
        KeyCode::Backspace       => Key::Backspace,
        KeyCode::Delete          => Key::Delete,
        KeyCode::Tab             => Key::Tab,
        KeyCode::BackTab         => Key::BackTab,
        KeyCode::F(n)            => Key::F(n),
        _                        => Key::Unknown,
    }
}
