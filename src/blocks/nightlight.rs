use std::env;
use std::time::Duration;
use std::process::Command;
use chan::Sender;
use scheduler::Task;

use block::{Block, ConfigBlock};
use config::Config;
use de::deserialize_opt_duration;
use errors::*;
use widgets::button::ButtonWidget;
use widget::{I3BarWidget, State};
use input::{I3BarEvent, MouseButton};

use uuid::Uuid;

pub struct NightLight {
    text: ButtonWidget,
    command_on: String,
    command_off: String,
    command_state: String,
    icon_on: String,
    icon_off: String,
    update_interval: Option<Duration>,
    toggled: bool,
    color_temperature: usize,
    id: String,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct NightLightConfig {
    /// Update interval in seconds
    #[serde(default, deserialize_with = "deserialize_opt_duration")]
    pub interval: Option<Duration>,

    /// Shell Command to enable the toggle
    #[serde(default = "NightLightConfig::default_command_on")]
    pub command_on: String,

    /// Shell Command to disable the toggle
    #[serde(default = "NightLightConfig::default_command_off")]
    pub command_off: String,

    /// Shell Command to determine toggle state. <br/>Empty output => off. Any output => on.
    #[serde(default = "NightLightConfig::default_command_state")]
    pub command_state: String,

    /// Icon ID when toggled on (default is "toggle_on")
    #[serde(default = "NightLightConfig::default_icon_on")]
    pub icon_on: String,

    /// Icon ID when toggled off (default is "toggle_off")
    #[serde(default = "NightLightConfig::default_icon_off")]
    pub icon_off: String,

    #[serde(default = "NightLightConfig::default_color_temperature")]
    pub color_temperature: usize,

    /// Text to display in i3bar for this block
    pub text: Option<String>,
}

impl NightLightConfig {

    fn default_command_state() -> String {
        "redshift -vx | grep Color".to_owned()
    }

    fn default_command_on() -> String {
        "redshift -PO 4500K".to_owned()
    }

    fn default_command_off() -> String {
        "redshift -x".to_owned()
    }

    fn default_icon_on() -> String {
        "moon".to_owned()
    }

    fn default_icon_off() -> String {
        "sun".to_owned()
    }

    fn default_color_temperature() -> usize {
        4500
    }
}

impl ConfigBlock for NightLight {
    type Config = NightLightConfig;

    fn new(block_config: Self::Config, config: Config, _tx_update_request: Sender<Task>) -> Result<Self> {
        let id = Uuid::new_v4().simple().to_string();
        Ok(NightLight {
            text: ButtonWidget::new(config, &id).with_content(block_config.text),
            command_on: block_config.command_on,
            command_off: block_config.command_off,
            command_state: block_config.command_state,
            icon_on: block_config.icon_on,
            icon_off: block_config.icon_off,
            id,
            toggled: false,
            update_interval: block_config.interval,
            color_temperature: block_config.color_temperature,
        })
    }
}

impl Block for NightLight {
    fn update(&mut self) -> Result<Option<Duration>> {
        
        if self.toggled {
            self.text.set_icon(self.icon_on.as_str());
        } else {
             self.text.set_icon(self.icon_off.as_str());           
        }

        Ok(self.update_interval)
    }

    fn view(&self) -> Vec<&I3BarWidget> {
        vec![&self.text]
    }

    fn click(&mut self, e: &I3BarEvent) -> Result<()> {
        if let Some(ref name) = e.name {
            if name.as_str() == self.id {
                self.text.set_text(self.color_temperature.to_string());
                match e.button {
                    MouseButton::WheelUp => {
                        self.color_temperature = self.color_temperature + 200;
                        if self.color_temperature > 25000 {
                            self.color_temperature = 25000;
                        }
                        Command::new(env::var("SHELL").unwrap_or("sh".to_owned()))
                            .args(&["-c", &self.command_on.replace("4500", &self.color_temperature.to_string())])
                            .output()
                            .block_error("toggle", "failed to run toggle command")?;
                    }
                    MouseButton::WheelDown => {
                        self.color_temperature = self.color_temperature - 200;
                        if self.color_temperature < 1000 {
                            self.color_temperature = 1000;
                        }
                        Command::new(env::var("SHELL").unwrap_or("sh".to_owned()))
                            .args(&["-c", &self.command_on.replace("4500", &self.color_temperature.to_string())])
                            .output()
                            .block_error("toggle", "failed to run toggle command")?;
                    }
                    MouseButton::Left => {
                        let cmd = if self.toggled {
                            self.color_temperature = 4500;
                            self.toggled = false;
                            self.text.set_text("");
                            self.text.set_icon(self.icon_off.as_str());
                            self.text.set_state(State::Idle);
                            &self.command_off
                        } else {
                            self.toggled = true;
                            self.text.set_icon(self.icon_on.as_str());
                            self.text.set_state(State::Warning);
                            self.command_on = self.command_on.replace("4500", &self.color_temperature.to_string());
                            &self.command_on
                        };

                        Command::new(env::var("SHELL").unwrap_or("sh".to_owned()))
                            .args(&["-c", cmd])
                            .output()
                            .block_error("toggle", "failed to run toggle command")?;
                    }
                    _ => ()
                }
            }
        }

        Ok(())
    }

    fn id(&self) -> &str {
        &self.id
    }
}
