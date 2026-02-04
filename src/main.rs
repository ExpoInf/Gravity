use iced::widget::scrollable::Scrollbar;
use iced::widget::scrollable::Direction;
use std::env;
use iced::widget::scrollable::Scrollable;
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
use std::process::Command;

struct Project {
    state: text_editor::Content,
    save_path: String,
    shell: String,
    shell_output: Vec<String>,
    shell_path: PathBuf,
    file_tree: Option<FileNode>,
    open_files: Vec<PathBuf>,
    browsing_path: String,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    PathChanged(String),
    Save,
    Test,
    ShellInputChange(String),
    ShellInputSubmit,
    ShellResult(String),
    ToggleFolder(PathBuf),
    OpenFile(PathBuf),
    OpenTab(PathBuf),
    BrowsePathChanged(String),
}

struct FileNode {
    name: String,
    path: PathBuf,
    is_dir: bool,
    children: Vec<FileNode>,
    is_expanded: bool,
}



//FileNode: gifted power
//SimpleFile: pure effort

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
            shell: String::new(),
            shell_output: Vec::new(),
            shell_path: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            file_tree: build_gui_tree("/Users/exi/RustroverProjects/Gravity"),
            open_files: Vec::new(),
            browsing_path: String::from("/"),
        }
    }
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
            rule::Style {
                color: Color::from_rgb8(80, 80, 80),
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
                snap: true, // <--- ADD THIS LINE
            }
        }))
            .padding(3.5);


        let editor = text_editor(&state.state)
            .placeholder("Start typing...")
            .on_action(Message::Edit)
            .height(Length::Fill)
            .style(|_theme, _status| text_editor::Style {
                background: Background::Color(Color::from_rgb8(35, 35, 35)),
                border: Border {
                    radius: 8.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                placeholder: Color::from_rgb8(120, 120, 120),
                value: Color::WHITE,
                selection: Color::from_rgb8(60, 100, 200),
            });
        let tabs = container (
                create_file_tabs(state.open_files.clone())

        )
            .height(50)
            .width(Length::Fill)
            .center_y(50)
            .padding(5)
            .style(|_theme| container::Style{
                background: Some(Background::Color(Color::from_rgb8(30,30,30))),
                border: Border {
                    radius: 8.0.into(),
                    width: 0.0,
                    color: Color::WHITE
                },
                ..Default::default()
            });
        let terminal_log: Scrollable<'_, Message, Theme, iced::Renderer> = scrollable(
            column(
                state.shell_output.iter().map(|line| {
                    text(line)
                        .size(12)
                        .font(iced::font::Font::MONOSPACE)
                        .into()
                })
            )
                .spacing(2)
        )
            .height(Length::Fill)
            .width(Length::Fill);

        let shell = text_input("...", &state.shell)
            .on_input(Message::ShellInputChange)
            .on_submit(Message::ShellInputSubmit)
            .style(|_theme, _status| text_input::Style {
                background: Background::Color(Color::from_rgb8(40, 40, 40)),
                border: Border { radius: 8.0.into(), width: 0.0, color: Color::TRANSPARENT },
                value: Color::WHITE, // Matrix Green text
                placeholder: Color::from_rgb8(80, 80, 80),
                selection: Color::from_rgb8(60, 100, 200),
                icon: Color::WHITE,
            });

        let terminal_panel = container(column![
            terminal_log,
            container(shell).padding(5).style(|_theme: &Theme| container::Style {
                border: Border { width: 0.0, color: Color::from_rgb8(60, 60, 60), radius: 0.0.into() },
                 ..container::Style::default()
            })
        ])
            .height(Length::Fixed(300.0))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgb8(30, 30, 30))),
                text_color: Some(Color::WHITE),
                border: Border {
                    radius: 8.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..container::Style::default()
            });

        let main_content = column![
            tabs,
            editor,
            terminal_panel
        ].spacing(10);

        container(row![
            sidebar,
            divider,
            main_content,
        ]).height(Length::Fill).padding(10).into()



    }

    fn update(state: &mut Project, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                state.state.perform(action);
                Task::none()
            },

            Message::PathChanged(new_path) => {
                state.save_path = new_path;
                Task::none()
            },
            Message::BrowsePathChanged(new_path) => {
                state.file_tree = build_gui_tree(new_path.as_str());
                state.browsing_path = new_path;
                Task::none()
            }
            Message::Save => {
                let file_path = &state.save_path;


                if file_path.trim().is_empty() {
                    println!("Cannot save: Path is empty");
                    return Default::default()
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
                Task::none()
            }
            Message::Test => {
                let _dir_tree = build_gui_tree(state.save_path.as_str()).unwrap();
                Task::none()
            }
            Message::ToggleFolder(path) => {
                if let Some(ref mut root) = state.file_tree {
                    toggle_node(root, &path);
                }
                Task::none()
            }

            Message::OpenFile(path) => {
                println!("Opening file: {:?}", path);
                if let Ok(content) = fs::read_to_string(&path) {
                    state.state = text_editor::Content::with_text(&content);
                    state.save_path = path.display().to_string();
                 }
                state.open_files.push(path);
                //for i in 0..state.open_files.len()-1 {
                    //println!("file tabs test: {:?}", state.open_files[i]);
                //}
                Task::none()
            }
            Message::OpenTab(path) => {
                if let Ok(content) = fs::read_to_string(&path) {
                    state.state = text_editor::Content::with_text(&content);
                    state.save_path = path.display().to_string();
                }
                Task::none()
            }
            Message::ShellInputChange(shell_input) => {
                state.shell = shell_input;
                Task::none()
            }
            Message::ShellInputSubmit => {
                let cmd_text = state.shell.clone();
                if cmd_text.trim().is_empty() { return Task::none(); }

                let prompt_path = state.shell_path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                state.shell_output.push(format!("[{}] $ {}", prompt_path, cmd_text));

                state.shell.clear();

                let parts: Vec<&str> = cmd_text.split_whitespace().collect();
                if let Some(cmd) = parts.first() {
                    if *cmd == "cd" {
                        let new_dir = if parts.len() > 1 {
                            PathBuf::from(parts[1])
                        } else {
                            PathBuf::from("/")
                        };

                        let target_path = if new_dir.is_absolute() {
                            new_dir
                        } else {
                            state.shell_path.join(new_dir)
                        };

                        // Check if it exists before switching
                        match target_path.canonicalize() {
                            Ok(p) => state.shell_path = p,
                            Err(e) => state.shell_output.push(format!("cd: {}", e)),
                        }

                        return Task::none();
                    }
                }

                Task::perform(async move {
                    run_system_command(&cmd_text).await
                }, Message::ShellResult)
            }Message::ShellResult(output) => {
                state.shell_output.push(output);
                Task::none()
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
                            keyboard::Key::Character(c) if c == "s" || c == "S" => Some(Message::Save),
                            keyboard::Key::Character(c) if c == "t" || c == "T" => Some(Message::Test),
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

async fn run_system_command(command: &str) -> String {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() { return String::new(); }

    let program = parts[0];
    let args = &parts[1..];
    
    match Command::new(program).args(args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            format!("{}{}", stdout, stderr)
        },
        Err(e) => format!("Error: {}", e),
    }
}
fn create_file_tabs(file_tabs: Vec<PathBuf>) -> Element<'static, Message> {
    let tabs_row = file_tabs
        .into_iter()
        .fold(row![].spacing(10), |tabs, path| {
            tabs.push(
                button(text(format!("{}", path.display())))
                    .on_press(Message::OpenTab(path))
                    .style(|_theme, status| {
                        button::Style {
                            background: Some(Background::Color(Color::from_rgb8(35, 35, 35))),
                            text_color: Color::WHITE,
                            border: Border { radius: 8.0.into(), ..Default::default() },
                            ..Default::default()
                        }
                    })
            )
        });

    scrollable(tabs_row)
        .direction(Direction::Horizontal(
            Scrollbar::new()
                .width(0)
                .scroller_width(0)
        ))
        .width(Length::Fill)
        .height(Length::Shrink)
        .into()
}
//testing