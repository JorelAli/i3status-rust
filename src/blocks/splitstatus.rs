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
    current_window: i64,
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
        let mut current_window = 0;

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
                                    container_mapping.insert(current_window, false);

                                    tx.send(Task {
                                        id: id_clone.clone(),
                                        update_time: Instant::now(),
                                    });
                            },
                            "split v" => {
                                    let mut container_mapping = container_mapping_original.lock().unwrap();
                                    container_mapping.insert(current_window, true);

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

                        match win_eventinfo.change {
                            WindowChange::New => {
                                container_mapping.insert(current_id, false);
                                current_window = current_id;
                                tx.send(Task {
                                    id: id_clone.clone(),
                                    update_time: Instant::now(),
                                });
                            },
                            WindowChange::Close => {
                                container_mapping.remove(&current_id);
                                current_window = current_id;
                                tx.send(Task {
                                    id: id_clone.clone(),
                                    update_time: Instant::now(),
                                });
                            },
                            WindowChange::Focus => {
                                current_window = current_id;
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
        let my_state = container.get(&self.current_window);
        let current_state =  my_state.unwrap_or(&false);
     /*   if *my_state {
            self.text.set_icon("vertical");
            self.text.set_state(State::Info);
        }*/
        if *current_state {
            self.text.set_icon("vertical");
            self.text.set_state(State::Info);
        } else {
            self.text.set_icon("horizontal");
            self.text.set_state(State::Good);
        }
        
        /*
        let s = self.container_mapping.lock().unwrap();
//        let current_state = (self.container_mapping
 //                            .lock()
  //                           .block_error("oops", "Failed to lock")?)
   //         .clone().get(&self.current_window);
        let current_state = s.get(&self.current_window);
        if *current_state.unwrap() {
            self.text.set_icon("vertical");
            self.text.set_state(State::Info);
        } else {
            self.text.set_icon("horizontal");
            self.text.set_state(State::Good);
        }*/

        //let string = (*self.title
        //    .lock()
        //    .block_error("focused_window", "failed to acquire lock")?)
        //    .clone();
        //match string.as_ref() {
        //    "splith" => {
        //        self.text.set_icon("horizontal");
        //        self.text.set_state(State::Good);
        //    },
        //    "splitv" => {
        //        self.text.set_icon("vertical");
        //        self.text.set_state(State::Info);
        //    },
        //    _ => {
        //        self.text.set_icon("horizontal");
        //        self.text.set_state(State::Good);
        //    }
//
//        }
        Ok(None)
    }

    fn view(&self) -> Vec<&I3BarWidget> {
        //let title = &*self.title.lock().unwrap();
        //if String::is_empty(title) {
        //    vec![]
        //} else {
            vec![&self.text]
        //}
    }

    fn id(&self) -> &str {
        &self.id
    }
}
