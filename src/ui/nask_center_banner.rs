use crate::ui::app_ui_state::AppUIState;
use crate::ui::renderable_trait::Renderable;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
};
use std::sync::OnceLock;

const GAP: u16 = 2;
const ASCII_ART_NASK_BANNER: &str = include_str!("../../assets/nask.txt");
static BANNER_HEIGHT: OnceLock<u16> = OnceLock::new();

pub fn banner_height() -> u16 {
    *BANNER_HEIGHT.get_or_init(|| ASCII_ART_NASK_BANNER.lines().count() as u16 + GAP)
}

pub struct Banner;
impl Banner {
    fn new() -> Self {
        Self {}
    }
}

pub fn create_banner() -> Box<dyn Renderable> {
    Box::new(Banner::new())
}

impl Renderable for Banner {
    fn area_rect(&self, area: Rect) -> Rect {
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: banner_height(),
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer, _state: &mut AppUIState) {
        let banner = Paragraph::new(ASCII_ART_NASK_BANNER)
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::BOLD));

        banner.render(area, buf);
    }
}
