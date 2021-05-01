pub mod broadcasting_software;
pub mod chat;
pub mod db;
pub mod error;
pub mod stream_servers;
pub mod switcher;

use broadcasting_software::obs::Obs;
use std::sync::Arc;
use stream_servers::Bsl;
use tokio::sync::{broadcast::Sender, Mutex};

pub use error::Error;
pub use switcher::Switcher;

// #[async_trait]
// pub trait SomethingAllTheServersNeed {
//     async fn get_stats(&self) -> Result<Option<NginxRtmpStream>, error::Error>;
//     fn get_bitrate() {}
// }

// pub trait SomethingAllTheSrtServersNeed {
//     fn get_rtt();
// }
//
// pub trait ChatCommands {
//     fn get_command_by_string(command: &str);
// }

// pub struct Config {}
//
// impl Config {
//     fn new() {
//         todo!();
//     }
//
//     fn load() {
//         todo!();
//     }
//
//     fn config_directory_exists() {
//         todo!();
//     }
// }

const VERSION: &str = "2.0.0";

pub fn print_logo() {
    println!(
        "
    ███╗   ██╗ ██████╗  █████╗ ██╗     ██████╗ ███████╗
    ████╗  ██║██╔═══██╗██╔══██╗██║     ██╔══██╗██╔════╝
    ██╔██╗ ██║██║   ██║███████║██║     ██████╔╝███████╗
    ██║╚██╗██║██║   ██║██╔══██║██║     ██╔══██╗╚════██║
    ██║ ╚████║╚██████╔╝██║  ██║███████╗██████╔╝███████║
    ╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝╚═════╝ ╚══════╝ v{}",
        VERSION
    );
}

#[derive(Debug, Clone)]
pub struct AutomaticSwitchMessage {
    channel: i64,
    scene: String,
}

#[derive(Debug, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum ChatLanguage {
    En,
}

pub struct Noalbs {
    username: i64,
    pub broadcasting_software: Arc<Obs>,
    pub switcher_state: Arc<Mutex<switcher::SwitcherState>>,
    pub chat_state: Arc<Mutex<chat::State>>,
    pub broadcast_sender: Sender<AutomaticSwitchMessage>,
    pub connections: Vec<db::Connection>,
    pub language: ChatLanguage,

    pub switcher_handler: Option<tokio::task::JoinHandle<Result<(), Error>>>,
}

impl Noalbs {
    pub fn new(
        username: i64,
        broadcasting_software: Obs,
        switcher_state: switcher::SwitcherState,
        chat_state: chat::State,
        broadcast_sender: Sender<AutomaticSwitchMessage>,
        connections: Vec<db::Connection>,
    ) -> Noalbs {
        let broadcasting_software = Arc::new(broadcasting_software);
        let switcher_state = Arc::new(Mutex::new(switcher_state));
        let chat_state = Arc::new(Mutex::new(chat_state));

        Self {
            username,
            broadcasting_software,
            switcher_state,
            chat_state,
            broadcast_sender,
            language: ChatLanguage::En,
            switcher_handler: None,
            connections,
        }
    }

    pub async fn add_stream_server<T>(&self, server: T)
    where
        T: Bsl + 'static,
    {
        let mut state = self.switcher_state.lock().await;
        state.stream_servers.push(Box::new(server));
    }

    pub fn create_switcher(&mut self) {
        let switcher = Switcher::new(
            self.username.to_owned(),
            self.broadcasting_software.clone(),
            self.switcher_state.clone(),
            self.broadcast_sender.clone(),
        );

        self.switcher_handler = Some(tokio::spawn(switcher.run()));
    }

    pub fn shutdown_switcher(&mut self) {
        if let Some(handler) = &self.switcher_handler {
            handler.abort();

            // Might not need to do this?
            self.switcher_handler = None;
        }
    }
}