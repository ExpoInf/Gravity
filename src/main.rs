use iced::widget::{text_editor};
use iced::Element;

#[derive(Default)]
struct Project {
    state: text_editor::Content,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action)
}
impl Project {
    fn view(state: &Project) -> Element<'_, Message> {
        text_editor(&state.state)
            .placeholder("Type something here...")
            .on_action(Message::Edit)
            .into()

    }

    fn update(state: &mut Project, message: Message) {
        match message {
            Message::Edit(action) => {
                state.state.perform(action);
            }
        }
    }
}

fn main() -> iced::Result {
    iced::run(Project::update, Project::view)
}