mod help;
mod keys;
mod render;

use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::DefaultTerminal;
use ratatui::widgets::ListState;

use crate::config;
use crate::operations::ToolAction;
use crate::skills::{self, Skill};
use crate::status::{self, ToolStatus};

pub fn run_tui(default_install_root: Option<PathBuf>, repo_root: PathBuf) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let terminal = ratatui::init();

    let result = run_app(terminal, default_install_root, repo_root);

    ratatui::restore();
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    result
}

pub(crate) struct App {
    pub(crate) install_root: Option<PathBuf>,
    pub(crate) repo_root: PathBuf,
    pub(crate) screen: Screen,
    pub(crate) page_stack: Vec<Screen>,
    pub(crate) background_action: Option<BackgroundAction>,
    pub(crate) help: Option<HelpPopup>,
    pub(crate) quit_confirm: bool,
    pub(crate) nav_focus: bool,
}

pub(crate) enum Screen {
    Dashboard {
        scroll: u16,
    },
    Tools(ToolManagerState),
    ConfigMenu {
        state: ListState,
    },
    Form(FormState),
    SkillSelection {
        state: ListState,
        skills: Vec<Skill>,
        selected: Vec<bool>,
    },
    Output {
        title: String,
        lines: Vec<String>,
        scroll: u16,
    },
}

pub(crate) struct HelpPopup {
    pub(crate) title: String,
    pub(crate) lines: Vec<String>,
    pub(crate) scroll: u16,
}

pub(crate) struct BackgroundAction {
    pub(crate) rx: Receiver<ActionOutcome>,
}

pub(crate) struct ActionOutcome {
    pub(crate) title: String,
    pub(crate) lines: Vec<String>,
}

pub(crate) struct ToolManagerState {
    pub(crate) list_state: ListState,
    pub(crate) selected: Vec<bool>,
    pub(crate) statuses: Vec<ToolStatus>,
    pub(crate) focus: ToolFocus,
    pub(crate) detail_scroll: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ToolFocus {
    ToolList,
    Detail,
}

pub(crate) struct ToolsKeyOutcome {
    pub(crate) next_screen: Option<Screen>,
    pub(crate) background_action: Option<BackgroundAction>,
}

#[derive(Clone, Copy)]
pub(crate) enum ConfigKind {
    Global,
    Git,
    Claude,
    Codex,
}

pub(crate) struct FormField {
    pub(crate) label: &'static str,
    pub(crate) value: String,
    pub(crate) secret: bool,
}

pub(crate) struct FormState {
    pub(crate) kind: ConfigKind,
    pub(crate) title: &'static str,
    pub(crate) fields: Vec<FormField>,
    pub(crate) selected: usize,
}

#[derive(Clone, Copy)]
pub(crate) enum ConfigMenuAction {
    Global,
    Git,
    Claude,
    Codex,
    Back,
}

impl App {
    fn new(install_root: Option<PathBuf>, repo_root: PathBuf) -> Self {
        Self {
            install_root,
            repo_root,
            screen: Screen::Dashboard { scroll: 0 },
            page_stack: Vec::new(),
            background_action: None,
            help: None,
            quit_confirm: false,
            nav_focus: false,
        }
    }
}

impl FormState {
    pub(crate) fn from_global() -> Self {
        let current = config::load_global_config();
        Self {
            kind: ConfigKind::Global,
            title: "Global",
            fields: vec![
                FormField {
                    label: "editor command",
                    value: current.editor,
                    secret: false,
                },
                FormField {
                    label: "proxy URL",
                    value: current.proxy,
                    secret: false,
                },
            ],
            selected: 0,
        }
    }

    pub(crate) fn from_git() -> Self {
        let current = config::load_git_config();
        Self {
            kind: ConfigKind::Git,
            title: "Git",
            fields: vec![
                FormField {
                    label: "user.name",
                    value: current.user_name,
                    secret: false,
                },
                FormField {
                    label: "user.email",
                    value: current.user_email,
                    secret: false,
                },
                FormField {
                    label: "default branch",
                    value: current.default_branch,
                    secret: false,
                },
            ],
            selected: 0,
        }
    }

    pub(crate) fn from_claude() -> Self {
        let current = config::load_claude_config();
        Self {
            kind: ConfigKind::Claude,
            title: "Claude Code",
            fields: vec![
                FormField {
                    label: "base URL",
                    value: current.base_url,
                    secret: false,
                },
                FormField {
                    label: "auth token",
                    value: current.auth_token,
                    secret: true,
                },
            ],
            selected: 0,
        }
    }

    pub(crate) fn from_codex() -> Self {
        let current = config::load_codex_config("gpt-5.2-codex");
        Self {
            kind: ConfigKind::Codex,
            title: "Codex",
            fields: vec![
                FormField {
                    label: "base URL",
                    value: current.base_url,
                    secret: false,
                },
                FormField {
                    label: "API key",
                    value: current.api_key,
                    secret: true,
                },
                FormField {
                    label: "model",
                    value: current.model,
                    secret: false,
                },
            ],
            selected: 0,
        }
    }
}

impl ToolManagerState {
    fn new(install_root: Option<&Path>) -> Self {
        let mut statuses = status::collect_statuses(install_root);
        status::populate_update_info(&mut statuses);
        let mut state = ListState::default();
        state.select((!statuses.is_empty()).then_some(0));
        Self {
            list_state: state,
            selected: vec![false; statuses.len()],
            statuses,
            focus: ToolFocus::ToolList,
            detail_scroll: 0,
        }
    }

    pub(crate) fn refresh(&mut self, install_root: Option<&Path>) {
        self.statuses = status::collect_statuses(install_root);
        status::populate_update_info(&mut self.statuses);
        self.selected.resize(self.statuses.len(), false);
        self.detail_scroll = 0;
        let selected = self.list_state.selected().unwrap_or(0);
        if self.statuses.is_empty() {
            self.list_state.select(None);
        } else if selected >= self.statuses.len() {
            self.list_state.select(Some(self.statuses.len() - 1));
        }
    }

    pub(crate) fn select_all(&mut self) {
        self.selected.iter_mut().for_each(|flag| *flag = true);
    }

    pub(crate) fn select_installable(&mut self) {
        for (flag, status) in self.selected.iter_mut().zip(self.statuses.iter()) {
            *flag = is_actionable(status, ToolAction::Install);
        }
    }

    pub(crate) fn select_installed(&mut self) {
        for (flag, status) in self.selected.iter_mut().zip(self.statuses.iter()) {
            *flag = status.path.is_some();
        }
    }

    pub(crate) fn selected_tools_for_action(&self, action: ToolAction) -> Vec<&'static crate::tool::Tool> {
        let selected: Vec<&'static crate::tool::Tool> = self
            .statuses
            .iter()
            .zip(self.selected.iter())
            .filter_map(|(status, enabled)| {
                if !is_actionable(status, action) {
                    return None;
                }
                if !enabled {
                    return None;
                }
                Some(status.tool)
            })
            .collect();

        if !selected.is_empty() {
            return selected;
        }

        self.list_state
            .selected()
            .and_then(|index| self.statuses.get(index))
            .filter(|status| is_actionable(status, action))
            .map(|status| vec![status.tool])
            .unwrap_or_default()
    }

    pub(crate) fn action_counts(&self) -> (usize, usize, usize) {
        let installable = self
            .statuses
            .iter()
            .filter(|status| is_actionable(status, ToolAction::Install))
            .count();
        let updatable = self
            .statuses
            .iter()
            .filter(|status| is_actionable(status, ToolAction::Update))
            .count();
        let removable = self
            .statuses
            .iter()
            .filter(|status| is_actionable(status, ToolAction::Uninstall))
            .count();
        (installable, updatable, removable)
    }
}

fn run_app(
    mut terminal: DefaultTerminal,
    install_root: Option<PathBuf>,
    repo_root: PathBuf,
) -> Result<()> {
    let mut app = App::new(install_root, repo_root);

    loop {
        poll_background_action(&mut app);
        terminal.draw(|frame| render::render(frame, &mut app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if keys::handle_key(&mut app, key.code)? {
                    break;
                }
            }
        }
    }

    Ok(())
}

fn poll_background_action(app: &mut App) {
    use std::sync::mpsc::TryRecvError;

    let Some(background) = app.background_action.as_ref() else {
        return;
    };

    let outcome = match background.rx.try_recv() {
        Ok(outcome) => outcome,
        Err(TryRecvError::Empty) => return,
        Err(TryRecvError::Disconnected) => {
            app.background_action = None;
            return;
        }
    };

    app.background_action = None;

    if let Screen::Output {
        title,
        lines,
        scroll,
    } = &mut app.screen
    {
        *title = outcome.title;
        *lines = outcome.lines;
        *scroll = 0;
        if let Err(err) = status::refresh_process_env_from_registry() {
            lines.push(format!(
                "Warning: failed to refresh PATH from registry: {err}"
            ));
        }
    }

    if let Screen::Tools(state) = &mut app.screen {
        state.refresh(app.install_root.as_deref());
    }
    if let Some(Screen::Tools(state)) = app.page_stack.last_mut() {
        state.refresh(app.install_root.as_deref());
    }
}

pub(crate) fn new_tool_manager(install_root: Option<&Path>) -> ToolManagerState {
    ToolManagerState::new(install_root)
}

pub(crate) fn new_config_menu() -> Screen {
    let mut state = ListState::default();
    state.select(Some(0));
    Screen::ConfigMenu { state }
}

pub(crate) fn new_skill_selection(repo_root: &Path) -> Screen {
    let mut state = ListState::default();
    state.select(Some(0));
    let skills = skills::available_skills(repo_root);
    let selected = vec![true; skills.len()];
    Screen::SkillSelection {
        state,
        skills,
        selected,
    }
}

pub(crate) fn top_page_index(screen: &Screen) -> usize {
    match screen {
        Screen::Dashboard { .. } => 0,
        Screen::Tools(_) => 1,
        Screen::ConfigMenu { .. } | Screen::Form(_) => 2,
        Screen::SkillSelection { .. } => 3,
        Screen::Output { .. } => 0,
    }
}

pub(crate) fn screen_for_top_page(index: usize, install_root: Option<&Path>, repo_root: &Path) -> Screen {
    match index % 4 {
        0 => Screen::Dashboard { scroll: 0 },
        1 => Screen::Tools(new_tool_manager(install_root)),
        2 => new_config_menu(),
        _ => new_skill_selection(repo_root),
    }
}

pub(crate) fn next_top_screen(screen: &Screen, install_root: Option<&Path>, repo_root: &Path) -> Screen {
    screen_for_top_page(top_page_index(screen) + 1, install_root, repo_root)
}

pub(crate) fn previous_top_screen(screen: &Screen, install_root: Option<&Path>, repo_root: &Path) -> Screen {
    screen_for_top_page((top_page_index(screen) + 3) % 4, install_root, repo_root)
}

pub(crate) fn screen_name(screen: &Screen) -> &'static str {
    match screen {
        Screen::Dashboard { .. } => "Dashboard",
        Screen::Tools(_) => "Manage Tools",
        Screen::ConfigMenu { .. } => "Configure",
        Screen::Form(form) => form.title,
        Screen::SkillSelection { .. } => "Skills",
        Screen::Output { .. } => "Output",
    }
}

pub(crate) fn config_menu_items() -> Vec<(&'static str, ConfigMenuAction)> {
    vec![
        ("Global settings (editor/proxy)", ConfigMenuAction::Global),
        ("Configure Git", ConfigMenuAction::Git),
        ("Configure Claude Code", ConfigMenuAction::Claude),
        ("Configure Codex", ConfigMenuAction::Codex),
        ("Back", ConfigMenuAction::Back),
    ]
}

pub(crate) fn is_actionable(status: &ToolStatus, action: ToolAction) -> bool {
    match action {
        ToolAction::Install => status.path.is_none(),
        ToolAction::Update => status.path.is_some() && status.update_available,
        ToolAction::Uninstall => status.path.is_some(),
    }
}

pub(crate) fn action_title(action: ToolAction) -> &'static str {
    match action {
        ToolAction::Install => "Install",
        ToolAction::Update => "Update",
        ToolAction::Uninstall => "Uninstall",
    }
}

pub(crate) fn action_past(action: ToolAction) -> &'static str {
    match action {
        ToolAction::Install => "installed",
        ToolAction::Update => "updated",
        ToolAction::Uninstall => "uninstalled",
    }
}
