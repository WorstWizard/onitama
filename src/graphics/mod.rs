use glam::{Vec2, Vec3};

pub mod board;
pub mod card;
pub mod piece;
pub mod renderer;

// const ANIM_TIME: f32 = 0.25;
pub type Color = Vec3;
#[derive(Clone, Copy)]
pub struct Rect {
    pub origin: Vec2,
    pub size: Vec2,
}
impl Rect {
    pub fn new(origin: Vec2, size: Vec2) -> Self {
        Rect { origin, size }
    }
    pub fn contains_point(&self, pos: Vec2) -> bool {
        pos.x >= self.origin.x
            && pos.x < self.origin.x + self.size.x
            && pos.y >= self.origin.y
            && pos.y < self.origin.y + self.size.y
    }
}
#[test]
fn contains_point() {
    use glam::vec2;
    let rect = Rect::new(vec2(1.0, 2.0), vec2(10.0, 20.0));
    assert!(rect.contains_point(vec2(5.0, 5.0)));
    assert!(!rect.contains_point(vec2(11.0, 5.0)));
}

mod colors {
    use super::Color;
    pub const BOARD_TILE: Color = Color::new(1.0, 1.0, 1.0);
    pub const BOARD_BG: Color = Color::new(0.5, 0.5, 0.5);
    pub const BOARD_HIGHLIGHT: Color = Color::new(1.0, 1.0, 0.0);
    pub const BOARD_RED_TEMPLE: Color = Color::new(200.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0);
    pub const BOARD_BLUE_TEMPLE: Color = Color::new(50.0 / 255.0, 50.0 / 255.0, 200.0 / 255.0);
    pub const CARD_BG: Color = Color::new(200.0 / 255.0, 200.0 / 255.0, 170.0 / 255.0);
    pub const CARD_TILE_BG: Color = Color::new(230.0 / 255.0, 230.0 / 255.0, 200.0 / 255.0);
    pub const CARD_TILE: Color = Color::new(130.0 / 255.0, 130.0 / 255.0, 100.0 / 255.0);
    pub const CARD_SELECTED: Color = Color::new(250.0 / 255.0, 250.0 / 255.0, 220.0 / 255.0);
    pub const CARD_CENTER: Color = Color::new(80.0 / 255.0, 80.0 / 255.0, 40.0 / 255.0);
    pub const PIECE_RED: Color = Color::new(1.0, 0.2, 0.2);
    pub const PIECE_BLUE: Color = Color::new(0.2, 0.2, 1.0);
}

/*
pub struct MoveAnimator {
    animating: bool,
    time: f32,
    start_point: (i32, i32),
    end_point: (i32, i32),
}
impl MoveAnimator {
    pub fn new(start_point: (i32, i32), end_point: (i32, i32)) -> Self {
        Self {
            animating: true,
            time: 0.0,
            start_point,
            end_point,
        }
    }
    pub fn animating(&self) -> bool {
        self.animating
    }
    /// Animates the piece, returns true if the animation is over
    pub fn animate(&mut self, piece: &mut GraphicPiece, delta_time: f32) -> bool {
        self.time += delta_time;
        if self.time >= ANIM_TIME {
            self.time = ANIM_TIME;
            self.animating = false
        }
        fn lerp(a: i32, b: i32, t: f32) -> i32 {
            (a as f32 * (1.0 - t) + b as f32 * t) as i32
        }
        piece.x = lerp(self.start_point.0, self.end_point.0, self.time / ANIM_TIME);
        piece.y = lerp(self.start_point.1, self.end_point.1, self.time / ANIM_TIME);
        !self.animating
    }
}
**/
