use iced::widget::{column, row, text, text_editor, text_input}; // Added widget imports
use iced::{keyboard, Element, Length, Subscription, Task};
use iced::event;
use iced::event::Event;
use std::fs;

struct Project {
    state: text_editor::Content,
    save_path: String, // 1. Added field to store the file path
}

// We implement Default manually to set a starting filename
impl Default for Project {
    fn default() -> Self {
        Self {
            state: text_editor::Content::default(),
            save_path: String::from("Gravity_Test.txt"), // Default startup path
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    PathChanged(String),
    Save,
}

impl Project {
    fn view(state: &Project) -> Element<'_, Message> {
        // 3. Define the Sidebar
        let sidebar = column![
            text("Save Directory/File:"),
            text_input("path/to/file.txt", &state.save_path)
                .on_input(Message::PathChanged)
                .padding(10)
        ]
            .spacing(10)
            .padding(10)
            .width(Length::Fixed(200.0));

        let editor = text_editor(&state.state)
            .placeholder("Type something here...")
            .on_action(Message::Edit)
            .height(Length::Fill);

        row![
            sidebar,
            editor
        ]
            .into()
    }

    fn update(state: &mut Project, message: Message) {
        match message {
            Message::Edit(action) => {
                state.state.perform(action);
            },

            Message::PathChanged(new_path) => {
                state.save_path = new_path;
            },
            Message::Save => {

                let file_path = &state.save_path;


                if file_path.trim().is_empty() {
                    println!("Cannot save: Path is empty");
                    return;
                }

                println!("Saving to: {}", file_path);


                match fs::write(file_path, state.state.text()) {
                    Ok(_) => println!("Successfully saved."),
                    Err(e) => println!("Failed to save: {}", e),
                }
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
                    if modifiers.command() {
                        match key {
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
        .title(|_state: &Project| String::from("Gravity Editor"))
        .subscription(Project::subscription)
        .run()
}