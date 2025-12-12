use anyhow::{Context, Result};
use aof_core::AgentConfig;
use aof_runtime::Runtime;
use std::fs;
use std::io::{self, IsTerminal};
use std::sync::{Arc, Mutex};
use tracing::info;
use crate::resources::ResourceType;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
    text::{Line, Span},
    style::{Modifier, Color, Style},
    layout::{Layout, Direction, Alignment, Constraint},
};
use std::time::Instant;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

/// Execute a resource (agent, workflow, job) with configuration and input
pub async fn execute(
    resource_type: &str,
    name_or_config: &str,
    input: Option<&str>,
    output: &str,
) -> Result<()> {
    // Parse resource type
    let rt = ResourceType::from_str(resource_type)
        .ok_or_else(|| anyhow::anyhow!("Unknown resource type: {}", resource_type))?;

    match rt {
        ResourceType::Agent => run_agent(name_or_config, input, output).await,
        ResourceType::Workflow => run_workflow(name_or_config, input, output).await,
        ResourceType::Job => run_job(name_or_config, input, output).await,
        _ => {
            anyhow::bail!("Resource type '{}' cannot be run directly", resource_type)
        }
    }
}

/// Run an agent with configuration
async fn run_agent(config: &str, input: Option<&str>, output: &str) -> Result<()> {
    info!("Loading agent config from: {}", config);

    // Load and parse agent configuration
    let config_content = fs::read_to_string(config)
        .with_context(|| format!("Failed to read config file: {}", config))?;

    let agent_config: AgentConfig = serde_yaml::from_str(&config_content)
        .with_context(|| format!("Failed to parse agent config from: {}", config))?;

    let agent_name = agent_config.name.clone();
    info!("Agent loaded: {}", agent_name);

    // Create runtime and load agent
    let mut runtime = Runtime::new();
    runtime
        .load_agent_from_config(agent_config)
        .await
        .context("Failed to load agent")?;

    // Check if interactive mode should be enabled (when no input provided and stdin is a TTY)
    if input.is_none() && io::stdin().is_terminal() {
        // Interactive REPL mode - only when stdin is a TTY (terminal)
        run_agent_interactive(&runtime, &agent_name, output).await?;
        return Ok(());
    }

    // Single execution mode
    let input_str = input.unwrap_or("default input");
    let result = runtime
        .execute(&agent_name, input_str)
        .await
        .context("Failed to execute agent")?;

    // Output result in requested format
    output_result(&agent_name, &result, output)?;

    Ok(())
}

/// Application state for TUI
struct AppState {
    chat_history: Vec<(String, String)>, // (role, message)
    current_input: String,
    logs: Vec<String>,
    agent_busy: bool,
    last_error: Option<String>,
    execution_start: Option<Instant>,
    execution_time_ms: u128,
    message_count: usize,
    spinner_state: u8,
}

impl AppState {
    fn new() -> Self {
        Self {
            chat_history: Vec::new(),
            current_input: String::new(),
            logs: Vec::new(),
            agent_busy: false,
            last_error: None,
            execution_start: None,
            execution_time_ms: 0,
            message_count: 0,
            spinner_state: 0,
        }
    }

    fn update_execution_time(&mut self) {
        if let Some(start) = self.execution_start {
            self.execution_time_ms = start.elapsed().as_millis();
        }
    }

    fn next_spinner(&mut self) {
        self.spinner_state = (self.spinner_state + 1) % 4;
    }

    fn get_spinner(&self) -> &str {
        match self.spinner_state {
            0 => "◐",
            1 => "◓",
            2 => "◑",
            3 => "◒",
            _ => "◐",
        }
    }
}

/// Run agent in interactive REPL mode with two-column TUI
async fn run_agent_interactive(runtime: &Runtime, agent_name: &str, _output: &str) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app state
    let mut app_state = AppState::new();
    let should_quit = Arc::new(Mutex::new(false));

    // Draw initial screen
    terminal.draw(|f| ui(f, agent_name, &app_state))?;

    // Main loop
    loop {
        // Check for quit
        if *should_quit.lock().unwrap() {
            break;
        }

        // Handle user input (non-blocking)
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') if key.modifiers == crossterm::event::KeyModifiers::CONTROL => {
                        break;
                    }
                    KeyCode::Enter => {
                        let trimmed = app_state.current_input.trim();

                        if trimmed.is_empty() {
                            // Do nothing for empty input
                        } else if trimmed.to_lowercase() == "exit" || trimmed.to_lowercase() == "quit" {
                            break;
                        } else if trimmed.to_lowercase() == "help" {
                            app_state.chat_history.push(("system".to_string(),
                                "Available: help, exit, quit. Type normally to chat with agent.".to_string()));
                        } else {
                            // Execute agent
                            app_state.chat_history.push(("user".to_string(), trimmed.to_string()));
                            app_state.agent_busy = true;
                            app_state.last_error = None;
                            app_state.execution_start = Some(Instant::now());
                            app_state.message_count = app_state.chat_history.len();

                            // Draw busy state
                            terminal.draw(|f| ui(f, agent_name, &app_state))?;

                            // Execute
                            match runtime.execute(agent_name, trimmed).await {
                                Ok(result) => {
                                    app_state.chat_history.push(("assistant".to_string(), result));
                                }
                                Err(e) => {
                                    let error_msg = format!("Error: {}", e);
                                    app_state.chat_history.push(("error".to_string(), error_msg.clone()));
                                    app_state.last_error = Some(error_msg);
                                }
                            }
                            app_state.agent_busy = false;
                            app_state.update_execution_time();
                        }

                        app_state.current_input.clear();
                    }
                    KeyCode::Backspace => {
                        app_state.current_input.pop();
                    }
                    KeyCode::Char(c) => {
                        app_state.current_input.push(c);
                    }
                    _ => {}
                }
            }
        }

        // Update animation state for spinner
        if app_state.agent_busy {
            app_state.next_spinner();
            app_state.update_execution_time();
        }

        // Redraw UI
        terminal.draw(|f| ui(f, agent_name, &app_state))?;
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    println!("\n-- Exiting Agentic Ops Framework --\n");
    Ok(())
}

/// Render the TUI with elegant professional styling for DevOps engineers
fn ui(f: &mut Frame, agent_name: &str, app: &AppState) {
    // Professional color scheme (dark terminal friendly)
    let primary_blue = Color::Rgb(0, 150, 200);
    let accent_green = Color::Rgb(0, 200, 100);
    let warning_orange = Color::Rgb(255, 165, 0);
    let error_red = Color::Rgb(255, 80, 80);

    // Main layout with footer for metrics
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(f.size());

    // Content area
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_layout[0]);

    // Left panel - Chat Interface
    let chat_block = Block::default()
        .title(Span::styled(
            format!(" {} ", agent_name.to_uppercase()),
            Style::default().fg(primary_blue).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Thick)
        .border_style(Style::default().fg(primary_blue))
        .padding(ratatui::widgets::Padding::symmetric(1, 0));

    let mut chat_lines = Vec::new();

    // Add conversation history
    for (role, msg) in &app.chat_history {
        let (style, prefix) = match role.as_str() {
            "user" => (
                Style::default().fg(Color::White),
                " ❯ ",
            ),
            "assistant" => (
                Style::default().fg(accent_green).add_modifier(Modifier::BOLD),
                " ◈ ",
            ),
            "error" => (
                Style::default().fg(error_red),
                " ✗ ",
            ),
            _ => (
                Style::default().fg(Color::DarkGray),
                " ► ",
            ),
        };

        for line in msg.lines() {
            chat_lines.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(line, style),
            ]));
        }
        chat_lines.push(Line::from("")); // Spacing
    }

    // Input line with active indicator
    if app.agent_busy {
        let time_str = format!("{}ms", app.execution_time_ms);
        let busy_indicator = format!("{} Processing... {}", app.get_spinner(), time_str);
        chat_lines.push(Line::from(Span::styled(
            busy_indicator,
            Style::default().fg(warning_orange).add_modifier(Modifier::DIM),
        )));
    } else {
        let mut input_spans = vec![Span::raw(" ❯ ")];

        // Show input with cursor
        if app.current_input.is_empty() {
            input_spans.push(Span::styled("_", Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)));
        } else {
            input_spans.push(Span::raw(&app.current_input));
            input_spans.push(Span::styled("_", Style::default().fg(primary_blue).add_modifier(Modifier::BOLD)));
        }
        chat_lines.push(Line::from(input_spans));
    }

    let chat_para = Paragraph::new(chat_lines)
        .block(chat_block)
        .wrap(Wrap { trim: true })
        .scroll((
            (app.chat_history.len() as u16).saturating_sub(chunks[0].height.saturating_sub(3) / 2),
            0,
        ));

    f.render_widget(chat_para, chunks[0]);

    // Right panel - System Logs with elegance
    let logs_block = Block::default()
        .title(Span::styled(
            " SYSTEM LOG ",
            Style::default().fg(primary_blue).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Thick)
        .border_style(Style::default().fg(primary_blue))
        .padding(ratatui::widgets::Padding::symmetric(1, 0));

    let log_lines: Vec<Line> = app.logs.iter()
        .map(|log| {
            let style = if log.contains("ERROR") {
                Style::default().fg(error_red).add_modifier(Modifier::BOLD)
            } else if log.contains("WARN") {
                Style::default().fg(warning_orange)
            } else if log.contains("DEBUG") {
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)
            } else if log.contains("INFO") {
                Style::default().fg(Color::White).add_modifier(Modifier::DIM)
            } else {
                Style::default().fg(Color::Gray)
            };

            let trimmed = log.chars().take(chunks[1].width.saturating_sub(4) as usize).collect::<String>();
            Line::from(Span::styled(trimmed, style))
        })
        .collect();

    let logs_para = Paragraph::new(log_lines)
        .block(logs_block)
        .wrap(Wrap { trim: true })
        .scroll((
            (app.logs.len() as u16).saturating_sub(chunks[1].height.saturating_sub(3) / 2),
            0,
        ));

    f.render_widget(logs_para, chunks[1]);

    // Footer metrics bar
    let metrics_text = if app.agent_busy {
        format!(
            "  ⧖ {:>5}ms  │  {} {} messages  │  Status: Active",
            app.execution_time_ms,
            app.get_spinner(),
            app.message_count / 2
        )
    } else {
        format!(
            "  ✓ Completed  │  {} messages  │  Last execution: {}ms",
            app.message_count / 2,
            app.execution_time_ms
        )
    };

    let metrics_block = Block::default()
        .style(Style::default().fg(primary_blue).bg(Color::Black))
        .padding(ratatui::widgets::Padding::symmetric(1, 0));

    let metrics_para = Paragraph::new(metrics_text)
        .block(metrics_block)
        .style(Style::default().fg(accent_green));

    f.render_widget(metrics_para, main_layout[1]);
}

/// Format and output agent result
fn output_result(agent_name: &str, result: &str, output: &str) -> Result<()> {
    match output {
        "json" => {
            let json_output = serde_json::json!({
                "success": true,
                "agent": agent_name,
                "result": result
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        "yaml" => {
            let yaml_output = serde_yaml::to_string(&serde_json::json!({
                "success": true,
                "agent": agent_name,
                "result": result
            }))?;
            println!("{}", yaml_output);
        }
        "text" | _ => {
            println!("Agent: {}", agent_name);
            println!("Result: {}", result);
        }
    }
    Ok(())
}

/// Run a workflow (placeholder)
async fn run_workflow(name_or_config: &str, input: Option<&str>, output: &str) -> Result<()> {
    println!("Run workflow - Not yet implemented");
    println!("Workflow: {}", name_or_config);
    if let Some(inp) = input {
        println!("Input: {}", inp);
    }
    println!("Output format: {}", output);
    Ok(())
}

/// Run a job (placeholder)
async fn run_job(name_or_config: &str, input: Option<&str>, output: &str) -> Result<()> {
    println!("Run job - Not yet implemented");
    println!("Job: {}", name_or_config);
    if let Some(inp) = input {
        println!("Input: {}", inp);
    }
    println!("Output format: {}", output);
    Ok(())
}
