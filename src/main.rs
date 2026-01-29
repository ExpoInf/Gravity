//TO-DO:
//1. make GUI non-shit
//2. syntax highlighting
//3. ctrl-f
//make an icon
use iced::widget::{column, container, row, text, text_editor, text_input, rule, scrollable, button, };
use iced::{keyboard, Background, Border, Color, Element, Length, Subscription, Task, Theme, Padding};
use iced::event;
use iced::event::Event;
use std::fs;
use walkdir::WalkDir;
use std::path::PathBuf;
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::Write;

struct Project {
    state: text_editor::Content,
    save_path: String,
    file_tree: Option<FileNode>,
    browsing_path: String,
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


    for entry in WalkDir::new(&root_buf).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        let is_dir = entry.file_type().is_dir();
        nodes.insert(path.clone(), FileNode::new(path, is_dir));
    }

    let mut paths: Vec<PathBuf> = nodes.keys().cloned().collect();

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

    nodes.remove(&root_buf).map(|mut root| {
        root.sort_children();
        root
    })
}

impl Default for Project {
    fn default() -> Self {
        Self {
            state: text_editor::Content::default(),
            save_path: String::from("Gravity_Test.txt"),
            file_tree: build_gui_tree("/Users/exi/RustroverProjects/Gravity"),
            browsing_path: String::from("/"),
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
    BrowsePathChanged(String),
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

        let sidebar_content = column![
            text("Save Directory/File:"),
text_input("path/to/file.txt", &state.save_path)
.on_input(Message::PathChanged)
                .padding(10)
                .style(|_theme, _status| text_input::Style {
                    // 1. Background
                    background: Background::Color(Color::from_rgb8(40, 40, 40)),

                    // 2. Border
                    border: Border {
                        radius: 8.0.into(),
                        width: 0.0,
                        color: Color::from_rgb8(80, 80, 80),
                    },

                    // 3. Text & Cursor Colors
                    icon: Color::from_rgb8(120, 120, 120),
                    value: Color::WHITE,
                    placeholder: Color::from_rgb8(120, 120, 120),
                    selection: Color::from_rgb8(60, 100, 200),
                }),


            text("Save Directory/File:"),
            text_input("Path to browse:", &state.browsing_path)
                .on_input(Message::BrowsePathChanged)
                .padding(10)
            .on_input(Message::PathChanged)
                .padding(10)
                .style(|_theme, _status| text_input::Style {
                    // 1. Background
                    background: Background::Color(Color::from_rgb8(40, 40, 40)),

                    // 2. Border
                    border: Border {
                        radius: 8.0.into(),
                        width: 0.0,
                        color: Color::from_rgb8(80, 80, 80),
                    },

                    // 3. Text & Cursor Colors
                    icon: Color::from_rgb8(120, 120, 120), // <--- Added missing Icon color
                    value: Color::WHITE,
                    placeholder: Color::from_rgb8(120, 120, 120),
                    selection: Color::from_rgb8(60, 100, 200)
                      }),
            tree_view
        ]
            .spacing(10);
        let sidebar = container(sidebar_content)
            .padding(10)
            .width(225)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(Color::from_rgb8(30, 30, 30))),
                text_color: Some(Color::WHITE),

                border: Border {
                    color: Color::from_rgb8(35, 35, 35),
                    width: 2.0,
                    radius: 8.0.into()
                },
                ..Default::default()
            });


        let divider = container(rule::vertical(0).style(|_theme| {
            iced::widget::rule::Style {
                color: Color::from_rgb8(80, 80, 80),
                radius: 0.0.into(),
                fill_mode: iced::widget::rule::FillMode::Full,
                snap: true, // <--- ADD THIS LINE
            }
        }))
            .padding(3.5);


        let editor = text_editor(&state.state)
            .placeholder("Type something here...")
            .on_action(Message::Edit)
            .height(Length::Fill);

        container(row![
            sidebar,
            divider,
            editor
        ]).height(Length::Fill).padding(10).into()
    }

    fn update(state: &mut Project, message: Message) {
        match message {
            Message::Edit(action) => {
                state.state.perform(action);
            },

            Message::PathChanged(new_path) => {
                state.save_path = new_path;
            },
            Message::BrowsePathChanged(new_path) => {
                state.file_tree = build_gui_tree(new_path.as_str());
                state.browsing_path = new_path;
            }
            Message::Save => {
                let file_path = &state.save_path;


                if file_path.trim().is_empty() {
                    println!("Cannot save: Path is empty");
                    return;
                }

                println!("Saving to: {}", file_path);

                if Path::new(&file_path).exists() {
                    match fs::write(file_path, state.state.text()) {
                        Ok(_) => println!("Successfully saved."),
                        Err(e) => println!("Failed to save: {}", e),
                    }
                } else {
                    let mut file = File::create(&file_path).expect("Failed to create file");
                    file.write_all(state.state.text().as_ref()).expect("Failed to write file");
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
                    state.save_path = path.display().to_string();
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

        let content = button(text(format!("{}{}", icon, node.name)))
            .on_press(if node.is_dir {
                Message::ToggleFolder(node.path.clone())
            } else {
                Message::OpenFile(node.path.clone())
            })
            .style(button::text)
            .padding(5)
            .width(Length::Fill);

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
//testing