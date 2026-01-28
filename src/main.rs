use iced::widget::{column, row, text, text_editor, text_input, button, container, scrollable}; // Added widget imports
use iced::{keyboard, Element, Length, Subscription, Task, Padding};
use iced::event;
use iced::event::Event;
use std::fs;
use walkdir::WalkDir;
use std::path::PathBuf;
use std::collections::HashMap;


struct Project {
    state: text_editor::Content,
    save_path: String, // 1. Added field to store the file path
    file_tree: Option<FileNode>,
}

struct FileNode {
    name: String,
    path: PathBuf,
    is_dir: bool,
    children: Vec<FileNode>,
    is_expanded: bool,
}

impl FileNode {
    fn new(path: PathBuf, is_dir: bool) -> Self {
        Self {
            name: path.file_name().unwrap_or_default().to_string_lossy().into_owned(),
            path,
            is_dir,
            children: Vec::new(),
            is_expanded: false,
        }
    }
    fn sort_children(&mut self) {
        self.children.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
    }
}

fn build_gui_tree(root_path: &str) -> Option<FileNode> {
    let mut nodes: HashMap<PathBuf, FileNode> = HashMap::new();
    let root_buf = PathBuf::from(root_path).canonicalize().ok()?;

    // 1. Fill the Map
    for entry in WalkDir::new(&root_buf).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        let is_dir = entry.file_type().is_dir();
        nodes.insert(path.clone(), FileNode::new(path, is_dir));
    }

    // 2. Link children (This must be AFTER the loop above)
    let mut paths: Vec<PathBuf> = nodes.keys().cloned().collect();
    // Sort by path depth so we move children into parents from the bottom up
    paths.sort_by_key(|p| std::cmp::Reverse(p.components().count()));

    for path in paths {
        if path == root_buf { continue; }

        if let Some(parent_path) = path.parent() {
            if let Some(mut child_node) = nodes.remove(&path) {
                child_node.sort_children();
                if let Some(parent_node) = nodes.get_mut(parent_path) {
                    parent_node.children.push(child_node);
                }
            }
        }
    }

    // 3. Final Expression (The return value)
    nodes.remove(&root_buf).map(|mut root| {
        root.sort_children();
        root
    }) // <--- No semicolon here means this is the return value
}

impl Default for Project {
    fn default() -> Self {
        Self {
            state: text_editor::Content::default(),
            save_path: String::from("Gravity_Test.txt"), // Default startup path
            file_tree: build_gui_tree("/Users/exi/RustroverProjects/Gravity")
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    PathChanged(String),
    Save,
    Test,
    ToggleFolder(PathBuf),
    OpenFile(PathBuf),
}

impl Project {
    fn view(state: &Project) -> Element<'_, Message> {
        let tree_view = if let Some(root) = &state.file_tree {
            container(
                scrollable(
                    Self::view_file_tree(root)
                )
            )
                .height(Length::Fill)
        } else {
            container(text("No folder open"))
        };

        let sidebar = column![
            text("Save Directory/File:"),
            text_input("path/to/file.txt", &state.save_path)
                .on_input(Message::PathChanged)
                .padding(10),
            tree_view
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
            Message::Test => {
                let _dir_tree = build_gui_tree(state.save_path.as_str()).unwrap();
            }
            Message::ToggleFolder(path) => {
                if let Some(ref mut root) = state.file_tree {
                    toggle_node(root, &path);
                }
            }

            Message::OpenFile(path) => {
                println!("Opening file: {:?}", path);
                if let Ok(content) = fs::read_to_string(&path) {
                    state.state = text_editor::Content::with_text(&content);
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
                            iced::keyboard::Key::Character(c) if c == "t" || c == "T" => Some(Message::Test),
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
    fn view_file_tree(node: &FileNode) -> Element<'_, Message> {
        let icon = if node.is_dir {
            if node.is_expanded { "▼ 📂 " } else { "▶ 📁 " }
        } else {
            "  📄 "
        };

        // 2. Now 'icon' is in scope for the button
        let content = button(text(format!("{}{}", icon, node.name)))
            .on_press(if node.is_dir {
                Message::ToggleFolder(node.path.clone())
            } else {
                Message::OpenFile(node.path.clone())
            })
            .style(button::text)  // Or button::text in v0.12+
            .padding(5)
            .width(Length::Fill);

        // 3. Recursive rendering
        if node.is_expanded && !node.children.is_empty() {
            let mut col = column![content];

            for child in &node.children {
                col = col.push(
                    container(Self::view_file_tree(child))
                        .padding(Padding {
                            top: 0.0,
                            right: 0.0,
                            bottom: 0.0,
                            left: 15.0,
                        })
                );
            }
            col.into()
        } else {
            content.into()
        }
    }
}

fn main() -> iced::Result {
    iced::application(Project::init, Project::update, Project::view)
        .title(|_state: &Project| String::from("Gravity Editor"))
        .subscription(Project::subscription)
        .run()
}


fn toggle_node(node: &mut FileNode, target_path: &PathBuf) {
    if node.path == *target_path {
        node.is_expanded = !node.is_expanded;
    } else if node.is_dir {
        for child in &mut node.children {
            toggle_node(child, target_path);
        }
    }
}
