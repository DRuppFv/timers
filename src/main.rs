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
use soloud::*;
use std::path::Path;
use std::sync::{self, Arc, Mutex};

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
        terminal: &mut tui::Tui,
        receiver: Receiver<anyhow::Error>,
        counter: Arc<Mutex<Counter>>,
        quit: Arc<Quit>,
        soloud: Soloud,
        wav: Wav,
    ) -> anyhow::Result<()> {
        while !quit.bool.load(sync::atomic::Ordering::Relaxed) && receiver.is_empty() {
            terminal
                .draw(|frame| self.render_frame(frame))
                .context("Failed to render the frame.")?;

            let locked_counter = counter.lock().unwrap();
            if locked_counter.count < 0 {
                soloud.play(&wav);
                std::thread::sleep(std::time::Duration::from_secs(1));
                Quit::quit(&quit);
            }

            self.update_clock(locked_counter);
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

    fn update_clock(&mut self, counter: sync::MutexGuard<Counter>) {
        self.hours = (counter.count / 3600) as u16;
        self.seconds = (counter.count % 60) as u16;
        self.minutes = ((counter.count - i32::from(self.hours) * 3600) / 60) as u16;
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

        let padding = if area.height < 6 {
            Padding::ZERO
        } else {
            Padding::top((area.height - 6) / 2)
        };

        let block = Block::bordered()
            .title_bottom(instructions)
            .title_alignment(Alignment::Center)
            .padding(padding)
            .border_set(border::THICK);

        let counter_text = self
            .font
            // convert() only returns err when the string is empty.
            .convert(&format!(
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
    let sl = Soloud::default()?;
    let mut wav = audio::Wav::default();
    wav.load(&Path::new("audio/tone.wav"))?;

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

    let app_result =
        App::new(font)?.run(&mut terminal, r, contador.start_counting(), quit, sl, wav);

    if let Err(e) = tui::restore() {
        return Err(anyhow!("Failed to import font. Err: {}", e));
    }
    app_result
}
