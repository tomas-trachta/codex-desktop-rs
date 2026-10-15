use codex_app_server_protocol::Model;
use codex_protocol::openai_models::ReasoningEffort;
use iced::Alignment;
use iced::Element;
use iced::Length;
use iced::widget::Row;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::space;
use iced::widget::text;

use super::composer::display_effort;
use super::composer::display_model;
use super::icon;
use super::icon::Icon;
use super::style;
use super::transcript::CONTENT_WIDTH;
use crate::app::App;
use crate::app::Message;
use crate::app::supported_efforts;

const MENU_WIDTH: f32 = 340.0;

pub fn view(app: &App) -> Element<'_, Message> {
    let selected = app.effective_model().map(|model| model.model.as_str());

    let models = column(
        app.models
            .iter()
            .map(|model| model_row(model, selected == Some(model.model.as_str()))),
    )
    .spacing(2);

    let mut menu = column![section_label("Model"), models].spacing(6);
    if let Some(model) = app.effective_model() {
        menu = menu
            .push(section_label("Reasoning"))
            .push(effort_row(model, app.effective_effort()));
    }

    let card = container(menu)
        .padding([10, 10])
        .width(MENU_WIDTH)
        .style(style::dialog);

    container(
        container(row![space::horizontal(), card])
            .max_width(CONTENT_WIDTH)
            .padding(iced::Padding::ZERO.bottom(4)),
    )
    .center_x(Length::Fill)
    .padding([0, 32])
    .into()
}

fn section_label<'a>(label: &'a str) -> Element<'a, Message> {
    container(text(label).size(12).color(style::TEXT_MUTED))
        .padding([2, 8])
        .into()
}

fn model_row(model: &Model, is_selected: bool) -> Element<'_, Message> {
    let check = if is_selected {
        Element::from(icon::icon(Icon::Check, 15.0, style::ACCENT))
    } else {
        Element::from(space().width(15))
    };

    let label = column![
        text(display_model(&model.display_name)).size(14),
        text(model.description.as_str())
            .size(12)
            .color(style::TEXT_MUTED),
    ]
    .spacing(2)
    .width(Length::Fill);

    button(row![label, check].spacing(8).align_y(Alignment::Center))
        .width(Length::Fill)
        .padding([6, 8])
        .style(style::row_button(false))
        .on_press(Message::SelectModel(model.model.clone()))
        .into()
}

fn effort_row(model: &Model, selected: Option<ReasoningEffort>) -> Element<'_, Message> {
    let chips = supported_efforts(model)
        .into_iter()
        .map(|effort| effort_chip(effort.clone(), selected.as_ref() == Some(&effort)));

    container(Row::with_children(chips).spacing(4).wrap())
        .padding([0, 4])
        .into()
}

fn effort_chip<'a>(effort: ReasoningEffort, is_selected: bool) -> Element<'a, Message> {
    let style = if is_selected {
        style::primary_button
    } else {
        style::secondary_button
    };

    button(text(display_effort(&effort)).size(12))
        .padding([4, 10])
        .style(style)
        .on_press(Message::SelectEffort(effort))
        .into()
}
