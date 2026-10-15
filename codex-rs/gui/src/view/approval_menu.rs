use iced::Alignment;
use iced::Element;
use iced::Length;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::space;
use iced::widget::text;

use super::icon;
use super::icon::Icon;
use super::style;
use super::transcript::CONTENT_WIDTH;
use crate::app::App;
use crate::app::Message;
use crate::approval_modes;
use crate::approval_modes::ApprovalMode;

const MENU_WIDTH: f32 = 380.0;

pub fn view(app: &App) -> Element<'_, Message> {
    let settings = app.active.as_ref().map(|thread| &thread.settings);
    let selected = approval_modes::current(settings, &app.overrides).map(|mode| mode.id);

    let modes = column(
        approval_modes::modes()
            .into_iter()
            .map(|mode| mode_row(selected == Some(mode.id), mode)),
    )
    .spacing(2);

    let card = container(column![section_label(), modes].spacing(6))
        .padding([10, 10])
        .width(MENU_WIDTH)
        .style(style::dialog);

    container(
        container(row![card, space::horizontal()])
            .max_width(CONTENT_WIDTH)
            .padding(iced::Padding::ZERO.bottom(4)),
    )
    .center_x(Length::Fill)
    .padding([0, 32])
    .into()
}

fn section_label<'a>() -> Element<'a, Message> {
    container(text("Approvals").size(12).color(style::TEXT_MUTED))
        .padding([2, 8])
        .into()
}

fn mode_row<'a>(is_selected: bool, mode: ApprovalMode) -> Element<'a, Message> {
    let check = if is_selected {
        Element::from(icon::icon(Icon::Check, 15.0, style::ACCENT))
    } else {
        Element::from(space().width(15))
    };

    let label = column![
        text(mode.label).size(14),
        text(mode.description).size(12).color(style::TEXT_MUTED),
    ]
    .spacing(2)
    .width(Length::Fill);

    button(row![label, check].spacing(8).align_y(Alignment::Center))
        .width(Length::Fill)
        .padding([6, 8])
        .style(style::row_button(false))
        .on_press(Message::SelectApprovalMode(mode.id))
        .into()
}
