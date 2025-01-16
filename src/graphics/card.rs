use glam::{vec2, Vec2};

use crate::game::Board;
use crate::cards::Card;
use super::colors;
use super::renderer::SimpleRenderer;

#[derive(Clone)]
pub struct GraphicCard {
    game_card: Card,
    pub origin: Vec2,
    width: f32,
    height: f32,
}
impl GraphicCard {
    const WIDTH: f32 = 200.0;
    const HEIGHT: f32 = 200.0;

    pub fn new(game_card: Card, origin: Vec2, width: f32, height: f32) -> Self {
        Self { game_card, origin, width, height }
    }
    pub fn card(&self) -> Card {
        self.game_card
    }
    pub fn draw(&self, renderer: &mut SimpleRenderer, upwards: bool, selected: bool) {
        const LINEWIDTH: f32 = 5.0;

        let bg_color = if selected {
            colors::CARD_SELECTED
        } else {
            colors::CARD_BG
        };
        renderer.draw_filled_rect(self.origin, self.width, self.height, bg_color);

        let (x, y) = self.origin.into();
        let sub_rect_w = (self.width - 6.0 * LINEWIDTH) / 5.0;
        let sub_rect_h = (self.height - 6.0 * LINEWIDTH) / 5.0;
        let offsets = if upwards {
            self.game_card.offsets()
        } else {
            self.game_card.rev_offsets()
        };
        for row in 0..5 {
            for col in 0..5 {
                renderer
                    .draw_filled_rect(
                        vec2(
                            x + LINEWIDTH + col as f32 * (sub_rect_w + LINEWIDTH),
                            y + LINEWIDTH + row as f32 * (sub_rect_h + LINEWIDTH)
                        ),
                        sub_rect_w,
                        sub_rect_h,
                        colors::CARD_TILE_BG
                    );
            }
        }
        for pos in offsets {
            renderer
                .draw_filled_rect(
                    vec2(
                        x + LINEWIDTH + (pos.1 as f32 + 2.0) * (sub_rect_w + LINEWIDTH),
                        y + LINEWIDTH + (pos.0 as f32 + 2.0) * (sub_rect_h + LINEWIDTH)
                    ),
                    sub_rect_w,
                    sub_rect_h,
                    colors::CARD_TILE
                );
        }
        renderer
            .draw_filled_rect(
                vec2(
                    x + LINEWIDTH + 2.0 * (sub_rect_w + LINEWIDTH),
                    y + LINEWIDTH + 2.0 * (sub_rect_h + LINEWIDTH)
                ),
                sub_rect_w,
                sub_rect_h,
                colors::CARD_CENTER
            )
    }
}
pub struct CardGraphicManager {
    pub red_cards: (GraphicCard, GraphicCard),
    pub blue_cards: (GraphicCard, GraphicCard),
    pub transfer_card: GraphicCard,
    selected_card: Option<GraphicCard>,
}
impl CardGraphicManager {
    pub fn new(game_board: &Board, origin: Vec2, width: f32, height: f32) -> Self {
        let cards = game_board.cards();
        let card_w = (width / 2.0).min(GraphicCard::WIDTH);
        let card_h = (height / 3.0).min(GraphicCard::HEIGHT);
        let x = origin.x;
        let y = origin.y;
        let w = width;
        let h = height;
        let red_card_0 = GraphicCard::new(
            cards[0],
            vec2(x, y + h - card_h),
            card_w,
            card_h
        );
        let red_card_1 = GraphicCard::new(
            cards[1],
            vec2(x + w - card_w, y + h - card_h),
            card_w,
            card_h
        );
        let blue_card_0 = GraphicCard::new(
            cards[2],
            vec2(x, y),
            card_w,
            card_h
        );
        let blue_card_1 = GraphicCard::new(
            cards[3],
            vec2(x + w - card_w, y),
            card_w,
            card_h
        );
        let transfer_card = GraphicCard::new(
            cards[4],
            vec2(x + (w - card_w) / 2.0, y + (h - card_h) / 2.0),
            card_w,
            card_h
        );
        // let red_card_1 = GraphicCard::new(
        //     cards[1],
        //     Rect::new(x + w - card_w, y + h - card_h, card_w as u32, card_h as u32),
        // );
        // let blue_card_0 = GraphicCard::new(cards[2], Rect::new(x, y, card_w as u32, card_h as u32));
        // let blue_card_1 = GraphicCard::new(
        //     cards[3],
        //     Rect::new(x + w - card_w, y, card_w as u32, card_h as u32),
        // );
        // let transfer_card = GraphicCard::new(
        //     cards[4],
        //     Rect::new(
        //         x + (w - card_w) / 2,
        //         y + (h - card_h) / 2,
        //         card_w as u32,
        //         card_h as u32,
        //     ),
        // );
        Self {
            red_cards: (red_card_0, red_card_1),
            blue_cards: (blue_card_0, blue_card_1),
            transfer_card,
            selected_card: None,
        }
    }
    // pub fn select_by_click(&mut self, clicked_pos: (i32, i32), red_to_move: bool) {
    //     if red_to_move {
    //         if self.red_cards.0.rect.contains_point(clicked_pos) {
    //             self.selected_card = Some(self.red_cards.0.clone())
    //         } else if self.red_cards.1.rect.contains_point(clicked_pos) {
    //             self.selected_card = Some(self.red_cards.1.clone())
    //         }
    //     } else if self.blue_cards.0.rect.contains_point(clicked_pos) {
    //         self.selected_card = Some(self.blue_cards.0.clone())
    //     } else if self.blue_cards.1.rect.contains_point(clicked_pos) {
    //         self.selected_card = Some(self.blue_cards.1.clone())
    //     }
    // }
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
    pub fn draw(&self, renderer: &mut SimpleRenderer, red_to_move: bool) {
        self.red_cards.0.draw(renderer, true, false);
        self.red_cards.1.draw(renderer, true, false);
        self.blue_cards.0.draw(renderer, false, false);
        self.blue_cards.1.draw(renderer, false, false);
        self.transfer_card.draw(renderer, red_to_move, false);
        if let Some(card) = &self.selected_card {
            card.draw(renderer, red_to_move, true);
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