use ratatui::{buffer::Buffer, layout::Rect};

use crate::ui::app_ui_state::AppUIState;
// TODO: AppUIState should NOT be mut
pub trait Renderable {
    fn area_rect(&self, area: Rect) -> Rect;
    fn render(&self, area: Rect, buf: &mut Buffer, state: &mut AppUIState);
}
