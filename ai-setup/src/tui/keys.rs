use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;

use anyhow::Result;
use crossterm::event::KeyCode;

use crate::config;
use crate::operations::{self, ToolAction};
use crate::skills;
use crate::status;

use super::{
    action_past, action_title, config_menu_items, new_config_menu,
    new_skill_selection, new_tool_manager, next_top_screen, previous_top_screen, screen_name,
    ActionOutcome, App, BackgroundAction, FormState, HelpPopup, Screen,
    ToolFocus, ToolManagerState, ToolsKeyOutcome,
};
use super::help::help_lines_for_screen;

pub(super) fn handle_key(app: &mut App, code: KeyCode) -> Result<bool> {
    if let Some(help) = app.help.as_mut() {
        match code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Enter => app.help = None,
            KeyCode::Down | KeyCode::Char('j') => help.scroll = help.scroll.saturating_add(1),
            KeyCode::Up | KeyCode::Char('k') => help.scroll = help.scroll.saturating_sub(1),
            _ => {}
        }
        return Ok(false);
    }

    if app.quit_confirm {
        match code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Char('q') => {
                return Ok(true);
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                app.quit_confirm = false;
            }
            _ => {}
        }
        return Ok(false);
    }

    if matches!(code, KeyCode::Char('?')) {
        app.help = Some(HelpPopup {
            title: format!("Help - {}", screen_name(&app.screen)),
            lines: help_lines_for_screen(&app.screen),
            scroll: 0,
        });
        return Ok(false);
    }

    if matches!(code, KeyCode::Tab) && !matches!(app.screen, Screen::Form(_)) {
        app.nav_focus = true;
        return Ok(false);
    }

    if !matches!(app.screen, Screen::Form(_) | Screen::Output { .. }) && app.nav_focus {
        match code {
            KeyCode::Left | KeyCode::Char('h') => {
                app.screen =
                    previous_top_screen(&app.screen, app.install_root.as_deref(), &app.repo_root);
                app.nav_focus = true;
                return Ok(false);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                app.screen =
                    next_top_screen(&app.screen, app.install_root.as_deref(), &app.repo_root);
                app.nav_focus = true;
                return Ok(false);
            }
            KeyCode::Char('1') => {
                app.screen = Screen::Dashboard { scroll: 0 };
                app.nav_focus = true;
                return Ok(false);
            }
            KeyCode::Char('2') => {
                app.screen = Screen::Tools(new_tool_manager(app.install_root.as_deref()));
                app.nav_focus = true;
                return Ok(false);
            }
            KeyCode::Char('3') => {
                app.screen = new_config_menu();
                app.nav_focus = true;
                return Ok(false);
            }
            KeyCode::Char('4') => {
                app.screen = new_skill_selection(&app.repo_root);
                app.nav_focus = true;
                return Ok(false);
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Enter => {
                app.nav_focus = false;
                return Ok(false);
            }
            KeyCode::Esc => {
                app.screen =
                    previous_top_screen(&app.screen, app.install_root.as_deref(), &app.repo_root);
                app.nav_focus = true;
                return Ok(false);
            }
            KeyCode::Char('q') => {
                app.quit_confirm = true;
                return Ok(false);
            }
            _ => return Ok(false),
        }
    }

    if matches!(code, KeyCode::Esc)
        && !app.nav_focus
        && !matches!(app.screen, Screen::Tools(_))
        && !matches!(app.screen, Screen::Form(_) | Screen::Output { .. })
    {
        app.nav_focus = true;
        return Ok(false);
    }

    if !matches!(
        app.screen,
        Screen::Form(_) | Screen::Output { .. }
    ) {
        match code {
            KeyCode::Char('1') => {
                app.screen = Screen::Dashboard { scroll: 0 };
                app.nav_focus = true;
                return Ok(false);
            }
            KeyCode::Char('2') => {
                app.screen = Screen::Tools(new_tool_manager(app.install_root.as_deref()));
                app.nav_focus = true;
                return Ok(false);
            }
            KeyCode::Char('3') => {
                app.screen = new_config_menu();
                app.nav_focus = true;
                return Ok(false);
            }
            KeyCode::Char('4') => {
                app.screen = new_skill_selection(&app.repo_root);
                app.nav_focus = true;
                return Ok(false);
            }
            _ => {}
        }
    }
    let mut next_screen = None;
    let mut pop_previous_page = false;
    let mut quit = false;

    match &mut app.screen {
        Screen::Dashboard { scroll } => match code {
            KeyCode::Char('q') => quit = true,
            KeyCode::Down | KeyCode::Char('j') => *scroll = scroll.saturating_add(1),
            KeyCode::Up | KeyCode::Char('k') => {
                if *scroll == 0 {
                    app.nav_focus = true;
                } else {
                    *scroll = scroll.saturating_sub(1);
                }
            }
            _ => {}
        },
        Screen::Tools(state) => {
            let outcome = handle_tools_key(
                state,
                code,
                app.install_root.as_deref(),
                &mut quit,
                app.background_action.is_some(),
            )?;
            if let Some(screen) = outcome.next_screen {
                next_screen = Some(screen);
            }
            if let Some(background_action) = outcome.background_action {
                app.background_action = Some(background_action);
            }
        }
        Screen::ConfigMenu { state } => {
            if matches!(code, KeyCode::Up | KeyCode::Char('k'))
                && state.selected().unwrap_or(0) == 0
            {
                app.nav_focus = true;
                return Ok(false);
            }
            if let Some(screen) = handle_config_menu_key(state, code, &mut quit) {
                next_screen = Some(screen);
            }
        }
        Screen::Form(form) => {
            if let Some(screen) = handle_form_key(form, code)? {
                next_screen = Some(screen);
            }
        }
        Screen::SkillSelection {
            state,
            skills,
            selected,
        } => {
            if matches!(code, KeyCode::Up | KeyCode::Char('k'))
                && state.selected().unwrap_or(0) == 0
            {
                app.nav_focus = true;
                return Ok(false);
            }
            if let Some(screen) = handle_skill_key(state, skills, selected, code, &mut quit)? {
                next_screen = Some(screen);
            }
        }
        Screen::Output { scroll, .. } => match code {
            KeyCode::Esc | KeyCode::Enter => {
                if app.background_action.is_none() {
                    pop_previous_page = true;
                }
            }
            KeyCode::Char('q') => quit = true,
            KeyCode::Down | KeyCode::Char('j') => *scroll = scroll.saturating_add(1),
            KeyCode::Up | KeyCode::Char('k') => *scroll = scroll.saturating_sub(1),
            _ => {}
        },
    }

    if pop_previous_page {
        next_screen = app.page_stack.pop().map(|mut screen| {
            if let Screen::Tools(state) = &mut screen {
                let _ = status::refresh_process_env_from_registry();
                state.refresh(app.install_root.as_deref());
            }
            screen
        });
        if next_screen.is_none() {
            next_screen = Some(Screen::Dashboard { scroll: 0 });
        }
    }

    if let Some(screen) = next_screen {
        let push_current =
            matches!(screen, Screen::Output { .. }) && !matches!(app.screen, Screen::Output { .. });
        if push_current {
            let current = std::mem::replace(&mut app.screen, screen);
            app.page_stack.push(current);
        } else {
            app.screen = screen;
        }
        app.nav_focus = false;
    }

    if quit {
        app.quit_confirm = true;
        return Ok(false);
    }

    Ok(false)
}

fn handle_tools_key(
    state: &mut ToolManagerState,
    code: KeyCode,
    install_root: Option<&Path>,
    quit: &mut bool,
    action_running: bool,
) -> Result<ToolsKeyOutcome> {
    let len = state.statuses.len();
    let index = state.list_state.selected().unwrap_or(0);
    let mut outcome = ToolsKeyOutcome {
        next_screen: None,
        background_action: None,
    };

    match code {
        KeyCode::Esc => {
            if state.focus == ToolFocus::Detail {
                state.focus = ToolFocus::ToolList;
            } else {
                outcome.next_screen = Some(Screen::Dashboard { scroll: 0 });
            }
            return Ok(outcome);
        }
        KeyCode::Char('q') => *quit = true,
        KeyCode::Down | KeyCode::Char('j') => match state.focus {
            ToolFocus::ToolList => {
                if len > 0 && index + 1 < len {
                    state.list_state.select(Some(index + 1));
                }
            }
            ToolFocus::Detail => state.detail_scroll = state.detail_scroll.saturating_add(1),
        },
        KeyCode::Up | KeyCode::Char('k') => match state.focus {
            ToolFocus::ToolList => {
                if len > 0 && index > 0 {
                    state.list_state.select(Some(index - 1));
                }
            }
            ToolFocus::Detail => {
                state.detail_scroll = state.detail_scroll.saturating_sub(1);
            }
        },
        KeyCode::Left | KeyCode::Char('h') => match state.focus {
            ToolFocus::ToolList => {}
            ToolFocus::Detail => state.focus = ToolFocus::ToolList,
        },
        KeyCode::Right | KeyCode::Char('l') => match state.focus {
            ToolFocus::ToolList => state.focus = ToolFocus::Detail,
            ToolFocus::Detail => {}
        },
        KeyCode::Char('r') => state.refresh(install_root),
        KeyCode::Char(' ') => {
            if len > 0 && state.focus == ToolFocus::ToolList {
                state.selected[index] = !state.selected[index];
            }
        }
        KeyCode::Enter => {
            if len > 0 && state.focus == ToolFocus::ToolList {
                state.focus = ToolFocus::Detail;
            }
        }
        KeyCode::Char('a') => state.select_all(),
        KeyCode::Char('m') => state.select_installable(),
        KeyCode::Char('p') => state.select_installed(),
        KeyCode::Char('c') => state.selected.iter_mut().for_each(|flag| *flag = false),
        KeyCode::Char('i') | KeyCode::Char('I') => {
            let (screen, background) =
                start_tools_action(state, ToolAction::Install, install_root, action_running)?;
            outcome.next_screen = Some(screen);
            outcome.background_action = background;
            return Ok(outcome);
        }
        KeyCode::Char('u') | KeyCode::Char('U') => {
            let (screen, background) =
                start_tools_action(state, ToolAction::Update, install_root, action_running)?;
            outcome.next_screen = Some(screen);
            outcome.background_action = background;
            return Ok(outcome);
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            let (screen, background) =
                start_tools_action(state, ToolAction::Uninstall, install_root, action_running)?;
            outcome.next_screen = Some(screen);
            outcome.background_action = background;
            return Ok(outcome);
        }
        _ => {}
    }

    Ok(outcome)
}

fn start_tools_action(
    state: &mut ToolManagerState,
    action: ToolAction,
    install_root: Option<&Path>,
    action_running: bool,
) -> Result<(Screen, Option<BackgroundAction>)> {
    if action_running {
        return Ok((
            Screen::Output {
                title: "Action already running".to_string(),
                lines: vec![
                    "Another tools action is still running.".to_string(),
                    "Please wait for it to finish before starting a new one.".to_string(),
                ],
                scroll: 0,
            },
            None,
        ));
    }

    let any_selected = state.selected.iter().any(|flag| *flag);
    let chosen = state.selected_tools_for_action(action);
    if chosen.is_empty() {
        let message = if any_selected {
            format!("Selected tools cannot be {}.", action_past(action))
        } else {
            format!("Current tool cannot be {}.", action_past(action))
        };
        return Ok((
            Screen::Output {
                title: format!("{} tools", action_title(action)),
                lines: vec![
                    message,
                    "Tip: move cursor to an actionable tool, or select multiple with Space."
                        .to_string(),
                ],
                scroll: 0,
            },
            None,
        ));
    }

    let (tx, rx) = mpsc::channel();
    let install_root_buf = install_root.map(Path::to_path_buf);
    let action_label = action_title(action).to_string();
    let tool_count = chosen.len();

    thread::spawn(move || {
        let result_lines =
            operations::execute_tool_action(action, &chosen, install_root_buf.as_deref())
                .map(operations::flatten_command_results)
                .unwrap_or_else(|err| vec![format!("Error: {err}")]);
        let title = format!("{} tools", action_label);
        let _ = tx.send(ActionOutcome {
            title,
            lines: result_lines,
        });
    });

    Ok((
        Screen::Output {
            title: format!("{} tools", action_title(action)),
            lines: vec![
                format!(
                    "Running {} for {} tool(s)...",
                    action_title(action),
                    tool_count
                ),
                "Please wait. Enter/Esc is available after completion.".to_string(),
            ],
            scroll: 0,
        },
        Some(BackgroundAction { rx }),
    ))
}

fn handle_config_menu_key(
    state: &mut ratatui::widgets::ListState,
    code: KeyCode,
    quit: &mut bool,
) -> Option<Screen> {
    let items = config_menu_items();
    let selected = state.selected().unwrap_or(0);
    match code {
        KeyCode::Char('q') => *quit = true,
        KeyCode::Esc => return Some(Screen::Dashboard { scroll: 0 }),
        KeyCode::Down | KeyCode::Char('j') => state.select(Some((selected + 1) % items.len())),
        KeyCode::Up | KeyCode::Char('k') => {
            state.select(Some((selected + items.len() - 1) % items.len()))
        }
        KeyCode::Enter => {
            use super::ConfigMenuAction;
            return match items[selected].1 {
                ConfigMenuAction::Global => Some(Screen::Form(FormState::from_global())),
                ConfigMenuAction::Git => Some(Screen::Form(FormState::from_git())),
                ConfigMenuAction::Claude => Some(Screen::Form(FormState::from_claude())),
                ConfigMenuAction::Codex => Some(Screen::Form(FormState::from_codex())),
                ConfigMenuAction::Back => Some(Screen::Dashboard { scroll: 0 }),
            };
        }
        _ => {}
    }
    None
}

fn handle_form_key(form: &mut FormState, code: KeyCode) -> Result<Option<Screen>> {
    let total = form.fields.len() + 3;
    match code {
        KeyCode::Esc => return Ok(Some(new_config_menu())),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
            form.selected = (form.selected + 1) % total
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
            form.selected = (form.selected + total - 1) % total
        }
        KeyCode::Backspace => {
            if form.selected < form.fields.len() {
                form.fields[form.selected].value.pop();
            }
        }
        KeyCode::Char('o') | KeyCode::Char('O') => return open_form_in_editor(form).map(Some),
        KeyCode::Char(c) => {
            if form.selected < form.fields.len() && !c.is_control() {
                form.fields[form.selected].value.push(c);
            }
        }
        KeyCode::Enter => {
            if form.selected == form.fields.len() {
                save_form(form)?;
                return Ok(Some(Screen::Output {
                    title: format!("Saved {}", form.title),
                    lines: vec![format!("{} configuration updated.", form.title)],
                    scroll: 0,
                }));
            }
            if form.selected == form.fields.len() + 1 {
                return open_form_in_editor(form).map(Some);
            }
            if form.selected == form.fields.len() + 2 {
                return Ok(Some(new_config_menu()));
            }
        }
        _ => {}
    }
    Ok(None)
}

fn handle_skill_key(
    state: &mut ratatui::widgets::ListState,
    skills: &[crate::skills::Skill],
    selected: &mut [bool],
    code: KeyCode,
    quit: &mut bool,
) -> Result<Option<Screen>> {
    let index = state.selected().unwrap_or(0);
    match code {
        KeyCode::Esc => return Ok(Some(Screen::Dashboard { scroll: 0 })),
        KeyCode::Down | KeyCode::Char('j') => state.select(Some((index + 1) % (skills.len() + 2))),
        KeyCode::Up | KeyCode::Char('k') => {
            state.select(Some((index + skills.len() + 1) % (skills.len() + 2)))
        }
        KeyCode::Char(' ') if index < skills.len() => selected[index] = !selected[index],
        KeyCode::Char('a') => selected.iter_mut().for_each(|flag| *flag = true),
        KeyCode::Char('c') => selected.iter_mut().for_each(|flag| *flag = false),
        KeyCode::Char('q') => *quit = true,
        KeyCode::Enter => {
            if index == skills.len() {
                let chosen: Vec<crate::skills::Skill> = skills
                    .iter()
                    .cloned()
                    .zip(selected.iter())
                    .filter_map(|(skill, enabled)| enabled.then_some(skill))
                    .collect();
                let lines = skills::install_skills(&chosen)?;
                return Ok(Some(Screen::Output {
                    title: "Skills".into(),
                    lines,
                    scroll: 0,
                }));
            }
            if index == skills.len() + 1 {
                return Ok(Some(Screen::Dashboard { scroll: 0 }));
            }
        }
        _ => {}
    }
    Ok(None)
}

fn save_form(form: &FormState) -> Result<()> {
    use super::ConfigKind;
    match form.kind {
        ConfigKind::Global => {
            let config = config::GlobalConfig {
                editor: form.fields[0].value.clone(),
                proxy: form.fields[1].value.clone(),
            };
            config::save_global_config(&config)
        }
        ConfigKind::Git => {
            let config = config::GitConfig {
                user_name: form.fields[0].value.clone(),
                user_email: form.fields[1].value.clone(),
                default_branch: if form.fields[2].value.trim().is_empty() {
                    "main".to_string()
                } else {
                    form.fields[2].value.clone()
                },
            };
            config::save_git_config(&config)
        }
        ConfigKind::Claude => {
            let config = config::ClaudeConfig {
                base_url: if form.fields[0].value.trim().is_empty() {
                    "https://api.anthropic.com".to_string()
                } else {
                    form.fields[0].value.clone()
                },
                auth_token: form.fields[1].value.clone(),
            };
            config::save_claude_config(&config)
        }
        ConfigKind::Codex => {
            let config = config::CodexConfig {
                base_url: if form.fields[0].value.trim().is_empty() {
                    "https://api.openai.com".to_string()
                } else {
                    form.fields[0].value.clone()
                },
                api_key: form.fields[1].value.clone(),
                model: if form.fields[2].value.trim().is_empty() {
                    "gpt-5.2-codex".to_string()
                } else {
                    form.fields[2].value.clone()
                },
            };
            config::save_codex_config(&config)
        }
    }
}

fn open_form_in_editor(form: &FormState) -> Result<Screen> {
    let path = form_config_path(form.kind);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let global = config::load_global_config();
    let editor = if global.editor.trim().is_empty() {
        std::env::var("EDITOR").unwrap_or_default()
    } else {
        global.editor.trim().to_string()
    };

    if editor.trim().is_empty() {
        return Ok(Screen::Output {
            title: "Editor not configured".to_string(),
            lines: vec![
                "No editor command configured.".to_string(),
                "Go to Configure -> Global, set `editor`, then retry.".to_string(),
                "Example: code -g {file}".to_string(),
            ],
            scroll: 0,
        });
    }

    let file_arg = format!("\"{}\"", path.display());
    let command_line = if editor.contains("{file}") {
        editor.replace("{file}", &file_arg)
    } else {
        format!("{editor} {file_arg}")
    };

    Command::new("cmd").args(["/C", &command_line]).spawn()?;

    Ok(Screen::Output {
        title: format!("Opened {}", form.title),
        lines: vec![
            format!("Opened in editor: {}", path.display()),
            format!("Command: {command_line}"),
        ],
        scroll: 0,
    })
}

fn form_config_path(kind: super::ConfigKind) -> std::path::PathBuf {
    use super::ConfigKind;
    match kind {
        ConfigKind::Global => config::global_config_path(),
        ConfigKind::Git => config::git_config_path(),
        ConfigKind::Claude => config::claude_settings_path(),
        ConfigKind::Codex => config::codex_config_path(),
    }
}
