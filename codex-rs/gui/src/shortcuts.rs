use iced::keyboard::Event;
use iced::keyboard::Key;
use iced::keyboard::Modifiers;
use iced::keyboard::key::Code;
use iced::keyboard::key::Named;
use iced::keyboard::key::Physical;

use iced::event::Status;
use iced::window;

use crate::app::Message;
use codex_gui::approval::Decision;

pub struct Context {
    pub has_pending_approval: bool,
}

pub fn global(event: Event, context: &Context) -> Option<Message> {
    let Event::KeyPressed {
        key,
        physical_key,
        modifiers,
        ..
    } = event
    else {
        return None;
    };

    approval(&key, modifiers, context).or_else(|| command(&key, physical_key, modifiers))
}

pub fn approval(key: &Key, modifiers: Modifiers, context: &Context) -> Option<Message> {
    if !context.has_pending_approval || !modifiers.is_empty() {
        return None;
    }

    (*key == Key::Named(Named::Enter)).then_some(Message::Decide(Decision::Accept))
}

pub fn copy(event: iced::Event, _status: Status, _window: window::Id) -> Option<Message> {
    let iced::Event::Keyboard(Event::KeyPressed {
        physical_key,
        modifiers,
        ..
    }) = event
    else {
        return None;
    };

    (modifiers.command() && physical_key == Physical::Code(Code::KeyC))
        .then_some(Message::CleanClipboard)
}

pub fn escape(event: iced::Event, _status: Status, _window: window::Id) -> Option<Message> {
    let iced::Event::Keyboard(Event::KeyPressed { key, .. }) = event else {
        return None;
    };

    (key == Key::Named(Named::Escape)).then_some(Message::Escape)
}

fn command(key: &Key, physical_key: Physical, modifiers: Modifiers) -> Option<Message> {
    if !modifiers.command() {
        return None;
    }

    if let Some(slot) = recent_slot(physical_key) {
        return Some(Message::OpenRecent(slot));
    }

    match key {
        Key::Named(Named::Tab) if modifiers.shift() => Some(Message::CycleThread(-1)),
        Key::Named(Named::Tab) => Some(Message::CycleThread(1)),
        _ => letter_command(key.to_latin(physical_key)?, modifiers),
    }
}

fn letter_command(letter: char, modifiers: Modifiers) -> Option<Message> {
    match (letter, modifiers.shift(), modifiers.alt()) {
        ('n', false, false) => Some(Message::NewChat),
        ('o', false, false) => Some(Message::AddProject),
        ('b', false, false) => Some(Message::ToggleSidebar),
        ('a', true, false) => Some(Message::ArchiveThread),
        ('m', true, false) => Some(Message::ToggleModelMenu),
        ('r', false, true) => Some(Message::StartRename),
        _ => None,
    }
}

fn recent_slot(physical_key: Physical) -> Option<usize> {
    let Physical::Code(code) = physical_key else {
        return None;
    };

    let slot = match code {
        Code::Digit1 | Code::Numpad1 => 0,
        Code::Digit2 | Code::Numpad2 => 1,
        Code::Digit3 | Code::Numpad3 => 2,
        Code::Digit4 | Code::Numpad4 => 3,
        Code::Digit5 | Code::Numpad5 => 4,
        Code::Digit6 | Code::Numpad6 => 5,
        Code::Digit7 | Code::Numpad7 => 6,
        Code::Digit8 | Code::Numpad8 => 7,
        Code::Digit9 | Code::Numpad9 => 8,
        _ => return None,
    };

    Some(slot)
}
