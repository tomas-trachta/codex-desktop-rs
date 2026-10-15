use iced::Color;
use iced::widget::Svg;
use iced::widget::svg;

use super::style;

const FOLDER: &str =
    r#"<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>"#;
const NEW_CHAT: &str = r#"<path d="M12 4H6a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-6"/><path d="M18.4 2.6a2 2 0 0 1 2.9 2.9L12 14.8l-4 1 1-4z"/>"#;
const ARROW_UP: &str = r#"<path d="M12 19V5"/><path d="M5 12l7-7 7 7"/>"#;
const STOP: &str = r#"<rect x="7" y="7" width="10" height="10" rx="2" fill="black"/>"#;
const COPY: &str = r#"<rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>"#;
const SHIELD: &str =
    r#"<path d="M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6z"/><path d="M9 12l2 2 4-4"/>"#;
const REFRESH: &str = r#"<path d="M21 12a9 9 0 1 1-3-6.7L21 8"/><path d="M21 3v5h-5"/>"#;
const TERMINAL: &str = r#"<path d="M4 17l6-6-6-6"/><path d="M12 19h8"/>"#;
const FILE: &str = r#"<path d="M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><path d="M14 3v6h6"/>"#;
const TOOL: &str = r#"<path d="M14.7 6.3a4 4 0 0 0-5.4 5.4L3 18l3 3 6.3-6.3a4 4 0 0 0 5.4-5.4l-2.6 2.6-2.4-.6-.6-2.4z"/>"#;

const PENCIL: &str =
    r#"<path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z"/>"#;
const ARCHIVE: &str = r#"<rect x="3" y="4" width="18" height="4" rx="1"/><path d="M5 8v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8"/><path d="M10 12h4"/>"#;
const SEARCH: &str = r#"<circle cx="11" cy="11" r="7"/><path d="M20 20l-3.5-3.5"/>"#;
const SIDEBAR: &str = r#"<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16"/>"#;

const CHEVRON_DOWN: &str = r#"<path d="M6 9l6 6 6-6"/>"#;
const CHECK: &str = r#"<path d="M5 12l5 5 9-11"/>"#;
const PLUS: &str = r#"<path d="M12 5v14"/><path d="M5 12h14"/>"#;

#[derive(Debug, Clone, Copy)]
pub enum Icon {
    Folder,
    NewChat,
    ArrowUp,
    Stop,
    Copy,
    Shield,
    Refresh,
    Terminal,
    File,
    Tool,
    Pencil,
    Archive,
    Search,
    Sidebar,
    ChevronDown,
    Check,
    Plus,
}

pub fn icon<'a>(icon: Icon, size: f32, color: Color) -> Svg<'a> {
    svg(svg::Handle::from_memory(document(icon).into_bytes()))
        .width(size)
        .height(size)
        .style(style::tinted(color))
}

fn document(icon: Icon) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">{}</svg>"#,
        paths(icon)
    )
}

fn paths(icon: Icon) -> &'static str {
    match icon {
        Icon::Folder => FOLDER,
        Icon::NewChat => NEW_CHAT,
        Icon::ArrowUp => ARROW_UP,
        Icon::Stop => STOP,
        Icon::Copy => COPY,
        Icon::Shield => SHIELD,
        Icon::Refresh => REFRESH,
        Icon::Terminal => TERMINAL,
        Icon::File => FILE,
        Icon::Tool => TOOL,
        Icon::Pencil => PENCIL,
        Icon::Archive => ARCHIVE,
        Icon::Search => SEARCH,
        Icon::Sidebar => SIDEBAR,
        Icon::ChevronDown => CHEVRON_DOWN,
        Icon::Check => CHECK,
        Icon::Plus => PLUS,
    }
}
