mod cli;
mod counter;
mod quit;
mod tui;

use anyhow::{anyhow, Context};
use clap::Parser;
use counter::{start_ticking, Counter};
use figlet_rs::FIGfont;
use quit::{AppEvent, Quit};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::ToText,
    widgets::{Paragraph, Widget},
    Frame,
};
use soloud::*;
use std::path::Path;
use std::sync::{self, Arc};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
pub struct App<'a> {
    hours: i16,
    minutes: i16,
    seconds: i16,
    counter: Counter,
    negative: bool,
    font: FIGfont,
    message: &'a str,
}

impl<'a> App<'a> {
    pub fn run(
        &mut self,
        terminal: &mut tui::Tui,
        mut receiver: UnboundedReceiver<AppEvent>,
        sender: UnboundedSender<AppEvent>,
        quit: Arc<Quit>,
        soloud: Soloud,
        wav: Wav,
    ) -> anyhow::Result<()> {
        let handle_app_events = |app: &mut Self, event: AppEvent| -> anyhow::Result<()> {
            match event {
                AppEvent::TickEvent => {
                    app.counter.count -= 1;
                    
                    if app.counter.count == -1 {
                        app.negative = true;
                        soloud.play(&wav);
                    } else if app.counter.count % 300 == 0 && app.counter.count.is_negative() {
                        soloud.play(&wav);
                    }
                }
                AppEvent::ErrorEvent => {
                    return Err(anyhow!("Failed to read keypress event."));
                }
            }

            Ok(())
        };

        start_ticking(sender);

        while !quit.bool.load(sync::atomic::Ordering::Relaxed) {
            if let Ok(event) = receiver.try_recv() {
                handle_app_events(self, event)?
            };

            terminal
                .draw(|frame| self.render_frame(frame))
                .context("Failed to render the frame.")?;

            self.update_clock(self.counter.count);
        }

        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(ratatui::widgets::Clear, frame.area());
        frame.render_widget(self, frame.area());
    }

    fn update_clock(&mut self, count: i32) {
        self.hours = (count.abs() / 3600) as i16;
        self.seconds = (count.abs() % 60) as i16;
        self.minutes = ((count.abs() - i32::from(self.hours) * 3600) / 60) as i16;
    }

    fn new(
        font: Result<FIGfont, String>,
        message_arg: &'a str,
        seconds: i32,
    ) -> anyhow::Result<Self> {
        if let Err(e) = font {
            return Err(anyhow!("Failed to import font. Err: {}", e));
        }

        Ok(Self {
            hours: 0,
            minutes: 0,
            seconds: 0,
            counter: Counter { count: seconds },
            font: font.unwrap(),
            message: message_arg,
            negative: false,
        })
    }
}

impl Widget for &App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let timer_area = Rect::new(
            area.x + (area.width.saturating_sub(127)) / 2,
            area.y + (area.height.saturating_sub(5) / 2),
            126.min(area.width),
            5.min(area.height),
        );

        let string = match (self.hours, self.minutes) {
            (0, 0) => format!("{}{}", if self.negative { "-" } else { "" }, self.seconds),
            (0, _) => format!(
                "{}{}:{:02}",
                if self.negative { "-" } else { "" },
                self.minutes,
                self.seconds
            ),
            _ => format!(
                "{}{}:{:02}:{:02}",
                if self.negative { "-" } else { "" },
                self.hours,
                self.minutes,
                self.seconds
            ),
        };

        let counter_text = self
            .font
            // convert() only returns err when the string is empty.
            .convert(&string)
            .unwrap(); // -> UNWRAPPING BECAUSE I'M SURE THE STRING IS NOT EMPTY

        Paragraph::new(counter_text.to_text().centered().green())
            .centered()
            .render(timer_area, buf);

        if self.message != "" {
            // vvv Block needed for gap between timer and mesasge
            let block = ratatui::widgets::Block::new().padding(ratatui::widgets::Padding::top(1));
            let message_area = Rect::new(
                area.x + (area.width.saturating_sub(60) / 2),
                timer_area.bottom(),
                60.min(area.width),
                area.height.saturating_sub(timer_area.bottom()),
            );

            Paragraph::new(self.message.white())
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: true })
                .centered()
                .render(message_area, buf);
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sl = Soloud::default()?;
    let mut wav = audio::Wav::default();
    wav.load(&Path::new("audio/tone.wav"))?;

    let (s, r) = unbounded_channel::<AppEvent>();

    let quit = Quit::default().handle_events(s.clone());

    let parsed_args = cli::args::Args::parse();
    let (seconds, message) = parsed_args.handle_command().context("Bad argument.")?;

    let font = FIGfont::from_file("fonts/Letters.flf");
    if let Err(e) = font {
        return Err(anyhow!("Failed to import font. Err: {}", e));
    }

    let mut terminal = tui::init().context("Failed to start new terminal.")?;

    let app_result = App::new(font, message, seconds)?.run(&mut terminal, r, s, quit, sl, wav);

    if let Err(e) = tui::restore() {
        return Err(anyhow!("Failed to restore terminal: {}", e));
    }
    app_result
}
