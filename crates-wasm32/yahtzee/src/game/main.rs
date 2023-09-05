use super::events::Event;
use crate::platform::EventHandlerProxy;
use crate::ui::{Ui};

pub struct Main {
    _ui: Ui,
}
impl Main {
    pub fn new(event_handler: EventHandlerProxy<Event>) -> Self {
        let ui = Ui::new();
            ui.div().with_class("row heading").text("Yahtzee!");
            ui.div().with_class("row").text("Enter display name you will join lobby as:");
        {
            let ui = ui.div().with_class("row");
            let event_handler_clone = event_handler.clone();
            let name_input = ui.text_input().with_callback(move |name| {
                event_handler_clone.call(Event::JoinLobby(name));
            });

            let event_handler_clone = event_handler.clone();
            ui.button().with_text("Join Lobby").with_callback(move || {
                event_handler_clone.call(Event::JoinLobby(name_input.value()));
            });
        }

        Self {
            _ui: ui,
        }
    }
}
