use tui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub fn create_basic_block(title: &str, highlight_color: Color) -> Block {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(highlight_color))
        .border_type(BorderType::Plain)
}

pub fn create_advanced_block(
    title: &str,
    highlight_color: Color,
    alignment: Alignment,
) -> Block {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title_alignment(alignment)
        .border_style(Style::default().fg(highlight_color))
        .border_type(BorderType::Plain)
}

pub fn create_basic_paragraph(paragraph: String, block: Block) -> Paragraph {
    Paragraph::new(paragraph)
        .style(Style::default().fg(Color::White))
        .block(block)
}
