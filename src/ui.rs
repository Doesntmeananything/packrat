use std::collections::HashMap;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, Gauge, Paragraph, Row, Table},
    Frame,
};

use crate::{
    application::{DependencyTable, State},
    package::Project,
};

pub fn draw_ui<B: Backend>(
    f: &mut Frame<B>,
    project: &Project,
    fetched_packages: &HashMap<String, String>,
    state: &mut State,
) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let header = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
        .split(root[0]);
    f.render_widget(project_info(project), header[0]);
    f.render_widget(loading_progress(state, fetched_packages), header[1]);

    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(root[1]);
    if state.dependencies_len != 0 {
        f.render_stateful_widget(
            dependencies_table(project, fetched_packages, state, DependencyTable::Runtime),
            main[0],
            &mut state.dependencies_table_state,
        );
    }
    if state.dev_dependencies_len != 0 {
        let area = if state.dependencies_len != 0 {
            main[1]
        } else {
            main[0]
        };
        f.render_stateful_widget(
            dependencies_table(project, fetched_packages, state, DependencyTable::Dev),
            area,
            &mut state.dev_dependencies_table_state,
        );
    }

    f.render_widget(help(), root[2]);
}

fn project_info(project: &Project) -> Paragraph {
    let info = vec![Spans::from(vec![
        Span::styled(
            project.name(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(": "),
        Span::styled(project.version(), Style::default().fg(Color::Green)),
    ])];

    Paragraph::new(info).block(
        Block::default()
            .title("Project")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    )
}

fn loading_progress<'a>(
    state: &'a State,
    fetched_packages: &'a HashMap<String, String>,
) -> Gauge<'a> {
    let fetched_count = fetched_packages.len();
    let total_count = state.dependencies_len + state.dev_dependencies_len;
    let label = format!("{}/{}", fetched_count, total_count);

    Gauge::default()
        .block(
            Block::default()
                .title("Status")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .gauge_style(Style::default().bg(Color::Black).fg(Color::White))
        .ratio(fetched_count as f64 * 1.0 / total_count as f64)
        .label(label)
}

fn dependencies_table<'a>(
    project: &'a Project,
    fetched_packages: &'a HashMap<String, String>,
    state: &State,
    dependency_type: DependencyTable,
) -> Table<'a> {
    let (label, dependencies, len, update_index, table_state) = match dependency_type {
        DependencyTable::Runtime => (
            "Dependencies",
            project.dependencies(),
            &state.dependencies_len,
            &state.update_index,
            &state.dependencies_table_state,
        ),
        DependencyTable::Dev => (
            "Development Dependencies",
            project.dev_dependencies(),
            &state.dev_dependencies_len,
            &state.dev_update_index,
            &state.dev_dependencies_table_state,
        ),
    };

    let deps = dependencies.iter().flat_map(|d| d.iter());

    let rows = deps.enumerate().map(|(i, (name, version))| {
        let is_toggled = update_index.contains(&i);
        let mut row_style = Style::default();
        let mut display_name = name.to_owned();

        if is_toggled {
            row_style = Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::LightYellow);
            display_name += "*";
        }

        let mut row = vec![
            Cell::from(display_name),
            Cell::from(version.as_str().unwrap()),
        ];

        let latest_version = fetched_packages.get(name);
        if let Some(latest_version) = latest_version {
            row.push(Cell::from(latest_version.to_owned()));
        }

        Row::new(row).style(row_style)
    });

    let mut border_style = Style::default();
    let mut highlight_style = Style::default();

    if state.active_table == dependency_type {
        highlight_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::DarkGray);
        border_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Green);
    }

    let title = format!(
        "{} [{}/{}]",
        label,
        table_state.selected().unwrap_or(0) + 1,
        len
    );

    Table::new(rows)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(Span::styled(
                    title,
                    Style::default().add_modifier(Modifier::BOLD),
                ))
                .style(Style::default())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style),
        )
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .column_spacing(0)
        .highlight_style(highlight_style)
}

const HELP_TEXT: &str =
    "↑ ↓: navigate, Space/Enter: select, Tab: switch group, u: update package.json, Esc/q: close";

fn help<'a>() -> Paragraph<'a> {
    Paragraph::new(HELP_TEXT).style(Style::default().fg(Color::Blue))
}
