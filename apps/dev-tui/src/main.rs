use ratatui::{ DefaultTerminal, Frame, crossterm::{ event::{self, Event} } };

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let result = run(terminal);

    ratatui::restore();

    result
}

fn run(mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("hello world", frame.area());
}