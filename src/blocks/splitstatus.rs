use std::time::{Duration, Instant};
use chan::Sender;
use std::thread;
use std::sync::{Arc, Mutex};

use block::{Block, ConfigBlock};
use config::Config;
use errors::*;
use widgets::text::TextWidget;
use widget::{I3BarWidget, State};
use scheduler::Task;

use uuid::Uuid;

extern crate i3ipc;
use self::i3ipc::I3EventListener;
use self::i3ipc::Subscription;
use self::i3ipc::event::Event;
use self::i3ipc::event::inner::WindowChange;

use std::collections::HashMap;

pub struct SplitStatus {
    text: TextWidget,
    id: String,
    container_mapping: Arc<Mutex<HashMap<i64, bool>>>, //<Window ID, State (v = true, h = false)>
    current_window: Arc<Mutex<i64>>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct SplitStatusConfig {}

impl SplitStatusConfig {}

impl ConfigBlock for SplitStatus {
    type Config = SplitStatusConfig;

    fn new(_block_config: Self::Config, config: Config, tx: Sender<Task>) -> Result<Self> {
        let id = Uuid::new_v4().simple().to_string();
        let id_clone = id.clone();

        let container_mapping_original = Arc::new(Mutex::new(HashMap::new()));
        let container_mapping = container_mapping_original.clone();

        let current_window_original = Arc::new(Mutex::new(0i64));
        let current_window = current_window_original.clone();

        thread::spawn(move || {
            // establish connection.
            let mut listener = I3EventListener::connect().unwrap();

            // subscribe to a couple events.
            let subs = [Subscription::Binding, Subscription::Window];
            listener.subscribe(&subs).unwrap();

            // handle them
            for event in listener.listen() {
                match event.unwrap() {
                    Event::BindingEvent(e) => {
                        let the_command = e.binding.command;
                        match the_command.as_ref() {
                            "split h" => {
                                    let mut container_mapping = container_mapping_original.lock().unwrap();
                                    let mut current_window = current_window_original.lock().unwrap();
                                    container_mapping.insert(*current_window, false);

                                    tx.send(Task {
                                        id: id_clone.clone(),
                                        update_time: Instant::now(),
                                    });
                            },
                            "split v" => {
                                    let mut container_mapping = container_mapping_original.lock().unwrap();
                                    let mut current_window = current_window_original.lock().unwrap();
                                    container_mapping.insert(*current_window, true);

                                    tx.send(Task {
                                        id: id_clone.clone(),
                                        update_time: Instant::now(),
                                    });
                            },
                            _ => {}
                        }
                    },
                    Event::WindowEvent(win_eventinfo) => {
                        let current_id = win_eventinfo.container.id;
                        let mut container_mapping = container_mapping_original.lock().unwrap();
                        let mut container_mapping_clone = container_mapping.clone();
                        //Set the current window (global variable) as the current window ID from
                        //this event.
                        let mut current_window = current_window_original.lock().unwrap();
                        match win_eventinfo.change {
                            WindowChange::New => {
                                //New windows automatically inherit the current state
                                //as their parents
                                let mut parent_state = container_mapping_clone.get(&current_window).unwrap_or(&false);
                                container_mapping.insert(current_id, *parent_state);
                                *current_window = current_id;
                                tx.send(Task {
                                    id: id_clone.clone(),
                                    update_time: Instant::now(),
                                });
                            },
                            WindowChange::Close => {
                                *current_window = current_id;
                                //When we close a window, remove it from the HashMap
                                //as it is no longer needed.
                                container_mapping.remove(&current_id);
                                tx.send(Task {
                                    id: id_clone.clone(),
                                    update_time: Instant::now(),
                                });
                            },
                            WindowChange::Focus => {
                                *current_window = current_id;
                                //There is a chance that we focus on a window which doesn't
                                //exist in the current HashMap. If so, add it to that 
                                //HashMap.
                                if !container_mapping.contains_key(&current_id) {
                                    container_mapping.insert(current_id, false);
                                }
                                tx.send(Task {
                                    id: id_clone.clone(),
                                    update_time: Instant::now(),
                                });
                            },
                            _ => {},
                        }
                    },
                    _ => unreachable!(),
                }
            }
        });

        Ok(SplitStatus {
            id,
            text: TextWidget::new(config),
            container_mapping,
            current_window,
        })
    }
}


impl Block for SplitStatus {
    fn update(&mut self) -> Result<Option<Duration>> {
        
        let container = self.container_mapping
            .lock()
            .block_error("Oops", "failed to aquire lock")?;

        let current_window2 = self.current_window
            .lock()
            .block_error("Oops", "failed to aquire lock")?;

        let my_state = container.get(&current_window2);

        let current_state =  my_state.unwrap_or(&false);
  
        if *current_state {
            self.text.set_icon("vertical");
            self.text.set_state(State::Info);
        } else {
            self.text.set_icon("horizontal");
            self.text.set_state(State::Good);
        }
        Ok(None)
    }

    fn view(&self) -> Vec<&I3BarWidget> {
        vec![&self.text]
    }

    fn id(&self) -> &str {
        &self.id
    }
}
