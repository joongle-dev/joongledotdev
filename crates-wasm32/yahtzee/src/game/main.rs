use super::events::{Event};
use crate::ui::Ui;
use super::Context;

pub struct Main {
    _ui: Ui,
}
impl Main {
    pub fn new(ctx: &mut Context) -> Self {
        let ui = Ui::new();
            ui.div().with_class("row heading").text("Yahtzee!");
            ui.div().with_class("row").text("Enter display name you will join lobby as:");
        {
            let ui = ui.div().with_class("row");
            let event_sender = ctx.event_sender.clone();
            let name_input = ui.text_input().with_callback(move |name| {
                event_sender.queue(Event::SubmitName(name));
            });

            let event_sender = ctx.event_sender.clone();
            ui.button().with_text("Join Lobby").with_callback(move || {
                event_sender.queue(Event::SubmitName(name_input.value()));
            });
        }

        Self {
            _ui: ui,
        }
    }
}
