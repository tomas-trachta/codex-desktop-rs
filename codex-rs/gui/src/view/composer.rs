use codex_protocol::openai_models::ReasoningEffort;
use iced::Alignment;
use iced::Element;
use iced::Length;
use iced::keyboard;
use iced::keyboard::key::Named;
use iced::widget::Id;
use iced::widget::Row;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::space;
use iced::widget::text;
use iced::widget::text_editor;
use iced::widget::text_editor::Binding;
use iced::widget::text_editor::KeyPress;

use super::icon;
use super::icon::Icon;
use super::style;
use super::transcript::CONTENT_WIDTH;
use crate::app::App;
use crate::app::COMPOSER_ID;
use crate::app::Message;
use crate::approval_modes;
use crate::shortcuts;

const MIN_HEIGHT: f32 = 44.0;
const MAX_HEIGHT: f32 = 240.0;
const ACTION_SIZE: f32 = 32.0;
const DEFAULT_APPROVAL_LABEL: &str = "Ask for approval";

pub fn view(app: &App) -> Element<'_, Message> {
    let card = container(column![editor(app), toolbar(app)])
        .padding([6, 6])
        .width(Length::Fill)
        .style(style::composer);

    container(container(card).max_width(CONTENT_WIDTH))
        .center_x(Length::Fill)
        .padding(iced::Padding {
            top: 8.0,
            right: 32.0,
            bottom: 20.0,
            left: 32.0,
        })
        .into()
}

fn editor(app: &App) -> Element<'_, Message> {
    let context = KeyContext {
        shortcuts: shortcuts::Context {
            has_pending_approval: app.has_pending_approval(),
        },
        is_empty: app.composer.text().trim().is_empty(),
    };

    text_editor(&app.composer)
        .id(Id::new(COMPOSER_ID))
        .placeholder(placeholder(app))
        .on_action(Message::ComposerEdited)
        .key_binding(move |key_press| key_binding(key_press, &context))
        .style(style::composer_editor)
        .padding([8, 10])
        .size(15)
        .min_height(MIN_HEIGHT)
        .max_height(MAX_HEIGHT)
        .into()
}

fn toolbar(app: &App) -> Element<'_, Message> {
    let mut bar = Row::new().spacing(8).align_y(Alignment::Center);
    if let Some(label) = approval_mode_label(app) {
        bar = bar.push(approval_button(label));
    }

    bar = bar.push(space::horizontal());
    if let Some(model) = model_button(app) {
        bar = bar.push(model);
    }

    container(bar.push(action_button(app)))
        .padding([4, 4])
        .into()
}

fn approval_mode_label(app: &App) -> Option<&'static str> {
    let settings = app.active.as_ref().map(|thread| &thread.settings);

    approval_modes::current(settings, &app.overrides)
        .map(|mode| mode.label)
        .or_else(|| settings.and_then(approval_modes::fallback_label))
        .or(Some(DEFAULT_APPROVAL_LABEL))
}

fn approval_button<'a>(label: &'static str) -> Element<'a, Message> {
    let content = row![
        icon::icon(Icon::Shield, 15.0, style::TEXT_MUTED),
        text(label).size(13).color(style::TEXT_MUTED),
        icon::icon(Icon::ChevronDown, 14.0, style::TEXT_MUTED),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    button(content)
        .padding([4, 8])
        .style(style::icon_button)
        .on_press(Message::ToggleApprovalMenu)
        .into()
}

fn model_button(app: &App) -> Option<Element<'_, Message>> {
    let model = app.effective_model_label()?;
    let mut label = row![text(display_model(&model)).size(13)]
        .spacing(4)
        .align_y(Alignment::Center);

    if let Some(effort) = app.effective_effort() {
        label = label.push(
            text(display_effort(&effort))
                .size(13)
                .color(style::TEXT_MUTED),
        );
    }
    label = label.push(icon::icon(Icon::ChevronDown, 14.0, style::TEXT_MUTED));

    let button = button(label)
        .padding([4, 8])
        .style(style::icon_button)
        .on_press(Message::ToggleModelMenu);

    Some(button.into())
}

fn action_button(app: &App) -> Element<'_, Message> {
    if app.is_turn_running() {
        return circle_button(Icon::Stop, style::stop_button, Some(Message::Interrupt));
    }

    let on_press = app.can_submit().then_some(Message::Submit);
    circle_button(Icon::ArrowUp, style::send_button, on_press)
}

fn circle_button<'a>(
    kind: Icon,
    style: fn(&iced::Theme, button::Status) -> button::Style,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    button(container(icon::icon(kind, 16.0, iced::Color::WHITE)).center(Length::Fill))
        .width(ACTION_SIZE)
        .height(ACTION_SIZE)
        .padding(0)
        .style(style)
        .on_press_maybe(on_press)
        .into()
}

fn placeholder(app: &App) -> &'static str {
    if app.is_read_only() {
        return "This thread is open in another Codex window. Close it there to continue here.";
    }

    "Work with Codex"
}

pub fn display_model(name: &str) -> String {
    if !name.contains('-') || name.contains(' ') {
        return name.to_string();
    }

    let mut parts = name.split('-');
    let Some(family) = parts.next() else {
        return name.to_string();
    };

    let mut words = Vec::new();
    match (family, parts.next()) {
        (family, Some(version)) if family.eq_ignore_ascii_case("gpt") => {
            words.push(format!("GPT-{version}"));
        }
        (family, version) => {
            words.push(capitalize(family));
            words.extend(version.map(capitalize));
        }
    }
    words.extend(parts.map(capitalize));

    words.join(" ")
}

pub fn display_effort(effort: &ReasoningEffort) -> String {
    match effort {
        ReasoningEffort::XHigh => "Extra High".to_string(),
        other => format!("{other:?}"),
    }
}

fn capitalize(word: &str) -> String {
    let mut characters = word.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => String::new(),
    }
}

struct KeyContext {
    shortcuts: shortcuts::Context,
    is_empty: bool,
}

fn key_binding(key_press: KeyPress, context: &KeyContext) -> Option<Binding<Message>> {
    if let Some(message) =
        shortcuts::approval(&key_press.key, key_press.modifiers, &context.shortcuts)
    {
        return Some(Binding::Custom(message));
    }

    match key_press.key {
        keyboard::Key::Named(Named::Enter) if !key_press.modifiers.shift() => {
            Some(Binding::Custom(Message::Submit))
        }
        keyboard::Key::Named(Named::ArrowUp) if context.is_empty => {
            Some(Binding::Custom(Message::RestorePrompt))
        }
        _ => Binding::from_key_press(key_press),
    }
}
