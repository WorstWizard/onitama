use glam::{vec2, Vec2, Vec3};

pub mod board;
pub mod card;
pub mod piece;
pub mod renderer;

const ANIM_TIME: f32 = 0.25;
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
#[derive(Clone)]
pub struct GraphicCard {
    game_card: Card,
    pub rect: Rect,
}
impl GraphicCard {
    const WIDTH: u32 = 200;
    const HEIGHT: u32 = 200;

    pub fn new(game_card: Card, rect: Rect) -> Self {
        Self { game_card, rect }
    }
    pub fn card(&self) -> Card {
        self.game_card
    }
    pub fn draw(&self, canvas: &mut Canvas<Window>, upwards: bool, selected: bool) {
        const LINEWIDTH: i32 = 5;

        if selected {
            canvas.set_draw_color(colors::CARD_SELECTED);
        } else {
            canvas.set_draw_color(colors::CARD_BG);
        }
        canvas.fill_rect(self.rect).unwrap();

        let (x, y) = (self.rect.x(), self.rect.y());
        let sub_rect_w = (self.rect.width() - 6 * LINEWIDTH as u32) / 5;
        let sub_rect_h = (self.rect.height() - 6 * LINEWIDTH as u32) / 5;
        // let sub_rect_w = self.rect.width() / 5;
        // let sub_rect_h = self.rect.height() / 5;
        let offsets = if upwards {
            self.game_card.offsets()
        } else {
            self.game_card.rev_offsets()
        };
        canvas.set_draw_color(colors::CARD_TILE_BG);
        for row in 0..5 {
            for col in 0..5 {
                canvas
                    .fill_rect(Rect::new(
                        x + LINEWIDTH + col * (sub_rect_w as i32 + LINEWIDTH),
                        y + LINEWIDTH + row * (sub_rect_h as i32 + LINEWIDTH),
                        sub_rect_w,
                        sub_rect_h,
                    ))
                    .unwrap();
            }
        }
        canvas.set_draw_color(colors::CARD_TILE);
        for pos in offsets {
            canvas
                .fill_rect(Rect::new(
                    x + LINEWIDTH + (pos.1 as i32 + 2) * (sub_rect_w as i32 + LINEWIDTH),
                    y + LINEWIDTH + (pos.0 as i32 + 2) * (sub_rect_h as i32 + LINEWIDTH),
                    sub_rect_w,
                    sub_rect_h,
                ))
                .unwrap();
        }
        canvas.set_draw_color(colors::CARD_CENTER);
        canvas
            .fill_rect(Rect::new(
                x + LINEWIDTH + 2 * (sub_rect_w as i32 + LINEWIDTH),
                y + LINEWIDTH + 2 * (sub_rect_h as i32 + LINEWIDTH),
                sub_rect_w,
                sub_rect_h,
            ))
            .unwrap();
    }
}
pub struct CardGraphicManager {
    pub red_cards: (GraphicCard, GraphicCard),
    pub blue_cards: (GraphicCard, GraphicCard),
    pub transfer_card: GraphicCard,
    selected_card: Option<GraphicCard>,
}
impl CardGraphicManager {
    pub fn new(game_board: &Board, container_rect: Rect) -> Self {
        let cards = game_board.cards();
        let card_w = (container_rect.width() / 2).min(GraphicCard::WIDTH) as i32;
        let card_h = (container_rect.height() / 3).min(GraphicCard::HEIGHT) as i32;
        let x = container_rect.x();
        let y = container_rect.y();
        let w = container_rect.width() as i32;
        let h = container_rect.height() as i32;
        let red_card_0 = GraphicCard::new(
            cards[0],
            Rect::new(x, y + h - card_h, card_w as u32, card_h as u32),
        );
        let red_card_1 = GraphicCard::new(
            cards[1],
            Rect::new(x + w - card_w, y + h - card_h, card_w as u32, card_h as u32),
        );
        let blue_card_0 = GraphicCard::new(cards[2], Rect::new(x, y, card_w as u32, card_h as u32));
        let blue_card_1 = GraphicCard::new(
            cards[3],
            Rect::new(x + w - card_w, y, card_w as u32, card_h as u32),
        );
        let transfer_card = GraphicCard::new(
            cards[4],
            Rect::new(
                x + (w - card_w) / 2,
                y + (h - card_h) / 2,
                card_w as u32,
                card_h as u32,
            ),
        );
        Self {
            red_cards: (red_card_0, red_card_1),
            blue_cards: (blue_card_0, blue_card_1),
            transfer_card,
            selected_card: None,
        }
    }
    pub fn select_by_click(&mut self, clicked_pos: (i32, i32), red_to_move: bool) {
        if red_to_move {
            if self.red_cards.0.rect.contains_point(clicked_pos) {
                self.selected_card = Some(self.red_cards.0.clone())
            } else if self.red_cards.1.rect.contains_point(clicked_pos) {
                self.selected_card = Some(self.red_cards.1.clone())
            }
        } else if self.blue_cards.0.rect.contains_point(clicked_pos) {
            self.selected_card = Some(self.blue_cards.0.clone())
        } else if self.blue_cards.1.rect.contains_point(clicked_pos) {
            self.selected_card = Some(self.blue_cards.1.clone())
        }
    }
    pub fn select_card(&mut self, card: Card) {
        let mut idx = None;
        for (i, graphic_card) in self.cards().into_iter().enumerate() {
            if graphic_card.game_card == card {
                idx = Some(i)
            }
        }
        if let Some(i) = idx {
            self.selected_card = Some(self.cards()[i].clone())
        }
    }
    fn cards(&self) -> [&GraphicCard; 5] {
        [
            &self.red_cards.0,
            &self.red_cards.1,
            &self.blue_cards.0,
            &self.blue_cards.1,
            &self.transfer_card,
        ]
    }
    pub fn unselect(&mut self) {
        self.selected_card = None
    }
    pub fn selected_card(&self) -> Option<&GraphicCard> {
        self.selected_card.as_ref()
    }
    pub fn draw(&self, canvas: &mut Canvas<Window>, red_to_move: bool) {
        self.red_cards.0.draw(canvas, true, false);
        self.red_cards.1.draw(canvas, true, false);
        self.blue_cards.0.draw(canvas, false, false);
        self.blue_cards.1.draw(canvas, false, false);
        self.transfer_card.draw(canvas, red_to_move, false);
        if let Some(card) = &self.selected_card {
            card.draw(canvas, red_to_move, true);
        }
    }
    /// Swaps selected card with transfer card
    /// Assumes a card is selected, panics otherwise
    pub fn swap_cards(&mut self) {
        let selected_card = self.selected_card().unwrap();
        if selected_card.game_card == self.red_cards.0.game_card {
            std::mem::swap(
                &mut self.red_cards.0.game_card,
                &mut self.transfer_card.game_card,
            );
        } else if selected_card.game_card == self.red_cards.1.game_card {
            std::mem::swap(
                &mut self.red_cards.1.game_card,
                &mut self.transfer_card.game_card,
            );
        } else if selected_card.game_card == self.blue_cards.0.game_card {
            std::mem::swap(
                &mut self.blue_cards.0.game_card,
                &mut self.transfer_card.game_card,
            );
        } else if selected_card.game_card == self.blue_cards.1.game_card {
            std::mem::swap(
                &mut self.blue_cards.1.game_card,
                &mut self.transfer_card.game_card,
            );
        }
        self.selected_card = None;
    }
}

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
