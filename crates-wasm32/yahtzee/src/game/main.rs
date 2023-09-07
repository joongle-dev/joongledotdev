use crate::ui::Ui;
use crate::event_loop::EventSender;
use crate::game::connecting::Connecting;
use crate::game::GameState;
use super::events::{GameEvent};

pub struct Main {
    _ui: Ui,
}
impl Main {
    pub fn new(event_sender: EventSender<GameEvent>) -> Self {
        let ui = Ui::new();
            ui.div().with_class("row heading").text("Yahtzee!");
            ui.div().with_class("row").text("Enter display name you will join lobby as:");
        {
            let ui = ui.div().with_class("row");
            let event_sender_clone = event_sender.clone();
            let name_input = ui.text_input().with_callback(move |name| {
                event_sender_clone.send(GameEvent::ChangeGameState(GameState::Connecting(
                    Connecting::new(event_sender_clone.clone(), name)
                )));
            });

            ui.button().with_text("Join Lobby").with_callback(move || {
                event_sender.send(GameEvent::ChangeGameState(GameState::Connecting(
                    Connecting::new(event_sender.clone(), name_input.value())
                )));
            });
        }

        Self {
            _ui: ui,
        }
    }
}
