mod tui;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use anyhow::{anyhow, Context};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Padding, Paragraph, Widget},
    Frame,
};

#[derive(Debug, Default)]
pub struct Counter {
    pub count: i32,
}

impl Counter {
    pub fn start_counting(self) -> Arc<Mutex<Self>> {
        let contador = Arc::new(Mutex::new(self));
        {
            let contador = Arc::clone(&contador);
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(1));
                let mut locked_data = contador.lock().unwrap();
                locked_data.count = locked_data.count - 1;
            });
        }

        return contador;
    }
}

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
                                    Quit::quit(&quit);
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

    pub fn quit(self_arc: &Arc<Mutex<Self>>) {
        self_arc.lock().unwrap().bool = true;
    }
}

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

use clap::Parser;
#[derive(Parser, Debug)]
#[command(about)]
pub struct Args {
    ///Time in seconds
    time: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let mut terminal = tui::init().context("Failed to start new terminal.")?;

    let quit = Quit::default().handle_events(); //ERROR HANDLING TODO

    if Args::parse().time.is_none() {
        return Err(anyhow!("Argument [TIME] not found."));
    }

    let contador = Counter {
        count: Args::parse().time.unwrap().parse::<i32>()?,
    }
    .start_counting();

    let app_result = App::default().run(&mut terminal, contador, quit);

    if let Err(e) = tui::restore() {
        eprint!("Failed to restore the terminal: {}", e)
    }
    app_result
}
