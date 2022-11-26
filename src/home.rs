use tui::{layout::Alignment, style::Color, widgets::Paragraph};

use crate::handler::create_basic_block;

const TODOIST: &str = "
.-----. .--. .---.  .--. .-. .--. .-----.   .-----..-..-..-.
`-. .-': ,. :: .  :: ,. :: :: .--'`-. .-'   `-. .-': :: :: :
  : :  : :; ;: :; :: :| ;; :`. `.   ; : _____ ; |  : :; || ;
  : |  | || || || || || || | _`, |  | |;_____;| |  | || || | 
  |_|  `.__.':___.'`.__.':_;`.__.'  |_|       |_|  `.__.'|_|";

pub fn render_home<'a>() -> Paragraph<'a> {
    let block = create_basic_block("Home", Color::White);

    let home = Paragraph::new(TODOIST)
        .alignment(Alignment::Center)
        .block(block);
    home
}
