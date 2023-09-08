use crate::ui::Ui;
use crate::event_loop::EventSender;
use crate::game::events::GameEvent;
use super::{GameScene, connecting::Connecting};

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
                event_sender_clone.send(GameEvent::ChangeGameScene(Box::new(
                    Connecting::new(event_sender_clone.clone(), name)
                )));
            });
            name_input.clone().focus();

            ui.button().with_text("Join Lobby").with_callback(move || {
                event_sender.send(GameEvent::ChangeGameScene(Box::new(
                    Connecting::new(event_sender.clone(), name_input.value())
                )));
            });
        }

        Self {
            _ui: ui,
        }
    }
}
impl GameScene for Main {
    fn update(&mut self, _time: f64) {}

    fn handle_event(&mut self, _event: GameEvent) {}
}
