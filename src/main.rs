mod cli;
mod counter;
mod quit;
mod tui;

use anyhow::Context;
use clap::Parser;
use counter::Counter;
use quit::Quit;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Padding, Paragraph, Widget},
    Frame,
};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct App {
    hours: u16,
    minutes: u16,
    seconds: u16,
}

impl App {
    pub fn run(
        &mut self,
        terminal: &mut tui::Tui,
        counter: Arc<Mutex<Counter>>,
        quit: Arc<Mutex<Quit>>,
    ) -> anyhow::Result<()> {
        while !quit.lock().unwrap().bool {
            terminal
                .draw(|frame| self.render_frame(frame))
                .context("Failed to render the frame.")?;

            let locked_counter = counter.lock().unwrap();

            if locked_counter.count < 0 {
                Quit::quit(&quit);
            }

            //put it inside a new func later
            self.hours = (locked_counter.count / 3600) as u16;
            self.seconds = (locked_counter.count % 60) as u16;
            self.minutes = ((locked_counter.count - self.hours as i32 * 3600) / 60) as u16;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(ratatui::widgets::Clear, frame.area());
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let instructions = Line::from(vec![" Quit".into(), " <Q> ".blue().bold()]);

        let block = Block::bordered()
            .title_bottom(instructions)
            .title_alignment(Alignment::Center)
            .padding(Padding::top(area.height / 2))
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            format!("{:02}", self.hours).to_string().green(),
            ":".to_string().blue(),
            format!("{:02}", self.minutes).to_string().yellow(),
            ":".to_string().blue(),
            format!("{:02}", self.seconds).to_string().red(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

fn main() -> anyhow::Result<()> {
    let mut terminal = tui::init().context("Failed to start new terminal.")?;

    let quit = Quit::default().handle_events(); //ERROR HANDLING TODO

    let mut contador = Counter::default();

    cli::args::Args::parse()
        .handle_command(&mut contador)
        .context("Bad command argument.")?;

    let app_result = App::default().run(&mut terminal, contador.start_counting(), quit);

    if let Err(e) = tui::restore() {
        eprint!("Failed to restore the terminal: {}", e)
    }
    app_result
}
