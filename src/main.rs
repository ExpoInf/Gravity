use iced::widget::text_editor;
use iced::{keyboard, Element, Subscription, Task}; // Added Task
use iced::event;
use iced::event::Event;
use std::fs;

#[derive(Default)]
struct Project {
    state: text_editor::Content,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    Save
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
            },
            Message::Save => {
                let file_path = "Gravity_Test.txt";
                println!("Saving {}", file_path);
                fs::write(file_path, state.state.text()).unwrap();
            }
        }
    }
    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _id| {
            match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                                    key,
                                    modifiers,
                                    ..
                                }) => {
                    // 1. Check for Command (Mac) or Control (Windows/Linux)
                    if modifiers.command() {
                        match key {
                            // 2. Match the "s" character specifically.
                            // Note: We check "s" OR "S" to handle Caps Lock or Shift.
                            iced::keyboard::Key::Character(c) if c == "s" || c == "S" => Some(Message::Save),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
    }
    fn init() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }
}

fn main() -> iced::Result {
    iced::application(Project::init, Project::update, Project::view)
        // FIX 1: Explicitly annotate `_state: &Project` here
        .title(|_state: &Project| String::from("Text Editor"))
        .subscription(Project::subscription)
        .run()
}