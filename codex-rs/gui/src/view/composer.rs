use iced::Alignment;
use iced::Element;
use iced::Length;
use iced::keyboard;
use iced::keyboard::key::Named;
use iced::widget::button;
use iced::widget::container;
use iced::widget::row;
use iced::widget::text;
use iced::widget::text_editor;
use iced::widget::text_editor::Binding;
use iced::widget::text_editor::KeyPress;

use crate::app::App;
use crate::app::Message;

const COMPOSER_WIDTH: f32 = 860.0;
const MIN_HEIGHT: f32 = 52.0;
const MAX_HEIGHT: f32 = 220.0;

pub fn view(app: &App) -> Element<'_, Message> {
    let editor = text_editor(&app.composer)
        .placeholder(placeholder(app))
        .on_action(Message::ComposerEdited)
        .key_binding(key_binding)
        .padding(12)
        .size(14)
        .min_height(MIN_HEIGHT)
        .max_height(MAX_HEIGHT);

    let controls = row![editor, action_button(app)]
        .spacing(8)
        .align_y(Alignment::End);

    container(container(controls).max_width(COMPOSER_WIDTH))
        .center_x(Length::Fill)
        .padding([12, 32])
        .into()
}

fn placeholder(app: &App) -> &'static str {
    if app.is_read_only() {
        return "This thread is open in another Codex window. Close it there to continue here.";
    }

    "Ask Codex anything. Enter to send, Shift+Enter for a new line."
}

fn action_button(app: &App) -> Element<'_, Message> {
    if app.is_turn_running() {
        return button(text("Stop").size(14))
            .padding([10, 18])
            .style(button::danger)
            .on_press(Message::Interrupt)
            .into();
    }

    button(text("Send").size(14))
        .padding([10, 18])
        .style(button::primary)
        .on_press_maybe(app.can_submit().then_some(Message::Submit))
        .into()
}

fn key_binding(key_press: KeyPress) -> Option<Binding<Message>> {
    let is_enter = key_press.key == keyboard::Key::Named(Named::Enter);

    if is_enter && !key_press.modifiers.shift() {
        return Some(Binding::Custom(Message::Submit));
    }

    Binding::from_key_press(key_press)
}
