use std::path::Path;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap,
};
use ratatui::Frame;

use crate::operations::ToolAction;
use crate::status::{self, ToolStatus};
use crate::tool::ToolGroup;

use super::{
    config_menu_items, is_actionable, App, ConfigKind, FormState,
    HelpPopup, Screen, ToolFocus, ToolManagerState,
};

pub(super) fn render(frame: &mut Frame, app: &mut App) {
    match &mut app.screen {
        Screen::Dashboard { scroll } => {
            render_dashboard(frame, app.install_root.as_deref(), *scroll, app.nav_focus)
        }
        Screen::Tools(state) => render_tools(frame, state, app.nav_focus),
        Screen::ConfigMenu { state } => render_config_menu(frame, state, app.nav_focus),
        Screen::Form(form) => render_form(frame, form),
        Screen::SkillSelection {
            state,
            skills,
            selected,
        } => render_skill_selection(frame, state, skills, selected, app.nav_focus),
        Screen::Output {
            title,
            lines,
            scroll,
        } => render_output(
            frame,
            title,
            lines,
            *scroll,
            app.nav_focus,
            app.background_action.is_some(),
        ),
    }
    if let Some(help) = &app.help {
        render_help_popup(frame, help);
    }
    if app.quit_confirm {
        render_quit_popup(frame);
    }
}

fn render_page_shell(
    frame: &mut Frame,
    page_idx: usize,
    title: &str,
    subtitle: &str,
    footer: &str,
    nav_focus: bool,
) -> Rect {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(area);

    let tabs = Tabs::new([
        "1\u{00b7}Dashboard",
        "2\u{00b7}Tools",
        "3\u{00b7}Configure",
        "4\u{00b7}Skills",
        "Output",
    ])
    .select(page_idx)
    .block(focus_block("AI Setup", Color::Cyan, nav_focus))
    .style(Style::default().fg(Color::DarkGray))
    .highlight_style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(tabs, chunks[0]);

    let header = Paragraph::new(vec![
        Line::from(Span::styled(
            title,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(subtitle, Style::default().fg(Color::Gray))),
    ])
    .block(panel_block("Page", Color::Yellow));
    frame.render_widget(header, chunks[1]);

    let footer_bar = Paragraph::new(footer)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::Gray))
        .block(panel_block("Keys", Color::Cyan));
    frame.render_widget(footer_bar, chunks[3]);

    chunks[2]
}

fn render_dashboard(frame: &mut Frame, install_root: Option<&Path>, scroll: u16, nav_focus: bool) {
    let statuses = status::collect_statuses(install_root);
    let installed = statuses.iter().filter(|s| s.path.is_some()).count();
    let missing = statuses.len().saturating_sub(installed);
    let core_missing = statuses
        .iter()
        .filter(|s| s.path.is_none() && matches!(s.tool.group, ToolGroup::Core))
        .count();
    let helper_missing = statuses
        .iter()
        .filter(|s| s.path.is_none() && matches!(s.tool.group, ToolGroup::Helper))
        .count();

    let body = render_page_shell(
        frame,
        0,
        "Dashboard",
        "Overview of installed AI tooling and helper CLIs.",
        "?: help  d/t/e/s: pages  Left/Right: tabs  Enter/Down: content  Esc: tabs/prev  q: quit",
        nav_focus,
    );

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8)])
        .split(body);

    let summary = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("Installed {}", installed),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("Missing {}", missing),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("Core missing {}", core_missing),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("Helper missing {}", helper_missing),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(panel_block("Summary", Color::Cyan));
    frame.render_widget(summary, rows[0]);

    let panel_width = rows[1].width.saturating_sub(4) as usize;
    let status_lines = dashboard_status_lines(&statuses, install_root, panel_width);
    let status_panel = Paragraph::new(status_lines)
        .wrap(Wrap { trim: true })
        .scroll((scroll, 0))
        .style(Style::default().fg(Color::Gray))
        .block(focus_block("Environment status", Color::Yellow, !nav_focus));
    frame.render_widget(status_panel, rows[1]);
}

fn dashboard_status_lines(
    statuses: &[ToolStatus],
    install_root: Option<&Path>,
    panel_width: usize,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Detailed tool status (version + resolved PATH location)",
        Style::default().fg(Color::Gray),
    )));

    if let Some(root) = install_root {
        let root_text =
            truncate_middle(&root.display().to_string(), panel_width.saturating_sub(10));
        lines.push(Line::from(vec![
            Span::styled("\u{1F4C1} ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("install root: {root_text}"),
                Style::default().fg(Color::Cyan),
            ),
        ]));
    }

    lines.push(Line::from(Span::styled(
        "STATE TOOL         VERSION                  PATH",
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )));

    let tool_w = 12usize;
    let version_w = 24usize;
    let fixed = 2 + 1 + tool_w + 1 + version_w + 1;
    let path_w = panel_width.saturating_sub(fixed).max(20);

    for status in statuses {
        let installed = status.path.is_some();
        let icon = if installed { "\u{2705}" } else { "\u{274C}" };
        let state_color = if installed { Color::Green } else { Color::Red };
        let version = status.version.as_deref().unwrap_or("-");
        let version_short = truncate_end(version, version_w);
        let path_text = status
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "-".to_string());
        let path_short = truncate_middle(&path_text, path_w);

        lines.push(Line::from(vec![
            Span::styled(icon.to_string(), Style::default().fg(state_color)),
            Span::raw(" "),
            Span::styled(
                format!("{:<tool_w$}", status.tool.display_name),
                Style::default()
                    .fg(status_color(status))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                format!("{:<version_w$}", version_short),
                Style::default().fg(Color::Gray),
            ),
            Span::raw(" "),
            Span::styled(path_short, Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines
}

fn truncate_end(text: &str, max_chars: usize) -> String {
    let total = text.chars().count();
    if total <= max_chars || max_chars <= 1 {
        return text.to_string();
    }
    let left: String = text.chars().take(max_chars - 1).collect();
    format!("{left}\u{2026}")
}

fn truncate_middle(text: &str, max_chars: usize) -> String {
    let total = text.chars().count();
    if total <= max_chars || max_chars <= 3 {
        return text.to_string();
    }

    let keep_left = (max_chars - 1) / 2;
    let keep_right = max_chars - 1 - keep_left;
    let left: String = text.chars().take(keep_left).collect();
    let right: String = text
        .chars()
        .rev()
        .take(keep_right)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{left}\u{2026}{right}")
}

fn render_tools(frame: &mut Frame, state: &mut ToolManagerState, nav_focus: bool) {
    let selected_count = state.selected.iter().filter(|flag| **flag).count();
    let (installable_count, updatable_count, removable_count) = state.action_counts();

    let subtitle = format!(
        "selected {} | installable {} | updatable {} | removable {} | focus {:?}",
        selected_count, installable_count, updatable_count, removable_count, state.focus,
    );
    let body = render_page_shell(
        frame,
        1,
        "Manage Tools",
        &subtitle,
        "?: help  i/u/x: run selected actions  Enter: detail  Esc: back/list  Up/Down: move/scroll  Space: toggle  Left/Right: list/detail  a/m/p/c/r",
        nav_focus,
    );

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8)])
        .split(body);

    let action_summary = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("i Install {}", installable_count),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  "),
        Span::styled(
            format!("u Update {}", updatable_count),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  "),
        Span::styled(
            format!("x Uninstall {}", removable_count),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(panel_block("Operations", Color::Cyan));
    frame.render_widget(action_summary, rows[0]);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(11)])
        .split(rows[1]);

    let list_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(sections[0]);

    let header = Paragraph::new(Line::from(vec![
        Span::styled("Sel ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:<14}", "Tool"),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(" \u{2705}  i u x", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(header, list_rows[0]);

    let items: Vec<ListItem> = state
        .statuses
        .iter()
        .enumerate()
        .map(|(index, status)| {
            let mark = if state.selected[index] { "[x]" } else { "[ ]" };
            let installed_mark = if status.path.is_some() {
                "\u{2705}"
            } else {
                " "
            };
            let install_tag = if is_actionable(status, ToolAction::Install) {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let update_tag = if is_actionable(status, ToolAction::Update) {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let uninstall_tag = if is_actionable(status, ToolAction::Uninstall) {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let line = Line::from(vec![
                Span::raw(format!("{:<4}", mark)),
                Span::styled(
                    format!("{:<14}", status.tool.display_name),
                    Style::default()
                        .fg(status_color(status))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(installed_mark, Style::default().fg(Color::Green)),
                Span::raw("   "),
                Span::styled("i", install_tag),
                Span::raw(" "),
                Span::styled("u", update_tag),
                Span::raw(" "),
                Span::styled("x", uninstall_tag),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(focus_block(
            "Tool list",
            Color::Green,
            !nav_focus && state.focus == ToolFocus::ToolList,
        ))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, list_rows[1], &mut state.list_state);

        let detail_lines = if let Some(index) = state.list_state.selected() {
        let status = &state.statuses[index];
        let selected = state.selected[index];
        let version = status
            .version
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let latest_version = status
            .latest_version
            .clone()
            .unwrap_or_else(|| "-".to_string());
        let path = status
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "not found in PATH".to_string());
        vec![
            Line::from(Span::styled(
                status.tool.display_name,
                Style::default()
                    .fg(status_color(status))
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(format!("key: {}", status.tool.key)),
            Line::from(format!("selected: {}", if selected { "yes" } else { "no" })),
            Line::from(format!(
                "installed: {}",
                if status.path.is_some() {
                    "\u{2705} yes"
                } else {
                    "\u{2B1C} no"
                }
            )),
            Line::from(format!("version: {}", version)),
            Line::from(format!("latest: {}", latest_version)),
            Line::from(format!(
                "update available: {}",
                if status.update_available { "yes" } else { "no" }
            )),
            Line::from(format!("path: {}", path)),
            Line::from(format!("command: {}", status.tool.command)),
            Line::from(format!("winget: {}", status.tool.winget_id)),
            Line::from(""),
            Line::from(Span::styled(
                "Operation availability",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(format!(
                "i install: {}",
                if is_actionable(status, ToolAction::Install) {
                    "available"
                } else {
                    "not available"
                }
            )),
            Line::from(format!(
                "u update: {}",
                if is_actionable(status, ToolAction::Update) {
                    "available"
                } else {
                    "not available"
                }
            )),
            Line::from(format!(
                "x uninstall: {}",
                if is_actionable(status, ToolAction::Uninstall) {
                    "available"
                } else {
                    "not available"
                }
            )),
        ]
    } else {
        vec![Line::from("No tool selected.")]
    };

    let detail_panel = Paragraph::new(detail_lines)
        .wrap(Wrap { trim: false })
        .scroll((state.detail_scroll, 0))
        .style(Style::default().fg(Color::Gray))
        .block(focus_block(
            "Selection detail",
            Color::Yellow,
            !nav_focus && state.focus == ToolFocus::Detail,
        ));
    frame.render_widget(detail_panel, sections[1]);
}

fn render_config_menu(frame: &mut Frame, state: &mut ListState, nav_focus: bool) {
    let body = render_page_shell(
        frame,
        2,
        "Configure",
        "Configure global editor/proxy plus Git, Claude Code, and Codex settings.",
        "?: help  Enter: open  Esc: tabs/prev  j/k or arrows: move  q: quit",
        nav_focus,
    );

    let items: Vec<ListItem> = config_menu_items()
        .iter()
        .map(|(label, _)| {
            ListItem::new(Line::from(Span::styled(
                *label,
                Style::default().fg(Color::Gray),
            )))
        })
        .collect();
    let list = List::new(items)
        .block(focus_block("Targets", Color::Yellow, !nav_focus))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, body, state);
}

fn render_form(frame: &mut Frame, form: &FormState) {
    let subtitle = match form.kind {
        ConfigKind::Global => {
            "Global settings used by ai-setup (editor command and proxy environment)."
        }
        ConfigKind::Git => "Git config is written via git config --global.",
        ConfigKind::Claude => "Writes ~/.claude/settings.json env values.",
        ConfigKind::Codex => "Writes ~/.codex/config.toml and ~/.codex/auth.json.",
    };
    let body = render_page_shell(
        frame,
        2,
        form.title,
        subtitle,
        "?: help  Enter: save/open/back  o: open in editor  Esc: cancel  Type: edit  Backspace: del  Tab/j/k/Arrows: move",
        false,
    );

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(4)])
        .split(body);

    let items: Vec<ListItem> = form
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let display_value = if field.secret {
                "*".repeat(field.value.chars().count())
            } else {
                field.value.clone()
            };
            let prefix = if form.selected == index { "> " } else { "  " };
            ListItem::new(Line::from(format!(
                "{}{}: {}",
                prefix, field.label, display_value
            )))
        })
        .chain([
            ListItem::new(Line::from(format!(
                "{}Save",
                if form.selected == form.fields.len() {
                    "> "
                } else {
                    "  "
                }
            ))),
            ListItem::new(Line::from(format!(
                "{}Open in editor",
                if form.selected == form.fields.len() + 1 {
                    "> "
                } else {
                    "  "
                }
            ))),
            ListItem::new(Line::from(format!(
                "{}Back",
                if form.selected == form.fields.len() + 2 {
                    "> "
                } else {
                    "  "
                }
            ))),
        ])
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Fields"));
    frame.render_widget(list, rows[0]);

    let current = if form.selected < form.fields.len() {
        format!("Editing: {}", form.fields[form.selected].label)
    } else if form.selected == form.fields.len() {
        "Editing: Save".to_string()
    } else if form.selected == form.fields.len() + 1 {
        "Editing: Open in editor".to_string()
    } else {
        "Editing: Back".to_string()
    };
    let hint = Paragraph::new(vec![
        Line::from(Span::styled(
            current,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Type to edit field. Tab/Shift+Tab or j/k to move. Enter confirms. Press o to open file.",
            Style::default().fg(Color::Gray),
        )),
    ])
    .wrap(Wrap { trim: true })
    .block(panel_block("Hint", Color::Cyan));
    frame.render_widget(hint, rows[1]);
}

fn render_skill_selection(
    frame: &mut Frame,
    state: &mut ListState,
    skills: &[crate::skills::Skill],
    selected: &[bool],
    nav_focus: bool,
) {
    let subtitle = format!(
        "{} skill source(s) detected. Install selected copies into ~/.claude/skills.",
        skills.len()
    );
    let body = render_page_shell(
        frame,
        3,
        "Skills",
        &subtitle,
        "?: help  Enter: install/back  Esc: tabs/prev  Space: toggle  a: all  c: clear  q: quit",
        nav_focus,
    );

    let mut items = Vec::new();
    for (index, skill) in skills.iter().enumerate() {
        let marker = if selected[index] { "[x]" } else { "[ ]" };
        items.push(ListItem::new(Line::from(vec![
            Span::raw(format!("{} ", marker)),
            Span::styled(&skill.name, Style::default().fg(Color::Gray)),
        ])));
    }
    items.push(ListItem::new(Line::from(Span::styled(
        "Install selected",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    ))));
    items.push(ListItem::new(Line::from(Span::styled(
        "Back",
        Style::default().fg(Color::Yellow),
    ))));

    let list = List::new(items)
        .block(focus_block("Available skills", Color::Cyan, !nav_focus))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, body, state);
}

fn render_output(
    frame: &mut Frame,
    title: &str,
    lines: &[String],
    scroll: u16,
    nav_focus: bool,
    running: bool,
) {
    let footer = if running {
        "?: help  Running... Enter/Esc disabled  q: quit  j/k or Up/Down: scroll"
    } else {
        "?: help  Enter/Esc: back  q: quit  j/k or Up/Down: scroll"
    };
    let body = render_page_shell(
        frame,
        4,
        title,
        "Execution results, saved confirmations, and help pages.",
        footer,
        nav_focus,
    );

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(body);

    let paragraph = Paragraph::new(lines.iter().cloned().map(Line::from).collect::<Vec<_>>())
        .style(Style::default().fg(Color::Gray))
        .block(focus_block("Details", Color::Yellow, !nav_focus))
        .wrap(Wrap { trim: true })
        .scroll((scroll, 0));
    frame.render_widget(paragraph, rows[0]);

    let hint = if running {
        "Hint: j/k scroll logs. Enter/Esc disabled while running. Press q to quit."
    } else {
        "Hint: j/k scroll logs. Enter/Esc returns to previous page. q quits."
    };
    let hint_bar = Paragraph::new(hint)
        .style(Style::default().fg(Color::Gray))
        .block(panel_block("Hint", Color::Cyan));
    frame.render_widget(hint_bar, rows[1]);
}

fn status_color(status: &ToolStatus) -> Color {
    if status.path.is_some() {
        match status.tool.group {
            ToolGroup::Core => Color::Green,
            ToolGroup::Helper => Color::Cyan,
        }
    } else {
        match status.tool.group {
            ToolGroup::Core => Color::Yellow,
            ToolGroup::Helper => Color::DarkGray,
        }
    }
}

fn render_help_popup(frame: &mut Frame, help: &HelpPopup) {
    let popup = centered_rect(70, 70, frame.area());
    let paragraph = Paragraph::new(
        help.lines
            .iter()
            .cloned()
            .map(Line::from)
            .collect::<Vec<_>>(),
    )
    .wrap(Wrap { trim: false })
    .scroll((help.scroll, 0))
    .style(Style::default().fg(Color::Gray).bg(Color::Black))
    .block(panel_block(&help.title, Color::Yellow));
    let backdrop = Block::default().style(
        Style::default()
            .bg(Color::Black)
            .add_modifier(Modifier::DIM),
    );
    frame.render_widget(backdrop, frame.area());
    frame.render_widget(Clear, popup);
    frame.render_widget(paragraph, popup);
}

fn render_quit_popup(frame: &mut Frame) {
    let popup = centered_rect(48, 24, frame.area());
    let content = vec![
        Line::from(Span::styled(
            "Exit ai-setup?",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Enter / y / q: confirm",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "Esc / n: cancel",
            Style::default().fg(Color::Gray),
        )),
    ];
    let paragraph = Paragraph::new(content)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::Gray).bg(Color::Black))
        .block(panel_block("Confirm Exit", Color::Yellow));
    let backdrop = Block::default().style(
        Style::default()
            .bg(Color::Black)
            .add_modifier(Modifier::DIM),
    );
    frame.render_widget(backdrop, frame.area());
    frame.render_widget(Clear, popup);
    frame.render_widget(paragraph, popup);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}

fn panel_block<'a>(title: &'a str, title_color: Color) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Line::from(Span::styled(
            title,
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        )))
}

fn focus_block<'a>(title: &'a str, title_color: Color, focused: bool) -> Block<'a> {
    let border = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let accent = if focused { Color::Cyan } else { title_color };
    let title_text = if focused {
        format!("> {title}")
    } else {
        title.to_string()
    };
    Block::default()
        .borders(Borders::ALL)
        .border_type(if focused {
            BorderType::Thick
        } else {
            BorderType::Plain
        })
        .border_style(Style::default().fg(border).add_modifier(if focused {
            Modifier::BOLD
        } else {
            Modifier::empty()
        }))
        .title(Line::from(Span::styled(
            title_text,
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )))
}
