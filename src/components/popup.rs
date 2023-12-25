use color_eyre::owo_colors::OwoColorize;
use derive_setters::Setters;
use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        *,
    },
};

#[derive(Debug, Default, Setters)]
pub struct Popup<'a> {
    #[setters(into)]
    title: Line<'a>,

    #[setters(into)]
    subtitle: Line<'a>,

    #[setters(into)]
    content: Text<'a>,

    border_style: Style,
    title_style: Style,
    style: Style,
}

impl Widget for Popup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let block = Block::new()
            .title(self.title)
            .title_style(self.title_style)
            .title(
                Title::from(self.subtitle)
                    .position(Position::Bottom)
                    .alignment(Alignment::Left),
            )
            .borders(Borders::ALL)
            .border_style(self.border_style);

        Paragraph::new(self.content)
            .wrap(Wrap { trim: true })
            .style(self.style)
            .block(block)
            .render(area, buf)
    }
}
