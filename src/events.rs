use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Quiet Action
    Quit,
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct EventHandler {
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = std::time::Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);
            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    _ = sender.closed() => {
                        break;
                    },
                    _ = tick_delay => {
                        sender.send(Event::Tick).unwrap();
                    },
                    Some(Ok(evt)) = crossterm_event => {
                        match evt {
                            CrosstermEvent::Key(key) => {
                                if key.kind == crossterm::event::KeyEventKind::Press {
                                    match key.code {
                                        // Exit application on `ESC` or `q`
                                        KeyCode::Esc | KeyCode::Char('q') => {
                                            sender.send(Event::Quit).unwrap();
                                        }
                                        // Exit application on `Ctrl-C`
                                        KeyCode::Char('c') | KeyCode::Char('C') => {
                                            if key.modifiers == KeyModifiers::CONTROL {
                                                sender.send(Event::Quit).unwrap();
                                            }
                                        }
                                        _ => {
                                            sender.send(Event::Key(key)).unwrap();
                                        }
                                    }
                                }
                            },
                            CrosstermEvent::Resize(x, y) => {
                                sender.send(Event::Resize(x, y)).unwrap();
                            },
                            CrosstermEvent::Mouse(_) => {
                            },
                            CrosstermEvent::FocusLost => {
                            },
                            CrosstermEvent::FocusGained => {
                            },
                            CrosstermEvent::Paste(_) => {
                            },
                        }
                    }
                };
            }
        });
        Self {
            receiver,
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub async fn next(&mut self) -> Event {
        self.receiver.recv().await.expect("no err")
    }
}
