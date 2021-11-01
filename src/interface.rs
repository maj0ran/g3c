use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::fs;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    },
    Terminal,
};

use crate::client::GeminiClient;

enum Event<I> {
    Input(I),
    Tick,
}

enum InputMode {
    Normal,
    NavEdit,
}

pub struct Interface {
    client: GeminiClient,
    inputmode: InputMode,
    navbar: String,
    curr_site: String,
    content: String,
}

impl Interface {
    pub fn new(client: GeminiClient) -> Self {
        Interface {
            inputmode: InputMode::Normal,
            navbar: String::new(),
            curr_site: String::new(),
            content: String::new(),
            client,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;

        let (tx, rx) = mpsc::channel();
        let tick_rate = Duration::from_millis(200);

        // setup the event handler loop
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("failure at poll") {
                    if let CEvent::Key(key) = event::read().expect("failure at reading event") {
                        tx.send(Event::Input(key))
                            .expect("failure at sending event");
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if let Ok(_) = tx.send(Event::Tick) {
                        last_tick = Instant::now();
                    }
                }
            }
        });

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.clear()?;

        loop {
            terminal.draw(|rect| {
                let size = rect.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Min(2),
                            Constraint::Length(3),
                        ]
                        .as_ref(),
                    )
                    .split(size);
                let navbar = Paragraph::new(self.navbar.as_ref())
                    .block(Block::default().borders(Borders::ALL).title("Visit"))
                    .style(match self.inputmode {
                        InputMode::Normal => Style::default(),
                        InputMode::NavEdit => Style::default().fg(Color::Green),
                    });
                rect.render_widget(navbar, chunks[0]);

                let mainscreen = Paragraph::new(self.content.as_ref())
                    .block(Block::default().borders(Borders::ALL).title("Web"));
                rect.render_widget(mainscreen, chunks[1]);
            });

            // Handle input
            if let Event::Input(event) = rx.recv()? {
                match self.inputmode {
                    InputMode::Normal => match event.code {
                        KeyCode::Char('v') => {
                            self.inputmode = InputMode::NavEdit;
                        }
                        KeyCode::Char('q') => {
                            disable_raw_mode()?;
                            terminal.show_cursor()?;
                            break;
                        }
                        _ => {}
                    },
                    InputMode::NavEdit => match event.code {
                        KeyCode::Enter => {
                            let content = self.client.goto_url(self.navbar.clone());
                            self.content = content;
                            self.curr_site = self.navbar.clone();
                            self.inputmode = InputMode::Normal;
                        }
                        KeyCode::Esc => {
                            self.inputmode = InputMode::Normal;
                        }
                        KeyCode::Backspace => {
                            self.navbar.pop();
                        }
                        KeyCode::Char(c) => {
                            self.navbar.push(c);
                        }
                        _ => {}
                    },
                }
            }
        }

        Ok(())
    }
}
