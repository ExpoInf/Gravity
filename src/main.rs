mod config_lib;

use iced::alignment;
use crate::config_lib::{load_config, AppConfig};
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{button, column, container, mouse_area, row, scrollable, text, text_editor, text_input, Space};
use iced::{ event, keyboard, window, Background, Border, Color, Element, Length, Padding, Subscription, Task, Theme};
use iced::mouse;
use iced::event::Event;
use image::GenericImageView;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;
use std::collections::HashMap;
use iced::font::{Family, Font, Weight, Stretch, Style};

struct Project {
    state: text_editor::Content,
    save_path: String,
    file_tree: Option<FileNode>,
    open_files: Vec<PathBuf>,
    browsing_path: String,
    sidebar_width: f32,
    terminal_height: f32,
    is_resizing_sidebar: bool,
    is_resizing_terminal: bool,
    last_cursor_pos: Option<iced::Point>,
    terminal: iced_term::Terminal,
    background_tabs: HashMap<PathBuf, text_editor::Content>,
    dynamic_width: f32,
}

#[derive(Debug, Clone)]
enum SidebarStates {
    ProjectDir,
    GitManager,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    PathChanged(String),
    Save,
    Test,
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
    TerminalEvent(iced_term::Event),
    DynamicIslandIncrease,
    SidebarStateChange(SidebarStates),
    TerminalViewEvent(iced_term::Event),
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

const NERD_FONT: Font = Font {
    family: Family::Name("JetBrainsMono Nerd Font"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

const INTER: Font = Font {
    family: Family::Name("Inter 24pt, Medium"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

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
        let default_str = directories::UserDirs::new()
            .map(|user_dirs| user_dirs.home_dir().join(".config").join("gravity").join("path.txt"))
            .and_then(|path_file| fs::read_to_string(path_file).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| {
                env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .to_string_lossy()
                    .to_string()
            });

        let system_shell = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));

        let term_settings = iced_term::settings::Settings {
            backend: iced_term::settings::BackendSettings {
                program: system_shell,
                args: vec![],
                ..Default::default()
            },
            theme: iced_term::settings::ThemeSettings {
                color_pallete: Box::new(iced_term::ColorPalette {
                    background: String::from("#1e1e1e"),
                    foreground: String::from("#c8c8c8"),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        Self {
            state: text_editor::Content::default(),
            save_path: String::from("Gravity_Test.txt"),
            file_tree: build_root_node(&default_str),
            open_files: Vec::new(),
            browsing_path: default_str,
            sidebar_width: 225.0,
            terminal_height: 300.0,
            is_resizing_sidebar: false,
            is_resizing_terminal: false,
            last_cursor_pos: None,
            terminal: iced_term::Terminal::new(0, term_settings).expect("Failed to init terminal"),
            background_tabs: HashMap::new(),
            dynamic_width: 10.0,
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

        let tabs = container(create_file_tabs(state.open_files.clone(), &state.save_path))
            .height(50)
            .width(Length::Fill)
            .center_y(50)
            .padding(5)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgb8(30, 30, 30))),
                border: Border { radius: 8.0.into(), width: 0.0, color: Color::WHITE },
                ..Default::default()
            });

        let terminal_divider: Element<'_, Message> = mouse_area(
                                                                 container(Space::new().width(Length::Fill).height(8.0))
                                                                     .style(|_theme| container::Style {
                                                                         background: Some(Background::Color(Color::TRANSPARENT)),
                                                                         ..Default::default()
                                                                     })
        )
            .on_press(Message::StartResizingTerminal)
            .interaction(iced::mouse::Interaction::ResizingVertically)
            .into();

        let sidebar_divider: Element<'_, Message> = mouse_area(
                                                                container(Space::new().width(8.0).height(Length::Fill))
                                                                    .style(|_theme| container::Style {
                                                                        background: Some(Background::Color(Color::TRANSPARENT)),
                                                                        ..Default::default()
                                                                    })
        )
            .on_press(Message::StartResizingSidebar)
            .interaction(iced::mouse::Interaction::ResizingHorizontally)
            .into();



        let icon = text("\u{f07b}")
            .font(NERD_FONT)
            .size(14)
            .line_height(text::LineHeight::Relative(1.0))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center);

        let sidebar_buttons = column!(
            button(icon)
            .on_press(Message::SidebarStateChange(SidebarStates::GitManager))
            .width(Length::Fixed(30.0))
            .height(Length::Fixed(30.0))
            .padding(Padding{
                top: 0.0,
                left: 0.0,
                right: 4.0,
                bottom: 0.0,

            })
            .style(|_theme, _status| button::Style {
                         background: Some(Background::Color(Color::from_rgb8(60, 60, 60))),
                         text_color: Color::WHITE,
                         border: Border { radius: 5.5.into(), ..Default::default() },
                         ..Default::default()
                    })
        );

        let sidebar_content = column![
            text("Current File:").font(INTER),
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
            text("Browse Directory:").font(INTER),

            row!
            [container(scrollable(text(&state.browsing_path).size(15).font(INTER))
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


                button(text("\u{f07b}")
            .font(NERD_FONT)
            .size(14)
            .line_height(text::LineHeight::Relative(1.0))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            )
                .width(Length::Fixed(35.0))
                .height(Length::Fixed(35.0))
                    .on_press(Message::OpenPicker)
                    .padding(Padding{
                    top: 0.0,
                    left: 0.0,
                    right: 4.0,
                    bottom: 0.0,

            })
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
            .height(Length::Fill)
            .width(Length::Fixed(state.sidebar_width))
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(Color::from_rgb8(30, 30, 30))),
                text_color: Some(Color::WHITE),
                border: Border { color: Color::from_rgb8(35, 35, 35), width: 2.0, radius: 8.0.into() },
                ..Default::default()
            });

        let terminal_panel = container(
            iced_term::TerminalView::show(&state.terminal)
                .map(Message::TerminalViewEvent)
        )
            .padding(5)
            .height(Length::Fixed(state.terminal_height))
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgb8(30, 30, 30))),
                border: Border { color: Color::TRANSPARENT, width: 2.0, radius: 8.0.into()},
                ..Default::default()
            });


        //let top = row![tabs, dynamic_container];

        let main_content = column![
            tabs,
            Space::new().height(10.0),
            editor,
            terminal_divider,
            terminal_panel
        ].spacing(0);

        container(row![
            sidebar_buttons,
            Space::new().width(5.0),
            sidebar,
            sidebar_divider,
            main_content
        ])
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(5)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgb8(50, 48, 51))),
                text_color: Some(Color::WHITE),
                ..Default::default()
            })
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

                if let Some(user_dirs) = directories::UserDirs::new() {
                    let config_dir = user_dirs.home_dir().join(".config").join("gravity");

                    let _ = fs::create_dir_all(&config_dir);
                    let _ = fs::write(config_dir.join("path.txt"), &path_str);
                }

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
            Message::OpenFile(path) | Message::OpenTab(path) => {
                let current_path = PathBuf::from(&state.save_path);

                if current_path == path {
                    return Task::none();
                }

                if !state.save_path.is_empty() {
                    let mut parked_content = text_editor::Content::new();
                    std::mem::swap(&mut state.state, &mut parked_content);
                    state.background_tabs.insert(current_path, parked_content);
                }

                if let Some(mut existing_content) = state.background_tabs.remove(&path) {
                    std::mem::swap(&mut state.state, &mut existing_content);
                    state.save_path = path.display().to_string();
                } else {
                    if let Ok(content) = fs::read_to_string(&path) {
                        state.state = text_editor::Content::with_text(&content);
                        state.save_path = path.display().to_string();
                    }
                }

                if !state.open_files.contains(&path) {
                    state.open_files.push(path);
                }

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
                state.background_tabs.remove(&path);

                if state.save_path == path.display().to_string() {
                    if let Some(next_tab) = state.open_files.last().cloned() {
                        return Task::perform(async move { next_tab }, Message::OpenTab);
                    } else {
                        state.state = text_editor::Content::new();
                        state.save_path = String::new();
                    }
                }

                Task::none()
            }
            Message::TerminalEvent(iced_term::Event::BackendCall(_, cmd)) => {
                match state.terminal.handle(iced_term::Command::ProxyToBackend(cmd)) {
                    iced_term::actions::Action::Shutdown => println!("Terminal closed!"),
                    _ => {}
                }
                Task::none()
            }
            Message::DynamicIslandIncrease => {
                for i in 0..50 {
                    state.dynamic_width = state.dynamic_width + 1.0;
                }

                Task::none()
            }
            Message::SidebarStateChange(sidebar_state) => {
                Task::none()
            }
            Message::TerminalViewEvent(event) => {
                if let iced_term::Event::BackendCall(_, cmd) = event {
                    match state.terminal.handle(iced_term::Command::ProxyToBackend(cmd)) {
                        iced_term::actions::Action::Shutdown => {
                            println!("Terminal closed!");
                        },
                        _ => {}
                    }
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            event::listen_with(|event, _status, _id| {
                match event {

                    Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                        if modifiers.command() {
                            match key {
                                keyboard::Key::Character(c) if c.as_ref() == "s" || c.as_ref() == "S" => Some(Message::Save),
                                keyboard::Key::Character(c) if c.as_ref() == "t" || c.as_ref() == "T" => Some(Message::DynamicIslandIncrease),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    }



                    Event::Mouse(mouse::Event::CursorMoved { position }) => {
                        Some(Message::CursorMoved(position))
                    }
                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                        Some(Message::StopResizing)
                    }

                    _ => None
                }
            }),

            self.terminal.subscription().map(Message::TerminalEvent),
        ])
    }

    fn init() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    fn view_file_tree(node: &FileNode) -> Element<'_, Message> {
        let icon_str = if node.is_dir {
            if node.is_expanded {
                "\u{f07c}"
            } else {
                "\u{f07b}"
            }
        } else {
            "\u{f15b}"
        };

        let icon = text(icon_str)
            .font(NERD_FONT)
            .size(16);

        let label = text(node.name.clone()).font(INTER)
            .size(14);

        let content = button(
            row![
        icon,
        label
    ]
                .spacing(8)
        )
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

fn create_file_tabs(file_tabs: Vec<PathBuf>, current_path: &str) -> Element<'static, Message> {
    let tabs_row = file_tabs.into_iter().fold(row![].spacing(10), |tabs, path| {

        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        let is_active = path.display().to_string() == current_path;

        let bg_color = if is_active {
            Color::from_rgb8(60, 60, 60)
        } else {
            Color::from_rgb8(35, 35, 35)
        };

        let boarder_color = if is_active {
            Color::from_rgb8(70, 70, 70)
        } else {
            Color::from_rgb8(35, 35, 35)
        };

        tabs.push(
            container(
                row![
                    button(text(file_name).font(INTER))
                        .on_press(Message::OpenTab(path.clone()))
                        .style(move |_theme, _status| button::Style {
                            background: Some(Background::Color(bg_color)),
                            text_color: Color::WHITE,
                            border: Border { radius: 8.0.into(),  ..Default::default() },
                            ..Default::default()
                        }),
                    button(text("").font(NERD_FONT))

                        .on_press(Message::CloseTab(path.clone()))
                        .style(|_theme, _status| button::Style {
                            background: Some(Background::Color(Color::TRANSPARENT)),
                            text_color: Color::from_rgb8(120, 120, 120),
                            border: Border { radius: 8.0.into(), ..Default::default() },
                            ..Default::default()
                        })
                ]
            )
                .style(move |_theme| container::Style {
                    background: Some(Background::Color(bg_color)),
                    text_color: Some(Color::WHITE),
                    border: Border { radius: 8.0.into(), color: boarder_color, width: 3.5,  ..Default::default() },
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



fn main() -> iced::Result {
    let icon = load_icon("final_icon.png");
    iced::application(Project::init, Project::update, Project::view)
        .title(|_state: &Project| String::from("Gravity Editor"))
        .theme(|_state: &Project| Theme::Dark)
        .subscription(Project::subscription)
        .font(include_bytes!("../fonts/JetBrainsMonoNerdFont-Regular.ttf"))
        .default_font(NERD_FONT)
        .window(window::Settings {
            icon,
            min_size: Some((800.0, 600.0).into()),
            ..Default::default()
        })
        .run()
}