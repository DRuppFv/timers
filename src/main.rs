mod cli;
mod counter;
mod quit;
mod tui;

use anyhow::{anyhow, Context};
use clap::Parser;
use counter::Counter;
use crossbeam_channel::{bounded, Receiver};
use figlet_rs::FIGfont;
use quit::Quit;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, ToText},
    widgets::{Block, Padding, Paragraph, Widget},
    Frame,
};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct App {
    hours: u16,
    minutes: u16,
    seconds: u16,
    font: FIGfont,
}

impl App {
    pub fn run(
        &mut self,
        receiver: Receiver<anyhow::Error>,
        terminal: &mut tui::Tui,
        counter: Arc<Mutex<Counter>>,
        quit: Arc<Mutex<Quit>>,
    ) -> anyhow::Result<()> {
        while !quit.lock().unwrap().bool && receiver.is_empty() {
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

        if let Ok(x) = receiver.try_recv() {
            return Err(x);
        }

        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(ratatui::widgets::Clear, frame.area());
        frame.render_widget(self, frame.area());
    }

    fn new(font: Result<FIGfont, String>) -> anyhow::Result<Self> {
        if let Err(e) = font {
            return Err(anyhow!("Failed to import font. Err: {}", e));
        }

        Ok(Self {
            hours: 0,
            minutes: 0,
            seconds: 0,
            font: font.unwrap(),
        })
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

        let counter_text = self
            .font
            .convert(&format!(
                // Converts only returns err when the string is empty.
                "{:02} : {:02} : {:02}",
                self.hours, self.minutes, self.seconds
            ))
            .unwrap(); // -> UNWRAPPING BECAUSE I'M SURE THE STRING IS NOT EMPTY

        Paragraph::new(counter_text.to_text().centered().green())
            .centered()
            .block(block)
            .render(area, buf);
    }
}

fn main() -> anyhow::Result<()> {
    let mut terminal = tui::init().context("Failed to start new terminal.")?;

    let (s, r) = bounded::<anyhow::Error>(1);

    let quit = Quit::default().handle_events(s);

    let mut contador = Counter::default();

    cli::args::Args::parse()
        .handle_command(&mut contador)
        .context("Bad argument.")?;

    let font = FIGfont::from_file("fonts/Letters.flf");

    if let Err(e) = font {
        return Err(anyhow!("Failed to import font. Err: {}", e));
    }

    let app_result = App::new(font)?.run(r, &mut terminal, contador.start_counting(), quit);

    if let Err(e) = tui::restore() {
        eprint!("Failed to restore the terminal: {}", e)
    }
    app_result
}
