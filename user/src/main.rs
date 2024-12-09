use std::{
    collections::LinkedList, io::{stdout, Write}, net::TcpStream, sync::Arc, time::Duration
};

use std::io;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

use chaos_core::{api::user_actions::{CreateScenario, LogSubscription, UserAction, UserActionResponse}, err::ChaosResult};
use rustls::{ClientConfig, RootCertStore};
use tungstenite::{stream::MaybeTlsStream, WebSocket};

const SERVER_ADDRESS: &str = env!("SERVER_ADDRESS");
const SERVER_PORT: &str = env!("SERVER_PORT");
pub const SERVER_CERTIFICATE: &[u8] = include_bytes!(env!("CA_CERT"));

#[derive(PartialEq, Eq, Default)]
pub enum SelectedWindow {
    #[default]
    Commands,
    AgentLogs,
    AppLogs
}

pub struct UserApp {
    pub position : usize,
    pub agent_logs_i : usize,
    pub app_logs_i : usize,
    pub exit: bool,
    pub client : WsClient,
    pub app_logs : LinkedList<[String; 2]>,
    pub agent_logs : LinkedList<[String; 2]>,
    pub output : LinkedList<String>,
    pub input : bool,
    pub input_text : String,
    pub command_state : CommandState,
    pub window : SelectedWindow,
    pub current_agent_completion : (u32, u32),
    pub current_agent : Option<String>,
    pub current_app : Option<String>
}
pub enum CommandState {
    None,
    CreateScenario(CreateScenarioState),
    StartScenario(SelectScenarioState),
    AgentLogs(SelectAgentState),
    AppLogs(SelectAgentState),
    Backup(BackupName),
}
#[derive(Default)]
pub struct CreateScenarioState {
    pub id : Option<String>,
    pub base_id : Option<String>
}
#[derive(Default)]
pub struct SelectScenarioState {
    pub id : Option<String>
}
#[derive(Default)]
pub struct SelectAgentState {
    pub name : Option<String>
}
#[derive(Default)]
pub struct BackupName {
    pub name : Option<String>
}

type WsClient = WebSocket<MaybeTlsStream<TcpStream>>;

const COMMAND_LIST : &[[&str; 2]] = &[
    ["List Agents", "List all agents"],
    ["All Agent logs", "Shows all agent logs"],
    ["Agent logs", "Shows an agent logs"],
    ["Stop agent logs", "Stops receiving agent logs"],
    ["All App logs", "Shows app logs of all agent"],
    ["App logs", "Shows app logs of an agent"],
    ["Stop app logs", "Stops receiving app logs"],
    ["Create scenario", "Creates a new testing scenario"],
    ["Start scenario", "Starts a testing scenario"],
    ["Stop scenario", "Stops a testing scenario"],
    ["Edit scenario", "Edit parameters of scenario"],
    ["List scenarios", "List all file scenario"],
    ["List test scenarios", "List all testing scenarios"],
    ["Backup", "Saves the server state"],
    ["Report", "Generates a markdown report"],
    ["Exit", "Exists the interface"]
];

const DEFAULT_ASCII_ART : &str = r#"______________                     ________                  ______  
__  ____/__  /_______ ________________  __ )____________________  /_ 
_  /    __  __ \  __ `/  __ \_  ___/_  __  |  _ \_  __ \  ___/_  __ \
/ /___  _  / / / /_/ // /_/ /(__  )_  /_/ //  __/  / / / /__ _  / / /
\____/  /_/ /_/\__,_/ \____//____/ /_____/ \___//_/ /_/\___/ /_/ /_/ 
"#;

const ASCII_ART : Option<&str> = option_env!("ASCIIART_USER");

fn main() -> io::Result<()> {
    let route = format!("wss://{}:{}/_user/connect", SERVER_ADDRESS, SERVER_PORT);
    let mut root_store = RootCertStore::empty();
    let cert = rustls_pemfile::read_one_from_slice(SERVER_CERTIFICATE)
        .unwrap()
        .unwrap()
        .0;
    let cert = match cert {
        rustls_pemfile::Item::X509Certificate(v) => v,
        _ => panic!("Invalid CA certificate format"),
    };
    root_store.add(cert).unwrap();
    let config = Arc::new(
        ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth(),
    );
    let sock = TcpStream::connect(format!("{}:{}", SERVER_ADDRESS, SERVER_PORT)).unwrap();

    let (client, _response) = tungstenite::client_tls_with_config(
        &route,
        sock,
        None,
        Some(tungstenite::Connector::Rustls(config)),
    )
    .unwrap();
    if let MaybeTlsStream::Rustls(stream) = client.get_ref() {
        //let _ = stream.set_nonblocking(true);
        let _ = stream
            .sock
            .set_read_timeout(Some(Duration::from_secs_f32(0.05)));
        let _ = stream
            .sock
            .set_write_timeout(Some(Duration::from_secs_f32(2.0)));
    }
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut app = UserApp::new(client);
    app.run(&mut terminal)?;
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

impl UserApp {
    pub fn new(client : WsClient) -> Self {
        Self {
            exit : false,
            position : 0,
            client,
            app_logs : LinkedList::new(),
            agent_logs : LinkedList::new(),
            output : LinkedList::new(),
            input : false,
            input_text : String::with_capacity(32),
            command_state : CommandState::None,
            window : SelectedWindow::Commands,
            agent_logs_i : 0,
            app_logs_i : 0,
            current_agent_completion : (0, 0),
            current_agent : None,
            current_app : None
        }
    }
    /// runs the application's main loop until the user quits
    pub fn run<B : Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        let mut pos = 0;
        let commands = COMMAND_LIST.iter().map(|v| {
            let mut row = Row::new(*v).white();
            if pos == self.position {
                row = row.light_yellow();
            }
            pos += 1;
            row
        });
        let output_rows = self.output.iter().map(|v| Row::new(vec![v.as_str()]));
        let output_rows = ConsoleIterator::new(output_rows, if self.input {
            Some(Row::new(vec![self.input_text.as_str()]).white())
        }else {
            None
        });
        let app_rows = self.app_logs.iter().skip(self.app_logs_i).map(|v| Row::new(vec![v[0].as_str(), v[1].as_str()]).white());
        let agent_rows = self.agent_logs.iter().skip(self.agent_logs_i).map(|v| Row::new(vec![v[0].as_str(), v[1].as_str()]).white());
        let main_layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(30), Constraint::Percentage(70)],
        )
        .split(frame.size());
        let left_pannel = Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(main_layout[0]);
        frame.render_widget(
            if self.window == SelectedWindow::Commands {
                Table::new(commands, [Constraint::Min(20),Constraint::Percentage(100)]).block(Block::bordered().style(border_style()).blue())
            }else {
                Table::new(commands, [Constraint::Min(20),Constraint::Percentage(100)]).block(Block::bordered().style(border_style()))
            },
            left_pannel[0],
        );
        frame.render_widget(
            Table::new(output_rows, [Constraint::Percentage(100)]).block(Block::bordered().style(border_style())),
            left_pannel[1],
        );
        let right_pannel = Layout::new(
            Direction::Vertical,
            [Constraint::Min(7), Constraint::Percentage(100)],
        )
        .split(main_layout[1]);
        let right_pannel_bottom = Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(right_pannel[1]);
        frame.render_widget(
            Paragraph::new(ASCII_ART.unwrap_or(DEFAULT_ASCII_ART)).block(Block::default().set_style(border_style()).borders(Borders::ALL)),
            right_pannel[0],
        );
        frame.render_widget(
            if self.window == SelectedWindow::AgentLogs {
                Table::new(agent_rows, [Constraint::Max(10), Constraint::Percentage(100)]).block(Block::bordered().style(border_style()).borders(Borders::ALL).title(format!(" Agent Logs ({}) ", self.current_agent.as_ref().map(|v|v.as_str()).unwrap_or("-"))).title_bottom(format!("{}/{}", self.current_agent_completion.0, self.current_agent_completion.1)).blue())
            }else {
                Table::new(agent_rows, [Constraint::Max(10), Constraint::Percentage(100)]).block(Block::bordered().style(border_style()).borders(Borders::ALL).title(format!(" Agent Logs ({}) ", self.current_agent.as_ref().map(|v|v.as_str()).unwrap_or("-"))).title_bottom(format!("{}/{}", self.current_agent_completion.0, self.current_agent_completion.1)))
            },
            right_pannel_bottom[0],
        );
        frame.render_widget(
            if self.window == SelectedWindow::AppLogs {
                Table::new(app_rows, [Constraint::Max(10), Constraint::Percentage(100)]).block(Block::bordered().style(border_style()).borders(Borders::ALL).title(format!(" App Logs ({}) ", self.current_app.as_ref().map(|v|v.as_str()).unwrap_or("-"))).blue())
            }else {
                Table::new(app_rows, [Constraint::Max(10), Constraint::Percentage(100)]).block(Block::bordered().style(border_style()).borders(Borders::ALL).title(format!(" App Logs ({}) ", self.current_app.as_ref().map(|v|v.as_str()).unwrap_or("-"))))
            },
            right_pannel_bottom[1],
        );
    }

    fn handle_events_commands(&mut self) -> io::Result<()> {
        self.check_executed_command();
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    if self.input {
                        if key.code == KeyCode::Backspace {
                            self.input_text.pop();
                        }else if key.code == KeyCode::Enter {
                            self.input = false;
                        }else {
                            if let KeyCode::Char(c) = key.code {
                                self.input_text.push(c);
                            }
                        }
                    }else {
                        if key.code == KeyCode::Char('q') {
                            self.exit = true;
                            return Ok(());
                        }else if key.code == KeyCode::Up {
                            if self.position > 0 {
                                self.position -= 1;
                            }else {
                                self.position = COMMAND_LIST.len() - 1;
                            }
                        }else if key.code == KeyCode::Down {
                            self.position += 1;
                            if self.position >= COMMAND_LIST.len() {
                                self.position = 0;
                            }
                        } else if key.code == KeyCode::Left {
                            self.left_window();
                        }else if key.code == KeyCode::Right {
                            self.right_window();
                        } else if key.code == KeyCode::Enter {
                            self.execute_command();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn left_window(&mut self) {
        let nw = match self.window {
            SelectedWindow::Commands => SelectedWindow::AppLogs,
            SelectedWindow::AgentLogs => SelectedWindow::Commands,
            SelectedWindow::AppLogs => SelectedWindow::AgentLogs,
        };
        self.window = nw;
    }
    fn right_window(&mut self) {
        let nw = match self.window {
            SelectedWindow::Commands => SelectedWindow::AgentLogs,
            SelectedWindow::AgentLogs => SelectedWindow::AppLogs,
            SelectedWindow::AppLogs => SelectedWindow::Commands,
        };
        self.window = nw;
    }

    fn handle_events_agent_logs(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    if key.code == KeyCode::Up {
                        if self.agent_logs_i > 0 {
                            self.agent_logs_i -= 1;
                        }else {
                            self.agent_logs_i = self.agent_logs.len().saturating_sub(1);
                        }
                    }else if key.code == KeyCode::Down {
                        self.agent_logs_i = self.agent_logs_i.saturating_add(1);
                        if self.agent_logs_i >= self.agent_logs.len() {
                            self.agent_logs_i = 0;
                        }
                    } else if key.code == KeyCode::Left {
                        self.left_window();
                    }else if key.code == KeyCode::Right {
                        self.right_window();
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_events_app_logs(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    if key.code == KeyCode::Up {
                        if self.app_logs_i > 0 {
                            self.app_logs_i -= 1;
                        }else {
                            self.app_logs_i = self.app_logs.len().saturating_sub(1);
                        }
                    }else if key.code == KeyCode::Down {
                        self.app_logs_i += 1;
                        if self.app_logs_i >= self.app_logs.len() {
                            self.app_logs_i = 0;
                        }
                    } else if key.code == KeyCode::Left {
                        self.left_window();
                    }else if key.code == KeyCode::Right {
                        self.right_window();
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match self.window {
            SelectedWindow::Commands => self.handle_events_commands()?,
            SelectedWindow::AgentLogs => self.handle_events_agent_logs()?,
            SelectedWindow::AppLogs => self.handle_events_app_logs()?,
        };
        self.receive_data();
        Ok(())
    }
    fn get_input_text(&mut self) -> String {
        let mut ret = String::with_capacity(32);
        std::mem::swap(&mut self.input_text, &mut ret);
        ret
    }
    fn check_executed_command(&mut self) {
        if self.input {
            return
        }
        let txt = self.get_input_text();
        let mut completed = false;
        let mut to_show = Vec::with_capacity(8);
        match &mut self.command_state {
            CommandState::None => return,
            CommandState::CreateScenario(ls) => {
                if ls.id.is_none() {
                    to_show.push(txt.clone());
                    ls.id = Some(txt);
                    to_show.push("Base scenario ID?".into());
                    self.input = true;
                } else if ls.base_id.is_none() {
                    to_show.push(txt.clone());
                    ls.base_id = Some(txt);
                }
                if ls.id.is_some() && ls.base_id.is_some() {
                    completed = true
                }
            },
            CommandState::StartScenario(ss) => {
                if ss.id.is_none() {
                    to_show.push(txt.clone());
                    ss.id = Some(txt);
                }
                completed = true;
            },
            CommandState::AgentLogs(ss) => {
                if ss.name.is_none() {
                    to_show.push(txt.clone());
                    ss.name = Some(txt);
                }
                completed = true;
            },
            CommandState::AppLogs(ss) => {
                if ss.name.is_none() {
                    to_show.push(txt.clone());
                    ss.name = Some(txt);
                }
                completed = true;
            },
            CommandState::Backup(b) => {
                if b.name.is_none() {
                    to_show.push(txt.clone());
                    b.name = Some(txt);
                }
                completed = true;
            }
        }
        for txt in to_show {
            self.show_text(txt);
        }
        if completed {
            let mut cs = CommandState::None;
            std::mem::swap(&mut self.command_state, &mut cs);
            match cs {
                CommandState::None => (),
                CommandState::CreateScenario(ls) => {
                    self.create_sceanario(ls.id.unwrap(), ls.base_id.unwrap())
                },
                CommandState::StartScenario(ss) => {
                    self.start_sceanario(ss.id.unwrap())
                },
                CommandState::Backup(v) => {
                    self.do_backup(v.name.unwrap());
                },
                CommandState::AppLogs(v) => {
                    self.start_app_logs(v.name.unwrap());
                },
                CommandState::AgentLogs(v) => {
                    self.start_agent_logs(v.name.unwrap());
                }
            }

        }
    }

    fn execute_command(&mut self) {
        let cmd = match COMMAND_LIST.get(self.position) {
            Some(v) => v[0],
            None => return
        };
        match cmd {
            "List Agents" => {
                self.list_agents()
            },
            "All Agent logs" => {
                self.start_all_agent_logs();
            },
            "Agent logs" => {
                self.init_agent_logs();
            },
            "Stop agent logs" => {
                self.stop_agent_logs();
            },
            "All App logs" => {
                self.start_all_app_logs();
            },
            "App logs" => {
                self.init_app_logs();
            },
            "Stop app logs" => {
                self.stop_app_logs();
            },
            "Create scenario" => {
                self.init_creating_scenario();
            },
            "Start scenario" => {
                self.init_start_scenario();
            },
            "Stop scenario" => {
                self.stop_sceanario();
            },
            "Edit scenario" => {
                
            },
            "List scenarios" => {
                self.list_scenarios();
            },
            "List test scenarios" => {
                self.list_test_scenarios();
            },
            "Backup" => {
                self.init_backup();
            },
            "Report" => {

            },
            "Exit" => {
                self.exit = true;
            },
            _ => {}
        }
    }
    fn show_text(&mut self, text : String) {
        self.output.push_front(text);
        if self.output.len() > 1024 {
            for _ in 0..(self.output.len() - 1024) {
                self.output.pop_back();
            }
        }
    }
    fn show_agent_log(&mut self, text : [String; 2]) {
        self.agent_logs.push_front(text);
        if self.agent_logs.len() > 1024 {
            for _ in 0..(self.agent_logs.len() - 1024) {
                self.agent_logs.pop_back();
            }
        }
    }
    fn show_app_log(&mut self, text : [String; 2]) {
        self.app_logs.push_front(text);
        if self.app_logs.len() > 1024 {
            for _ in 0..(self.app_logs.len() - 1024) {
                self.app_logs.pop_back();
            }
        }
    }

    fn receive_data(&mut self) {
        loop {
            let msg = match process_message(&mut self.client) {
                Ok(v) => v,
                Err(_) => break,
            };
            match msg {
                UserActionResponse::Logs(v) => {
                    self.show_agent_log([v.agent, v.msg]);
                    return;
                },
                UserActionResponse::AgentCompletion(v) => {
                    self.current_agent_completion = v;
                },
                UserActionResponse::AppLogs(v) => {
                    self.show_app_log([v.file, v.msg]);
                    return
                },
                UserActionResponse::EnumerateAgents(v) => {
                    for s in v {
                        self.show_text(format!("- {}", s));
                    }
                    self.show_text("Agents:".into());
                    return
                }
                UserActionResponse::BackupDB(v) => {
                    self.show_text(format!("Server Backup status {}", result_to_string(v)));
                },
                UserActionResponse::StartScenario(v) => {
                    self.show_text(format!("Start scenario {}", result_to_string(v)));
                },
                UserActionResponse::StopScenario(v) => {
                    self.show_text(format!("Stop scenario {}", result_to_string(v)));
                },
                UserActionResponse::CreateScenario(v) => {
                    self.show_text(format!("Create scenario {}", result_to_string(v)));
                },
                UserActionResponse::EnumerateScenarios(v) => {
                    for s in v {
                        self.show_text(format!("- {}", s));
                    }
                    self.show_text("Scenarios:".into());
                    return
                },
                UserActionResponse::EnumerateTestingScenarios(v) => {
                    for s in v {
                        self.show_text(format!("- {}", s));
                    }
                    self.show_text("Testing Scenarios:".into());
                    return
                },
                UserActionResponse::Report(rprt) => {
                    let file_name = format!("report-{}.md", rprt.id);
                    let mut file = std::fs::File::create(&file_name).unwrap();
                    file.write_all(rprt.report.as_bytes()).unwrap();
                    self.show_text(format!("Generated report {}", file_name));
                    return
                },
                UserActionResponse::None => return,
            };

        }
    }

    fn start_all_agent_logs(&mut self) {
        self.current_agent = Some("*".into());
        self.client
            .send(user_action_to_message(&UserAction::AgentLogsAll))
            .unwrap();
    }
    fn start_agent_logs(&mut self, agent: String) {
        self.current_agent = Some(agent.clone());
        self.client
            .send(user_action_to_message(&UserAction::AgentLogs(LogSubscription{
                agent
            })))
            .unwrap();
    }
    fn stop_agent_logs(&mut self) {
        self.current_agent = None;
        self.client
            .send(user_action_to_message(&UserAction::StopAgentLogs))
            .unwrap();
    }
    fn start_all_app_logs(&mut self) {
        self.current_app = Some("*".into());
        self.client
            .send(user_action_to_message(&UserAction::AppLogsAll))
            .unwrap();
    }
    fn start_app_logs(&mut self, agent : String) {
        self.current_app = Some(agent.clone());
        self.client
            .send(user_action_to_message(&UserAction::AppLogs(LogSubscription {
                agent
            })))
            .unwrap();
    }
    fn stop_app_logs(&mut self) {
        self.current_app = None;
        self.client
            .send(user_action_to_message(&UserAction::StopAppLogs))
            .unwrap();
    }
    fn list_scenarios(&mut self) {
        self.client
            .send(user_action_to_message(&UserAction::EnumerateScenarios))
            .unwrap();
    }
    fn list_agents(&mut self) {
        self.client
            .send(user_action_to_message(&UserAction::EnumerateAgents))
            .unwrap();
    }
    fn list_test_scenarios(&mut self) {
        self.client
            .send(user_action_to_message(&UserAction::EnumerateTestingScenarios))
            .unwrap();
    }

    fn init_creating_scenario(&mut self) {
        self.input = true;
        self.input_text.clear();
        self.command_state = CommandState::CreateScenario(CreateScenarioState::default());
        self.show_text("Scenario ID?".into());
    }
    fn init_agent_logs(&mut self) {
        self.input = true;
        self.input_text.clear();
        self.command_state = CommandState::AgentLogs(SelectAgentState::default());
        self.show_text("Agent Name?".into());
    }
    fn init_app_logs(&mut self) {
        self.input = true;
        self.input_text.clear();
        self.command_state = CommandState::AppLogs(SelectAgentState::default());
        self.show_text("Agent Name?".into());
    }

    fn create_sceanario(&mut self, id : String, base_id : String) {
        self.client
            .send(user_action_to_message(&UserAction::CreateScenario(CreateScenario {
                base_id,
                id
            })))
            .unwrap();
    }
    fn init_start_scenario(&mut self) {
        self.input = true;
        self.input_text.clear();
        self.command_state = CommandState::StartScenario(SelectScenarioState::default());
        self.show_text("Scenario ID?".into());
    }
    fn start_sceanario(&mut self, id : String) {
        self.client
            .send(user_action_to_message(&UserAction::StartScenario(id)))
            .unwrap();
    }
    fn stop_sceanario(&mut self) {
        self.client
            .send(user_action_to_message(&UserAction::StopScenario))
            .unwrap();
    }
    fn init_backup(&mut self) {
        self.input = true;
        self.input_text.clear();
        self.command_state = CommandState::Backup(BackupName::default());
        self.show_text("Backup name?".into());
    }
    fn do_backup(&mut self, name : String) {
        self.client
            .send(user_action_to_message(&UserAction::BackupDB(name)))
            .unwrap();
    }
}


fn border_style() -> Style {
    Style::new().light_cyan()
}

struct ConsoleIterator<'a, I> where I: Iterator<Item = Row<'a>> {
    iter : I,
    element : Option<Row<'a>>
}
impl<'a, I> Iterator for ConsoleIterator<'a, I> where I: Iterator<Item = Row<'a>>{
    type Item = Row<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.element.take() { return Some(v) }
        self.iter.next()
    }
}
impl<'a, I> ConsoleIterator<'a, I> where I: Iterator<Item = Row<'a>> {
    pub fn new(iter : I, element : Option<Row<'a>>) -> Self {
        Self {
            iter,
            element
        }
    }
}

fn process_message(client: &mut WsClient) -> Result<UserActionResponse, tungstenite::Error> {
    let msg = match client.read() {
        Ok(v) => v,
        Err(e) => match e {
            tungstenite::Error::Io(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => return Ok(UserActionResponse::None),
                _ => return Err(tungstenite::Error::Io(e)),
            },
            _ => return Err(e),
        },
    };
    let res: UserActionResponse = match msg {
        tungstenite::Message::Text(v) => serde_json::from_str(&v).unwrap_or_default(),
        tungstenite::Message::Binary(v) => serde_json::from_slice(&v).unwrap_or_default(),
        _ => return Ok(UserActionResponse::None),
    };
    Ok(res)
}


fn user_action_to_message(action: &UserAction) -> tungstenite::Message {
    tungstenite::Message::Binary(serde_json::to_vec(action).unwrap())
}

fn result_to_string(r : ChaosResult<()>) -> String {
    match r {
        Ok(_) => "OK".into(),
        Err(e) => format!("ERR: {}", e),
    }
}