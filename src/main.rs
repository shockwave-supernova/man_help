use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use regex::Regex;
use std::env;
use std::io;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Language {
    System,
    English,
}

#[derive(Clone, Debug)]
struct Flag {
    short: Option<String>,
    long: Option<String>,
    desc: String,
    selected: bool,
}

impl Flag {
    fn to_display_string(&self) -> String {
        let checkmark = if self.selected { "[x]" } else { "[ ]" };

        let flags_str = match (&self.short, &self.long) {
            (Some(s), Some(l)) => format!("{}, {}", s, l),
            (Some(s), None) => format!("{}", s),
            (None, Some(l)) => format!("    {}", l),
            (None, None) => "???".to_string(),
        };

        format!("{} {:<25} | {}", checkmark, flags_str, self.desc)
    }

    fn as_arg(&self) -> String {
        if let Some(l) = &self.long {
            l.clone()
        } else {
            self.short.clone().unwrap_or_default()
        }
    }
}

enum ExitAction {
    Execute,
    Print,
    Cancel,
}

struct App {
    target_cmd: String,
    flags: Vec<Flag>,
    list_state: ListState,
    should_quit: bool,
    exit_action: ExitAction,
    current_lang: Language,
}

impl App {
    fn new(target_cmd: String, flags: Vec<Flag>) -> Self {
        let mut list_state = ListState::default();
        if !flags.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            target_cmd,
            flags,
            list_state,
            should_quit: false,
            exit_action: ExitAction::Cancel,
            current_lang: Language::System,
        }
    }

    fn next(&mut self) {
        if self.flags.is_empty() { return; }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.flags.len() - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.flags.is_empty() { return; }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 { self.flags.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn toggle_selection(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if i < self.flags.len() {
                self.flags[i].selected = !self.flags[i].selected;
            }
        }
    }

    fn toggle_language(&mut self) {
        let new_lang = match self.current_lang {
            Language::System => Language::English,
            Language::English => Language::System,
        };

        let selected_args: Vec<String> = self.get_selected_args();

        match fetch_flags(&self.target_cmd, new_lang) {
            Ok(mut new_flags) => {
                for flag in &mut new_flags {
                    if selected_args.contains(&flag.as_arg()) {
                        flag.selected = true;
                    }
                }
                self.flags = new_flags;
                self.current_lang = new_lang;
                if let Some(i) = self.list_state.selected() {
                    if i >= self.flags.len() {
                        self.list_state.select(Some(0));
                    }
                }
            }
            Err(_) => {}
        }
    }

    fn get_selected_args(&self) -> Vec<String> {
        self.flags.iter()
            .filter(|f| f.selected)
            .map(|f| f.as_arg())
            .collect()
    }

    fn build_preview_string(&self) -> String {
        let args = self.get_selected_args();
        if args.is_empty() {
            self.target_cmd.clone()
        } else {
            format!("{} {}", self.target_cmd, args.join(" "))
        }
    }
}

// Запуск с таймаутом
fn run_with_timeout(mut cmd: Command, timeout: Duration) -> Result<String> {
    // Важно: stdout/stderr должны быть piped, чтобы можно было читать их
    // и чтобы они не гадили в терминал.
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let start = Instant::now();
    let sleep_step = Duration::from_millis(50);

    loop {
        // try_wait() не блокирует поток. Возвращает Ok(None) если процесс еще идет.
        match child.try_wait() {
            Ok(Some(_status)) => {
                // Процесс завершился сам
                let output = child.wait_with_output()?; // Собираем выхлоп
                if output.status.success() {
                    return Ok(String::from_utf8_lossy(&output.stdout).to_string());
                } else {
                    return Err(anyhow::anyhow!("Команда вернула ошибку"));
                }
            },
            Ok(None) => {
                // Процесс еще идет. Проверяем таймаут.
                if start.elapsed() > timeout {
                    // УБИВАЕМ ПАРОВОЗ (команда sl стартовала и справка не отрабатывала пока поезд не проедет)
                    let _ = child.kill();
                    return Err(anyhow::anyhow!("Таймаут выполнения команды (возможно, интерактивная программа)"));
                }
                std::thread::sleep(sleep_step);
            },
            Err(e) => return Err(e.into()),
        }
    }
}

fn fetch_raw_help(cmd_name: &str, lang: Language) -> Result<String> {
    // 1. Пробуем --help с жестким таймаутом (1 секунда).
    // Если программа не успела выплюнуть справку за 1с, скорее всего это `sl` или `vim`

    let mut help_cmd = Command::new(cmd_name);
    help_cmd.arg("--help");
    help_cmd.env("COLUMNS", "500");
    if lang == Language::English { help_cmd.env("LC_ALL", "C"); }

    // Используем нашу функцию с таймаутом вместо простого .output()
    if let Ok(text) = run_with_timeout(help_cmd, Duration::from_secs(1)) {
        // Эвристика: если есть хотя бы 3 дефиса
        if text.matches(" -").count() >= 3 || text.matches("\n-").count() >= 3 {
            return Ok(text);
        }
    }

    // 2. Fallback: MAN
    // Если --help отвалился по таймауту или ошибке, идем в man
    let mut man_cmd = Command::new("man");
    man_cmd.arg(cmd_name);
    man_cmd.env("PAGER", "cat");
    man_cmd.env("MANROFFOPT", "-c");
    man_cmd.env("GROFF_NO_SGR", "1");
    man_cmd.env("COLUMNS", "500");

    if lang == Language::English { man_cmd.env("LC_ALL", "C"); }

    // Для man тоже можно использовать таймаут, но он обычно безопаснее (cat сразу отдает)
    // Но для надежности прогоним и его
    if let Ok(text) = run_with_timeout(man_cmd, Duration::from_secs(2)) {
        return Ok(text);
    }

    Err(anyhow::anyhow!("Не удалось получить справку."))
}

fn fetch_flags(cmd_name: &str, lang: Language) -> Result<Vec<Flag>> {
    let text = fetch_raw_help(cmd_name, lang)?;

    let re = Regex::new(r"(?m)^\s+(?:(?P<short>-[a-zA-Z0-9?])(?:,?\s+(?P<long>--[a-zA-Z0-9\-_]+))?|(?P<long_only>--[a-zA-Z0-9\-_]+))\s+(?P<desc>.+)$").unwrap();

    let mut flags = Vec::new();

    for cap in re.captures_iter(&text) {
        let short = cap.name("short").map(|m| m.as_str().to_string());

        let long = cap.name("long")
            .or_else(|| cap.name("long_only"))
            .map(|m| m.as_str().to_string());

        let desc = cap.name("desc").map(|m| m.as_str().trim().to_string()).unwrap_or_default();

        if short.is_some() || long.is_some() {
            if let Some(ref s) = short { if !s.starts_with('-') { continue; } }
            if let Some(ref l) = long { if !l.starts_with("--") { continue; } }
            if desc.len() < 2 { continue; }

            flags.push(Flag { short, long, desc, selected: false });
        }
    }

    if flags.is_empty() {
        return Err(anyhow::anyhow!("Текст справки получен, но флаги не найдены."));
    }

    Ok(flags)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let target = args.get(1).map(|s| s.as_str()).unwrap_or("ls");

    println!("Загрузка справки для '{}'...", target);

    let flags = match fetch_flags(target, Language::System) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("\nОшибка: {}", e);
            eprintln!("Попробуйте другую команду или проверьте, установлен ли 'man'.");
            return Ok(());
        }
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(target.to_string(), flags);
    let run_result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = run_result {
        eprintln!("Ошибка UI: {:?}", err);
        return Ok(());
    }

    match app.exit_action {
        ExitAction::Execute => {
            let args = app.get_selected_args();
            println!(">>> Запуск: {} {}", app.target_cmd, args.join(" "));
            println!("---------------------------------------------------");

            let status = Command::new(&app.target_cmd)
                .args(&args)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();

            match status {
                Ok(s) => {
                    if !s.success() {
                        println!("\nПроцесс завершен (код {}).", s);
                    }
                }
                Err(e) => eprintln!("\nНе удалось запустить команду: {}", e),
            }
        }
        ExitAction::Print => {
            println!("{}", app.build_preview_string());
        }
        ExitAction::Cancel => {
            println!("Отмена.");
        }
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.exit_action = ExitAction::Cancel;
                            app.should_quit = true;
                        },
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Char(' ') => app.toggle_selection(),
                        KeyCode::Enter => {
                            app.exit_action = ExitAction::Execute;
                            app.should_quit = true;
                        }
                        KeyCode::Char('p') => {
                            app.exit_action = ExitAction::Print;
                            app.should_quit = true;
                        }
                        KeyCode::Char('l') => {
                            app.toggle_language();
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5)]).split(f.area());

    let items: Vec<ListItem> = app.flags.iter().map(|flag| {
        let style = if flag.selected { Style::default().fg(Color::Green) } else { Style::default() };
        ListItem::new(flag.to_display_string()).style(style)
    }).collect();

    let source = match app.current_lang {
        Language::System => "Sys",
        Language::English => "EN",
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(" rlhelp: {} [{}] ", app.target_cmd, source)))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[0], &mut app.list_state);

    let help_text = vec![
        Line::from(vec![
            Span::styled("Предпросмотр: ", Style::default().fg(Color::Yellow)),
            Span::raw(app.build_preview_string())
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
            Span::raw(": Старт  "),
            Span::styled("[p]", Style::default().fg(Color::Cyan)),
            Span::raw(": Печать  "),
            Span::styled("[l]", Style::default().fg(Color::Magenta)),
            Span::raw(": Язык  "),
            Span::styled("[Space]", Style::default().fg(Color::DarkGray)),
            Span::raw(": Выбор  "),
            Span::styled("[Esc]", Style::default().fg(Color::Red)),
            Span::raw(": Выход"),
        ]),
    ];

    let preview = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(preview, chunks[1]);
}