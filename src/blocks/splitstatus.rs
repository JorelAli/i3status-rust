use std::time::{Duration, Instant};
use chan::Sender;
use std::thread;
use std::sync::{Arc, Mutex};

use block::{Block, ConfigBlock};
use config::Config;
use errors::*;
use widgets::text::TextWidget;
use widget::I3BarWidget;
use scheduler::Task;

use uuid::Uuid;

extern crate i3ipc;
use self::i3ipc::I3EventListener;
use self::i3ipc::Subscription;
use self::i3ipc::event::Event;
//use self::i3ipc::event::inner::{WindowChange, WorkspaceChange};

pub struct SplitStatus {
    text: TextWidget,
    title: Arc<Mutex<String>>,
    max_width: usize,
    id: String,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct SplitStatusConfig {
    /// Truncates titles if longer than max-width
    #[serde(default = "SplitStatusConfig::default_max_width")]
    pub max_width: usize,
}

impl SplitStatusConfig {
    fn default_max_width() -> usize {
        21
    }
}

impl ConfigBlock for SplitStatus {
    type Config = SplitStatusConfig;

    fn new(block_config: Self::Config, config: Config, tx: Sender<Task>) -> Result<Self> {
        let id = Uuid::new_v4().simple().to_string();
        let id_clone = id.clone();

        let title_original = Arc::new(Mutex::new(String::from("")));
        let title = title_original.clone();

        thread::spawn(move || {
            // establish connection.
            let mut listener = I3EventListener::connect().unwrap();

            // subscribe to a couple events.
            let subs = [Subscription::Window, Subscription::Workspace, Subscription::Binding];
            listener.subscribe(&subs).unwrap();

            // handle them
            for event in listener.listen() {
                match event.unwrap() {
//BindingEvent(BindingEventInfo { change: Run, binding: Binding { command: "split h", event_state_mask: ["Mod4"], input_code: 0, symbol: Some("h"), input_type: Keyboard } })

//BindingEvent(BindingEventInfo { change: Run, binding: Binding { command: "split v", event_state_mask: ["Mod4"], input_code: 0, symbol: Some("v"), input_type: Keyboard } })
                    Event::BindingEvent(e) => {
                        let the_command = e.binding.command;
                        match the_command.as_ref() {
                            "split h" => {

                                    let mut title = title_original.lock().unwrap();
                                    *title = "splith".to_string();
                                    tx.send(Task {
                                        id: id_clone.clone(),
                                        update_time: Instant::now(),
                                    });
                            },
                            "split v" => {
                                    let mut title = title_original.lock().unwrap();
                                    *title = "splitv".to_string();
                                    tx.send(Task {
                                        id: id_clone.clone(),
                                        update_time: Instant::now(),
                                    });
                            },
                            _ => {}
                        }
                    },
                    _ => unreachable!(),
                }
            }
        });

        Ok(SplitStatus {
            id,
            text: TextWidget::new(config),
            max_width: block_config.max_width,
            title,
        })
    }
}


impl Block for SplitStatus {
    fn update(&mut self) -> Result<Option<Duration>> {
        let mut string = (*self.title
            .lock()
            .block_error("focused_window", "failed to acquire lock")?)
            .clone();
        string = string.chars().take(self.max_width).collect();
        self.text.set_text(string);
        Ok(None)
    }

    fn view(&self) -> Vec<&I3BarWidget> {
        let title = &*self.title.lock().unwrap();
        if String::is_empty(title) {
            vec![]
        } else {
            vec![&self.text]
        }
    }

    fn id(&self) -> &str {
        &self.id
    }
}
