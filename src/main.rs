mod config_lib;

use crate::config_lib::{load_config, AppConfig};
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{
    button, column, container, row, rule, scrollable, text, text_editor, text_input, Scrollable,
};
use iced::{
    event, keyboard, window, Background, Border, Color, Element, Length, Padding, Subscription,
    Task, Theme,
};
use iced::mouse;
use iced::event::Event;
use image::GenericImageView;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

struct Project {
    state: text_editor::Content,
    save_path: String,
    shell: String,
    shell_output: Vec<String>,
    shell_path: PathBuf,
    file_tree: Option<FileNode>,
    open_files: Vec<PathBuf>,
    browsing_path: String,
    sidebar_width: f32,
    terminal_height: f32,
    is_resizing_sidebar: bool,
    is_resizing_terminal: bool,
    last_cursor_pos: Option<iced::Point>,
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
    OpenPicker,
    PickerResult(Option<PathBuf>),
    StartResizingSidebar,
    StartResizingTerminal,
    StopResizing,
    CursorMoved(iced::Point),
    CloseTab(PathBuf),
}

#[derive(Debug, Clone)]
struct FileNode {
    name: String,
    path: PathBuf,
    is_dir: bool,
    children: Vec<FileNode>,
    is_expanded: bool,
    scanned: bool,
}

static APP_CONFIG: LazyLock<AppConfig> = LazyLock::new(|| {
    load_config().expect("Could not read settings")
});

impl FileNode {
    fn new(path: PathBuf, is_dir: bool) -> Self {
        Self {
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            path,
            is_dir,
            children: Vec::new(),
            is_expanded: false,
            scanned: false,
        }
    }
}

fn read_dir_shallow(root: &Path) -> Vec<FileNode> {
    let mut nodes = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            nodes.push(FileNode::new(path.clone(), path.is_dir()));
        }
    }
    nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    nodes
}

fn build_root_node(root_path: &str) -> Option<FileNode> {
    let path = PathBuf::from(root_path);
    if !path.exists() { return None; }

    let is_dir = path.is_dir();
    let mut root = FileNode::new(path.clone(), is_dir);

    if is_dir {
        root.children = read_dir_shallow(&path);
        root.is_expanded = true;
        root.scanned = true;
    }

    Some(root)
}

impl Default for Project {
    fn default() -> Self {
        let default_str = fs::read_to_string("path.txt")
        .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| {
                env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .to_string_lossy()
                    .to_string()
            });

        let default_path = PathBuf::from(&default_str);

        Self {
            state: text_editor::Content::default(),
            save_path: String::from("Gravity_Test.txt"),
            shell: String::new(),
            shell_output: Vec::new(),
            shell_path: default_path.clone(),
            file_tree: build_root_node(&default_str),
            open_files: Vec::new(),
            browsing_path: default_str,
            sidebar_width: 225.0,
            terminal_height: 300.0,
            is_resizing_sidebar: false,
            is_resizing_terminal: false,
            last_cursor_pos: None,
        }
    }
}

impl Project {
    fn view(state: &Project) -> Element<'_, Message> {
        let tree_view = if let Some(root) = &state.file_tree {
            container(scrollable(Self::view_file_tree(root))).height(Length::Fill)
        } else {
            container(text("No folder open"))
        };

        let sidebar_content = column![
            text("Current File:"),
            text_input("path/to/file.txt", &state.save_path)
                .on_input(Message::PathChanged)
                .padding(10)
                .style(|_theme, _status| text_input::Style {
                    background: Background::Color(Color::from_rgb8(40, 40, 40)),
                    border: Border { radius: 8.0.into(), width: 0.0, color: Color::from_rgb8(80, 80, 80) },
                    icon: Color::from_rgb8(120, 120, 120),
                    value: Color::WHITE,
                    placeholder: Color::from_rgb8(120, 120, 120),
                    selection: Color::from_rgb8(60, 100, 200),
                }),
            text("Browse Directory:"),

            row!
            [container(scrollable(text(&state.browsing_path).size(15))
            .direction(Direction::Horizontal(
                Scrollbar::new().width(0).scroller_width(0)
            ))
            .width(Length::Fill))
        .padding(10)
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border {
                radius: 8.0.into(),
                width: 0.0,
                color: Color::from_rgb8(80, 80, 80)
            },
            ..Default::default()
        }),
                button(text("📂"))
                    .on_press(Message::OpenPicker)
                    .padding(10)
                    .style(|_theme, _status| button::Style {
                         background: Some(Background::Color(Color::from_rgb8(50, 50, 50))),
                         text_color: Color::WHITE,
                         border: Border { radius: 8.0.into(), ..Default::default() },
                         ..Default::default()
                    })
            ].spacing(5),

            tree_view
        ].spacing(10);

        let sidebar = container(sidebar_content)
            .padding(10)
            .width(275)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(Color::from_rgb8(30, 30, 30))),
                text_color: Some(Color::WHITE),
                border: Border { color: Color::from_rgb8(35, 35, 35), width: 2.0, radius: 8.0.into() },
                ..Default::default()
            });

        let divider = container(rule::vertical(0).style(|_theme| rule::Style {
            color: Color::from_rgb8(80, 80, 80),
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        })).padding(3.5);

        let editor = text_editor(&state.state)
            .placeholder("Start typing...")
            .on_action(Message::Edit)
            .height(Length::Fill)
            .style(|_theme, _status| text_editor::Style {
                background: Background::Color(Color::from_rgb8(35, 35, 35)),
                border: Border { radius: 8.0.into(), width: 0.0, color: Color::TRANSPARENT },
                placeholder: Color::from_rgb8(120, 120, 120),
                value: Color::WHITE,
                selection: Color::from_rgb8(60, 100, 200),
            });

        let tabs = container(create_file_tabs(state.open_files.clone()))
            .height(50)
            .width(Length::Fill)
            .center_y(50)
            .padding(5)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgb8(30, 30, 30))),
                border: Border { radius: 8.0.into(), width: 0.0, color: Color::WHITE },
                ..Default::default()
            });

        let terminal_log: Scrollable<'_, Message, Theme, iced::Renderer> = scrollable(
            column(state.shell_output.iter().map(|line| {
                text(line).size(12).font(iced::font::Font::MONOSPACE).into()
            })).spacing(2)
        ).height(Length::Fill).width(Length::Fill);

        let shell = text_input("...", &state.shell)
            .on_input(Message::ShellInputChange)
            .on_submit(Message::ShellInputSubmit)
            .style(|_theme, _status| text_input::Style {
                background: Background::Color(Color::from_rgb8(APP_CONFIG.accent_r, 40, 40)),
                border: Border { radius: 8.0.into(), width: 0.0, color: Color::TRANSPARENT },
                value: Color::WHITE,
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
                border: Border { radius: 8.0.into(), width: 0.0, color: Color::TRANSPARENT },
                ..container::Style::default()
            });

        let main_content = column![tabs, editor, terminal_panel].spacing(10);

        container(row![sidebar, divider, main_content])
            .height(Length::Fill)
            .padding(10)
            .into()
    }

    fn update(state: &mut Project, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                state.state.perform(action);
                Task::none()
            }
            Message::PathChanged(new_path) => {
                state.save_path = new_path;
                Task::none()
            }
            Message::BrowsePathChanged(new_path) => {
                state.browsing_path = new_path.clone();
                state.file_tree = build_root_node(&new_path);
                Task::none()
            }
            Message::OpenPicker => {
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Open Project Folder")
                            .pick_folder()
                            .await
                            .map(|handle| handle.path().to_path_buf())
                    },
                    Message::PickerResult
                )
            }
            Message::PickerResult(Some(path)) => {
                let path_str = path.to_string_lossy().to_string();
                fs::write("path.txt", path.to_string_lossy().to_string()).unwrap();
                state.browsing_path = path_str.clone();
                state.file_tree = build_root_node(&path_str);
                Task::none()
            }
            Message::PickerResult(None) => Task::none(),

            Message::Save => {
                let file_path = &state.save_path;
                if !file_path.trim().is_empty() {
                    if Path::new(&file_path).exists() {
                        let _ = fs::write(file_path, state.state.text());
                    } else {
                        if let Ok(mut file) = File::create(&file_path) {
                            let _ = file.write_all(state.state.text().as_ref());
                        }
                    }
                }
                Task::none()
            }
            Message::Test => Task::none(),
            Message::ToggleFolder(path) => {
                if let Some(ref mut root) = state.file_tree {
                    toggle_and_scan(root, &path);
                }
                Task::none()
            }
            Message::OpenFile(path) => {
                if let Ok(content) = fs::read_to_string(&path) {
                    state.state = text_editor::Content::with_text(&content);
                    state.save_path = path.display().to_string();
                }
                if tab_scan(state.open_files.clone(), path.clone()) {
                    state.open_files.push(path);
                }
                Task::none()
            }
            Message::OpenTab(path) => {
                if let Ok(content) = fs::read_to_string(&path) {
                    state.state = text_editor::Content::with_text(&content);
                    state.save_path = path.display().to_string();
                }
                Task::none()
            }
            Message::ShellInputChange(input) => {
                state.shell = input;
                Task::none()
            }
            Message::ShellInputSubmit => {
                let cmd_text = state.shell.clone();
                if cmd_text.trim().is_empty() { return Task::none(); }

                state.shell_output.push(format!("$ {}", cmd_text));
                state.shell.clear();

                let parts: Vec<&str> = cmd_text.split_whitespace().collect();
                if parts.first() == Some(&"cd") {
                    return Task::none();
                }

                Task::perform(async move { run_system_command(&cmd_text).await }, Message::ShellResult)
            }
            Message::ShellResult(output) => {
                state.shell_output.push(output);
                Task::none()
            }
            Message::StartResizingSidebar => {
                state.is_resizing_sidebar = true;
                Task::none()
            }
            Message::StartResizingTerminal => {
                state.is_resizing_terminal = true;
                Task::none()
            }
            Message::StopResizing => {
                state.is_resizing_sidebar = false;
                state.is_resizing_terminal = false;
                Task::none()
            }
            Message::CursorMoved(pos) => {

                if state.is_resizing_sidebar {
                    state.sidebar_width = pos.x.clamp(150.0, 800.0);
                }


                if state.is_resizing_terminal {
                    if let Some(last_pos) = state.last_cursor_pos {
                        let delta_y = last_pos.y - pos.y;
                        state.terminal_height = (state.terminal_height + delta_y).clamp(100.0, 800.0);
                    }
                }


                state.last_cursor_pos = Some(pos);
                Task::none()
            }
            Message::CloseTab(path) => {
                if let Some(index) = state.open_files.iter().position(|r| *r == path) {
                    state.open_files.remove(index);
                }

                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _id| {
            if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
                if modifiers.command() {
                    match key {
                        keyboard::Key::Character(c) if c == "s" || c == "S" => Some(Message::Save),
                        _ => None,
                    }
                } else { None }
            } else { None }
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

        if node.is_expanded {
            let mut col = column![content];
            for child in &node.children {
                col = col.push(
                    container(Self::view_file_tree(child))
                        .padding(Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 15.0 })
                );
            }
            col.into()
        } else {
            content.into()
        }
    }
}

fn load_icon(path: &str) -> Option<window::Icon> {
    let img = image::open(path).ok()?;
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();
    window::icon::from_rgba(rgba, width, height).ok()
}

fn toggle_and_scan(node: &mut FileNode, target_path: &PathBuf) {
    if node.path == *target_path {
        if node.is_dir {
            if !node.is_expanded {
                if !node.scanned {
                    node.children = read_dir_shallow(&node.path);
                    node.scanned = true;
                }
                node.is_expanded = true;
            } else {
                node.is_expanded = false;
            }
        }
    } else if node.is_dir && node.is_expanded {
        for child in &mut node.children {
            toggle_and_scan(child, target_path);
        }
    }
}

async fn run_system_command(command: &str) -> String {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() { return String::new(); }
    match Command::new(parts[0]).args(&parts[1..]).output() {
        Ok(out) => format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)),
        Err(e) => format!("Error: {}", e),
    }
}

fn create_file_tabs(file_tabs: Vec<PathBuf>) -> Element<'static, Message> {
    let tabs_row = file_tabs.into_iter().fold(row![].spacing(10), |tabs, path| {
        tabs.push(
            container (
                row! {
                button(text(format ! ("{}", path.display())))
                .on_press(Message::OpenTab(path.clone()))
                .style( | _theme, _status | button::Style {
                background: Some(Background::Color(Color::from_rgb8(35, 35, 35))),
                text_color: Color::WHITE,
                border: Border { radius: 8.0.into(), ..Default::default() },
                ..Default::default()
                }),
                button(text("✕"))
                .on_press(Message::CloseTab(path))
                .style( | _theme, _status | button::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                text_color: Color::from_rgb8(120, 120, 120),
                border: Border { radius: 8.0.into(), ..Default::default() },
                ..Default::default()
                })
                }
            ).style( | _theme| container::Style {
                background: Some(Background::Color(Color::from_rgb8(35, 35, 35))),
                text_color: Some(Color::WHITE),
                border: Border { radius: 8.0.into(), ..Default::default() },
                ..Default::default()
            })
        )
    });

    scrollable(tabs_row)
        .direction(Direction::Horizontal(Scrollbar::new().width(0).scroller_width(0)))
        .width(Length::Fill)
        .height(Length::Shrink)
        .into()
}


fn tab_scan(file_tabs: Vec<PathBuf>, path: PathBuf) -> bool {
    if file_tabs.contains(&path) {
        false
    } else {
        true
    }
}

fn main() -> iced::Result {
    let icon = load_icon("final_icon.png");
    iced::application(Project::init, Project::update, Project::view)
        .title(|_state: &Project| String::from("Gravity Editor"))
        .subscription(Project::subscription)
        .window(window::Settings {
            icon,
            min_size: Some((800.0, 600.0).into()),
            ..Default::default()
        })
        .run()
}