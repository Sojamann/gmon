
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType};

pub trait Theme {
    fn text() -> Style;
    fn background() -> Style;
    fn red() -> Style;
    fn green() -> Style;
    fn blue() -> Style;

    fn block(&self) -> Block {
        Block::bordered()
            .border_type(BorderType::Thick)
            .border_style(Self::text())
            .style(Self::background())
            .title_style(Self::text())
    }
}

pub struct Catpuccin;

impl Theme for Catpuccin {
    fn text() -> Style {
        Color::Rgb(205, 214, 244).into()
    }
    fn background() -> Style {
        Color::Rgb(49, 50, 68).into()
    }
    fn red() -> Style {
        Color::Rgb(243, 139, 168).into()
    }
    fn green() -> Style {
        Color::Rgb(166, 227, 161).into()
    }
    fn blue() -> Style {
        Color::Rgb(137, 180, 250).into()
    }
}

