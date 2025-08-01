//! Event handling for the TUI

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

/// Terminal events
#[derive(Clone, Debug)]
pub enum Event {
    /// Terminal tick
    Tick,
    /// Key press
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
}

/// Terminal event handler
pub struct EventHandler {
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`]
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or_else(|| Duration::from_secs(0));

                    if event::poll(timeout).expect("Failed to poll events") {
                        match event::read().expect("Failed to read event") {
                            CrosstermEvent::Key(key) => {
                                sender.send(Event::Key(key)).expect("Failed to send key event");
                            }
                            CrosstermEvent::Mouse(mouse) => {
                                sender.send(Event::Mouse(mouse)).expect("Failed to send mouse event");
                            }
                            CrosstermEvent::Resize(width, height) => {
                                sender.send(Event::Resize(width, height)).expect("Failed to send resize event");
                            }
                            _ => {}
                        }
                    }

                    if last_tick.elapsed() >= tick_rate {
                        sender.send(Event::Tick).expect("Failed to send tick event");
                        last_tick = Instant::now();
                    }
                }
            })
        };
        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Receive the next event from the handler thread
    pub fn next(&mut self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}