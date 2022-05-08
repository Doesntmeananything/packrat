use std::{
    collections::{HashMap, HashSet},
    env,
    io::stdout,
    time::{Duration, Instant},
};

use anyhow::Error;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute, terminal,
};
use futures::{stream, StreamExt};
use reqwest::Client;
use tokio::sync::mpsc;
use tui::{backend::CrosstermBackend, widgets::TableState, Terminal};

use crate::{args::Args, package::Project, registry, ui::draw_ui};

#[derive(Debug)]
enum ApplicationEvent<T> {
    Input(T),
    Tick,
}

#[derive(PartialEq)]
pub enum DependencyTable {
    Runtime,
    Dev,
}

pub struct State {
    pub dependencies_len: usize,
    pub dev_dependencies_len: usize,
    pub active_table: DependencyTable,
    pub dependencies_table_state: TableState,
    pub dev_dependencies_table_state: TableState,
    pub update_index: HashSet<usize>,
    pub dev_update_index: HashSet<usize>,
}

pub struct Application {
    project: Project,
    pub fetched_packages: HashMap<String, String>,
    state: State,
}

impl Application {
    pub fn new(args: Args) -> Result<Self, Error> {
        let mut path = env::current_dir()?;
        if let Some(custom_directory) = args.directory {
            path = custom_directory;
        }
        path.push("package.json");

        let project = Project::new(&path)?;

        let dependencies_len = match project.dependencies() {
            Some(dependencies) => dependencies.len(),
            None => 0,
        };
        let dev_dependencies_len = match project.dev_dependencies() {
            Some(dependencies) => dependencies.len(),
            None => 0,
        };

        let mut app = Self {
            project,

            fetched_packages: HashMap::new(),

            state: State {
                dependencies_len,
                dev_dependencies_len,

                active_table: DependencyTable::Runtime,
                dependencies_table_state: TableState::default(),
                dev_dependencies_table_state: TableState::default(),

                update_index: HashSet::new(),
                dev_update_index: HashSet::new(),
            },
        };

        if app.state.dev_dependencies_len != 0 {
            app.state.dev_dependencies_table_state.select(Some(0));
            app.state.active_table = DependencyTable::Dev;
        }
        if app.state.dependencies_len != 0 {
            app.state.dependencies_table_state.select(Some(0));
            app.state.active_table = DependencyTable::Runtime;
        }

        Ok(app)
    }

    fn switch_table(&mut self) {
        if self.state.dependencies_len == 0 || self.state.dev_dependencies_len == 0 {
            return;
        }

        match self.state.active_table {
            DependencyTable::Runtime => self.state.active_table = DependencyTable::Dev,
            DependencyTable::Dev => self.state.active_table = DependencyTable::Runtime,
        }
    }

    fn next(&mut self) {
        let (state, len) = match self.state.active_table {
            DependencyTable::Runtime => (
                &mut self.state.dependencies_table_state,
                self.state.dependencies_len,
            ),
            DependencyTable::Dev => (
                &mut self.state.dev_dependencies_table_state,
                self.state.dev_dependencies_len,
            ),
        };

        let i = match state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        state.select(Some(i))
    }

    fn previous(&mut self) {
        let (state, len) = match self.state.active_table {
            DependencyTable::Runtime => (
                &mut self.state.dependencies_table_state,
                self.state.dependencies_len,
            ),
            DependencyTable::Dev => (
                &mut self.state.dev_dependencies_table_state,
                self.state.dev_dependencies_len,
            ),
        };

        let i = match state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        state.select(Some(i))
    }

    fn toggle_update(&mut self) {
        match self.state.active_table {
            DependencyTable::Runtime => {
                let selected_index = self.state.dependencies_table_state.selected().unwrap_or(0);
                if self.state.update_index.contains(&selected_index) {
                    self.state.update_index.remove(&selected_index);
                } else {
                    self.state.update_index.insert(selected_index);
                }
            }
            DependencyTable::Dev => {
                let selected_index = self
                    .state
                    .dev_dependencies_table_state
                    .selected()
                    .unwrap_or(0);
                if self.state.dev_update_index.contains(&selected_index) {
                    self.state.dev_update_index.remove(&selected_index);
                } else {
                    self.state.dev_update_index.insert(selected_index);
                }
            }
        }
    }

    fn update_package_json(&mut self) {
        if self.state.update_index.is_empty() && self.state.dev_update_index.is_empty() {
            return;
        }

        let project = self.project.clone();
        let dependencies = project.dependencies().into_iter().flatten();
        let dev_dependencies = project.dev_dependencies().into_iter().flatten();

        for (i, (name, version)) in dependencies.enumerate() {
            if self.state.update_index.contains(&i) {
                let latest_version = self
                    .fetched_packages
                    .get(name)
                    .expect("Unable to get the latest version to update package");

                let range_prefix = match version.as_str().unwrap().chars().next() {
                    Some('~') | Some('^') => version.as_str().unwrap().chars().next(),
                    _ => None,
                };

                self.project
                    .update_dependency_version(name, latest_version, range_prefix);
            }
        }

        for (i, (name, version)) in dev_dependencies.enumerate() {
            if self.state.dev_update_index.contains(&i) {
                let latest_version = self
                    .fetched_packages
                    .get(name)
                    .expect("Unable to get the latest version to update package");

                let range_prefix = match version.as_str().unwrap().chars().next() {
                    Some('~') | Some('^') => version.as_str().unwrap().chars().next(),
                    _ => None,
                };

                self.project
                    .update_dependency_version(name, latest_version, range_prefix);
            }
        }

        self.project
            .write_to_file()
            .expect("Unable to write updates to package.json file");
    }

    async fn event_loop(&mut self) {
        let tick_rate = Duration::from_millis(20);

        // Process inputs in a separate task
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move {
            let mut last_tick = Instant::now();

            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("Unable to poll events") {
                    if let Event::Key(key) = event::read().expect("Unable to read events") {
                        tx.send(ApplicationEvent::Input(key))
                            .await
                            .expect("Unable to send application events");
                    }
                }

                if last_tick.elapsed() >= tick_rate && tx.send(ApplicationEvent::Tick).await.is_ok()
                {
                    last_tick = Instant::now();
                }
            }
        });

        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend).expect("Unable to create a terminal session");

        let client = Client::new();

        let project = self.project.clone();
        let dependencies = project.dependencies().into_iter().map(|d| d.keys());
        let dev_dependencies = project.dev_dependencies().into_iter().map(|d| d.keys());
        let all_dependencies = dependencies.chain(dev_dependencies).flatten();

        let mut package_updates = stream::iter(all_dependencies)
            .map(|package_name| {
                let client = &client;
                registry::fetch_metadata(client, package_name)
            })
            .buffer_unordered(10);

        loop {
            terminal
                .draw(|f| draw_ui(f, &self.project, &self.fetched_packages, &mut self.state))
                .expect("Unable to draw a terminal frame");

            tokio::select! {
                biased;

                Some(event) = rx.recv() => {
                    match event {
                        ApplicationEvent::Input(key) => match key.code {
                            KeyCode::Down => self.next(),
                            KeyCode::Up => self.previous(),
                            KeyCode::Tab | KeyCode::BackTab => self.switch_table(),
                            KeyCode::Enter | KeyCode::Char(' ') => self.toggle_update(),
                            KeyCode::Char('u') => self.update_package_json(),
                            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => break,
                            _ => {}
                        },
                        ApplicationEvent::Tick => {}
                    }
                }
                Some(package) = package_updates.next() => {
                    match package {
                        Ok(package) => {
                            self.fetched_packages.insert(package.name, package.dist_tags.latest);
                        },
                        Err(_e) => {
                            todo!();
                        }
                    }

                }
                else => { break }
            };
        }
    }

    fn claim_terminal(&mut self) -> Result<(), Error> {
        terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            terminal::Clear(terminal::ClearType::All),
            EnableMouseCapture
        )?;

        Ok(())
    }

    fn restore_terminal(&mut self) -> Result<(), Error> {
        terminal::disable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, terminal::LeaveAlternateScreen, DisableMouseCapture)?;

        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        self.claim_terminal()?;

        // Restore the terminal on panic
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = execute!(std::io::stdout(), DisableMouseCapture);
            let _ = execute!(std::io::stdout(), terminal::LeaveAlternateScreen);
            let _ = terminal::disable_raw_mode();
            hook(info);
        }));

        self.event_loop().await;

        self.restore_terminal()?;

        Ok(())
    }
}
