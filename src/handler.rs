use tui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub fn create_basic_block<'a>(title: &'a str, highlight_color: Color) -> Block<'a> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(highlight_color))
        .border_type(BorderType::Plain)
}

pub fn create_advanced_block<'a>(
    title: &'a str,
    highlight_color: Color,
    alignment: Alignment,
) -> Block<'a> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title_alignment(alignment)
        .border_style(Style::default().fg(highlight_color))
        .border_type(BorderType::Plain)
}

pub fn create_basic_paragraph<'a>(paragraph: String, block: Block<'a>) -> Paragraph<'a> {
    Paragraph::new(paragraph)
        .style(Style::default().fg(Color::White))
        .block(block)
}
