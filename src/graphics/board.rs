use glam::{vec2, Vec2};

use super::colors;
use super::renderer::SimpleRenderer;
use crate::game::Pos;

#[derive(Clone, Copy)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}
impl Rect {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect { x, y, width, height }
    }
    fn contains_point(&self, pos: Vec2) -> bool {
        pos.x >= self.x && pos.x < self.x + self.width &&
        pos.y >= self.y && pos.y < self.y + self.height
    }
}

/// Draws board and provides offsets for individual tiles
pub struct GraphicBoard {
    origin: Vec2,
    board_width: f32,
    tile_width: f32,
    tiles: [Rect; 25],
    tile_corners: [Vec2; 25],
}
impl GraphicBoard {
    pub fn new(renderer: &SimpleRenderer) -> Self {
        const LINEWIDTH: f32 = 10.0; //px

        let screen_size = renderer.output_size();
        let board_width = u32::min(screen_size.0, screen_size.1) as f32;
        let tile_width = (board_width - LINEWIDTH * 6.0) / 5.0;
        let origin = vec2(0.0, 0.0);
        let mut tiles = [Rect::new(0.0, 0.0, 0.0, 0.0); 25];
        let mut tile_corners = [vec2(0.0, 0.0); 25];
        let mut i = 0;
        for row in 0..5 {
            for col in 0..5 {
                let tile_x = LINEWIDTH + origin.x + col as f32 * (tile_width + LINEWIDTH);
                let tile_y = LINEWIDTH + origin.y + row as f32 * (tile_width + LINEWIDTH);
                tiles[i] = Rect::new(tile_x, tile_y, tile_width, tile_width);
                tile_corners[i] = vec2(tile_x, tile_y);
                i += 1;
            }
        }
        Self {
            origin,
            board_width,
            tile_width,
            tiles,
            tile_corners,
        }
    }
    pub fn draw_board(&self, renderer: &mut SimpleRenderer) {
        renderer.draw_filled_rect(
            self.origin,
            self.board_width,
            self.board_width,
            colors::BOARD_BG,
        );
        for pos in self.tile_corners() {
            renderer
                .draw_filled_rect(*pos, self.tile_width, self.tile_width, colors::BOARD_TILE)
        }
        // Draw temples
        let w = self.tile_width / 3.0;
        let h = self.tile_width / 4.0;
        let red_start_corner = self.tile_corners[22];
        let blue_start_corner = self.tile_corners[2];
        renderer.draw_filled_rect(
            vec2(
                red_start_corner.x + w,
                red_start_corner.y + 3.0 * h,
            ),
            w,
            h,
            colors::BOARD_RED_TEMPLE
        );
        renderer.draw_filled_rect(
            vec2(
                blue_start_corner.x + w,
                blue_start_corner.y + 3.0 * h,
            ),
            w,
            h,
            colors::BOARD_BLUE_TEMPLE
        );
    }
    pub fn highlight_tiles(&self, renderer: &mut SimpleRenderer, positions: &[Pos]) {
        for pos in positions {
            let corner_pos = self.tile_corners[pos.to_index()];
            renderer.draw_filled_rect(
                corner_pos,
                self.tile_width,
                self.tile_width,
                colors::BOARD_HIGHLIGHT
            );
        }
    }
    pub fn tile_corners(&self) -> &[Vec2; 25] {
        &self.tile_corners
    }
    pub fn board_width(&self) -> f32 {
        self.board_width
    }
    /// If the given position in window coords is on a tile on the board, returns that position.
    pub fn window_to_board_pos(&self, pos: Vec2) -> Option<Pos> {
        for (i, tile) in self.tiles.iter().enumerate() {
            if tile.contains_point(pos) {
                return Some(Pos::from_index(i));
            }
        }
        None
    }
}