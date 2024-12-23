mod tui;

use std::{
    sync::{Arc, Mutex},
    thread,
};

use anyhow::Context;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Padding, Paragraph, Widget},
    DefaultTerminal, Frame,
};

#[derive(Debug, Default)]
pub struct Quit {
    pub bool: bool,
}

impl Quit {
    pub fn handle_events(self) -> Arc<Mutex<Quit>> {
        let quit: Arc<Mutex<Quit>> = Arc::new(Mutex::new(self));
        {
            let quit = Arc::clone(&quit);
            thread::spawn(move || {
                loop {
                    match event::read() {
                        Ok(Event::Key(key_event)) if key_event.kind == KeyEventKind::Press => {
                            match key_event.code {
                                KeyCode::Char('q') => {
                                    quit.lock().unwrap().bool = true;
                                }
                                _ => {}
                            }
                        }
                        Err(_) => {} //HANDLING NEEDED TODO
                        _ => {}
                    };
                }
            });
        }

        return quit;
    }
}

#[derive(Debug, Default)]
pub struct App {
    seconds: u16,
}

impl App {
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        quit: Arc<Mutex<Quit>>,
    ) -> anyhow::Result<()> {
        while !quit.lock().unwrap().bool {
            terminal.draw(|frame| self.render_frame(frame))?;
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
            .border_set(border::THICK)
            .render(area, buf);
    }
}

fn main() -> anyhow::Result<()> {
    let mut terminal = tui::init().context("Failed to start new terminal.")?;

    let quit = Quit::default().handle_events(); //ERROR HANDLING TODO

    let app_result = App::default().run(&mut terminal, quit);

    if let Err(e) = tui::restore() {
        eprint!("Failed to restore the terminal: {}", e)
    }
    app_result
}
