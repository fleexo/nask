use crate::Rect;
use crate::ui::nask_center_banner::{banner_height, create_banner};
use crate::ui::nask_center_input::create_input_box;
use crate::ui::renderable_trait::Renderable;

pub struct NaskCenter {
    pub center_rect: Rect,
    banner: Box<dyn Renderable>,
    input_box: Box<dyn Renderable>,
}

pub fn calculate_nask_center_rect(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);

    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) * 2 / 5,
        width: w,
        height: h,
    }
}

pub const INPUT_HEIGHT: u16 = 5;
const CENTER_WIDTH: u16 = 70;

impl NaskCenter {
    pub fn new(area: Rect) -> Self {
        let title_height = banner_height();

        let center_height = title_height + INPUT_HEIGHT;
        let nask_center_rect = calculate_nask_center_rect(area, CENTER_WIDTH, center_height);

        Self {
            center_rect: nask_center_rect,
            banner: create_banner(),
            input_box: create_input_box(INPUT_HEIGHT),
        }
    }
    pub fn get_renderables(&self) -> [&dyn Renderable; 2] {
        [self.banner.as_ref(), self.input_box.as_ref()]
    }
}
