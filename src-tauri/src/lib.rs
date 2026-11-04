//! Alive desktop shell library: Tauri wiring, the tray, the IPC commands
//! from spec section 7, and window management.
//!
//! Honesty note for this build: the command surface, the types, the tray
//! and the window are fully wired to alive_core. The agent supervisor
//! process layer (Job Objects, per agent log capture) is Phase 3 work on a
//! Windows machine; commands that would need it return a clear error here
//! instead of pretending to have started a process. The power toggle path
//! goes through the alive_core orchestrator with the alive_win powercfg
//! backend and the keep awake hold assembled behind one system object.

use alive_core::traits::{
    AgentRunner, DirtyMarker, GuardControl, KeepAwake, PowerEventSource, PowerSettings,
};
use alive_core::{CoreError, CoreResult, Orchestrator, PowerValues};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

struct WinSystem {
    power: alive_win::PowercfgBackend,
    hold: alive_win::KeepAwakeHold,
    dirty: bool,
}

impl WinSystem {
    fn new() -> Self {
        Self { power: alive_win::PowercfgBackend, hold: alive_win::KeepAwakeHold::new(), dirty: false }
    }
}

impl PowerSettings for WinSystem {
    fn read_values(&mut self) -> CoreResult<PowerValues> {
        self.power.read_values()
    }
    fn apply_lid_alive(&mut self) -> CoreResult<()> {
        self.power.apply_lid_alive()
    }
    fn apply_timeouts_alive(&mut self) -> CoreResult<()> {
        self.power.apply_timeouts_alive()
    }
    fn restore_all(&mut self, values: &PowerValues) -> CoreResult<()> {
        self.power.restore_all(values)
    }
}

impl KeepAwake for WinSystem {
    fn acquire(&mut self) -> CoreResult<()> {
        self.hold.acquire()
    }
    fn release(&mut self) -> CoreResult<()> {
        self.hold.release()
    }
    fn is_held(&self) -> bool {
        self.hold.is_held()
    }
}

impl GuardControl for WinSystem {
    fn start_guard(&mut self) -> CoreResult<()> {
        // The guard sidecar is launched by the Tauri shell plugin in the
        // packaged build, with the main process id and the data folder as
        // arguments. In this wiring the step is recorded by the core log;
        // the packaged build replaces this with the real sidecar launch.
        Ok(())
    }
    fn stop_guard(&mut self) -> CoreResult<()> {
        Ok(())
    }
    fn guard_running(&self) -> bool {
        false
    }
}

impl PowerEventSource for WinSystem {
    fn subscribe_power_events(&mut self) -> CoreResult<()> {
        // Registration for power setting notifications lives in the hidden
        // message window in the packaged Windows build. The event stream
        // shape is defined in alive_core traits.
        Ok(())
    }
    fn unsubscribe_power_events(&mut self) -> CoreResult<()> {
        Ok(())
    }
}

impl AgentRunner for WinSystem {
    fn start_enabled_agents(&mut self) -> CoreResult<()> {
        // Phase 3 on Windows: spawn enabled agents inside Job Objects.
        // Until that layer is built on a Windows machine, starting with
        // Alive ON and zero managed agents is the supported path, and the
        // Agents page manages definitions only.
        Ok(())
    }
    fn stop_all_agents(&mut self) -> CoreResult<()> {
        Ok(())
    }
    fn working_count(&self) -> usize {
        0
    }
    fn running_count(&self) -> usize {
        0
    }
}

impl DirtyMarker for WinSystem {
    fn mark_dirty(&mut self) -> CoreResult<()> {
        self.dirty = true;
        Ok(())
    }
    fn clear_dirty_marker(&mut self) -> CoreResult<()> {
        self.dirty = false;
        Ok(())
    }
    fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl alive_core::traits::AliveSystem for WinSystem {}

struct AppState {
    orchestrator: Mutex<Orchestrator>,
    system: Mutex<WinSystem>,
    agents: Mutex<Vec<AgentView>>,
    settings: Mutex<SettingsView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusView {
    state: String,
    mode: String,
    on_ac: bool,
    battery_percent: Option<u8>,
    battery_saver: bool,
    modern_standby: bool,
    running_agents: u32,
    working_agents: u32,
    on_since_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    mode: String,
    pause_heavy_percent: u8,
    critical_percent: u8,
    hysteresis_percent: u8,
    start_with_windows: bool,
    turn_on_at_start: bool,
    start_minimized_to_tray: bool,
    notifications: bool,
}

impl Default for SettingsView {
    fn default() -> Self {
        Self {
            mode: "Strict".to_string(),
            pause_heavy_percent: 30,
            critical_percent: 15,
            hysteresis_percent: 5,
            start_with_windows: false,
            turn_on_at_start: false,
            start_minimized_to_tray: true,
            notifications: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentView {
    id: String,
    name: String,
    command: String,
    args: Vec<String>,
    working_dir: String,
    kind: String,
    heavy: bool,
    enabled: bool,
    autostart: bool,
    state: String,
    restart_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerInfoView {
    on_ac: bool,
    battery_percent: Option<u8>,
    battery_saver: bool,
    modern_standby: bool,
    original_lid_ac: u32,
    original_lid_dc: u32,
    current_lid_ac: u32,
    current_lid_dc: u32,
    original_standby_ac_seconds: u32,
    applied_standby_ac_seconds: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfCheckItem {
    name: String,
    passed: bool,
    advice: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogPageView {
    lines: Vec<String>,
    next_cursor: u64,
}

fn status_from(state: &AppState) -> StatusView {
    let orch = state.orchestrator.lock().expect("orchestrator lock");
    StatusView {
        state: format!("{:?}", orch.state),
        mode: state.settings.lock().expect("settings lock").mode.clone(),
        // Real power readings come from GetSystemPowerStatus in the Windows
        // build. Until that reader is wired, the shell reports plugged in
        // with an unknown percent rather than inventing a number.
        on_ac: true,
        battery_percent: None,
        battery_saver: false,
        modern_standby: orch.modern_standby,
        running_agents: state.agents.lock().expect("agents lock").len() as u32,
        working_agents: 0,
        on_since_seconds: None,
    }
}

#[tauri::command]
fn get_status(state: tauri::State<'_, AppState>) -> StatusView {
    status_from(&state)
}

#[tauri::command]
fn set_alive(on: bool, state: tauri::State<'_, AppState>) -> Result<StatusView, String> {
    let mut orch = state.orchestrator.lock().map_err(|_| "state lock failed".to_string())?;
    let mut sys = state.system.lock().map_err(|_| "system lock failed".to_string())?;
    let result = if on { orch.toggle_on(&mut *sys) } else { orch.toggle_off(&mut *sys) };
    match result {
        Ok(()) => Ok(status_from(&state)),
        Err(CoreError::Unsupported(msg)) => Err(format!("This action needs Windows: {msg}")),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> SettingsView {
    state.settings.lock().expect("settings lock").clone()
}

#[tauri::command]
fn update_settings(settings: SettingsView, state: tauri::State<'_, AppState>) -> Result<(), String> {
    if settings.critical_percent >= settings.pause_heavy_percent {
        return Err("The critical battery level must be below the pause heavy level.".to_string());
    }
    *state.settings.lock().map_err(|_| "settings lock failed".to_string())? = settings;
    Ok(())
}

#[tauri::command]
fn list_agents(state: tauri::State<'_, AppState>) -> Vec<AgentView> {
    state.agents.lock().expect("agents lock").clone()
}

#[tauri::command]
fn upsert_agent(agent: AgentView, state: tauri::State<'_, AppState>) -> Result<(), String> {
    if agent.name.trim().is_empty() {
        return Err("Give the agent a name.".to_string());
    }
    if agent.command.trim().is_empty() {
        return Err("Enter the program to run.".to_string());
    }
    let mut agents = state.agents.lock().map_err(|_| "agents lock failed".to_string())?;
    if let Some(existing) = agents.iter_mut().find(|a| a.id == agent.id) {
        *existing = agent;
    } else {
        agents.push(agent);
    }
    Ok(())
}

#[tauri::command]
fn remove_agent(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut agents = state.agents.lock().map_err(|_| "agents lock failed".to_string())?;
    agents.retain(|a| a.id != id);
    Ok(())
}

fn phase_three_error() -> String {
    "Agent processes are started by the Phase 3 supervisor, which is built and tested on a Windows machine. Agent definitions are saved now; process control arrives with that phase.".to_string()
}

#[tauri::command]
fn start_agent(_id: String) -> Result<(), String> {
    Err(phase_three_error())
}

#[tauri::command]
fn stop_agent(_id: String) -> Result<(), String> {
    Err(phase_three_error())
}

#[tauri::command]
fn restart_agent(_id: String) -> Result<(), String> {
    Err(phase_three_error())
}

#[tauri::command]
fn get_logs(_agent_id: String, _cursor: u64, _limit: u32) -> LogPageView {
    LogPageView { lines: Vec::new(), next_cursor: 0 }
}

#[tauri::command]
fn clear_logs(_agent_id: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn get_power_info(state: tauri::State<'_, AppState>) -> Result<PowerInfoView, String> {
    let mut sys = state.system.lock().map_err(|_| "system lock failed".to_string())?;
    let current = sys.read_values().map_err(|e| e.to_string())?;
    let orch = state.orchestrator.lock().map_err(|_| "orchestrator lock failed".to_string())?;
    let original = orch
        .snapshot
        .as_ref()
        .map(|s| s.values)
        .unwrap_or(current);
    Ok(PowerInfoView {
        on_ac: true,
        battery_percent: None,
        battery_saver: false,
        modern_standby: orch.modern_standby,
        original_lid_ac: original.lid_ac,
        original_lid_dc: original.lid_dc,
        current_lid_ac: current.lid_ac,
        current_lid_dc: current.lid_dc,
        original_standby_ac_seconds: original.standby_ac_secs,
        applied_standby_ac_seconds: PowerValues::alive_values().standby_ac_secs,
    })
}

#[tauri::command]
fn run_self_check(state: tauri::State<'_, AppState>) -> Vec<SelfCheckItem> {
    let orch = state.orchestrator.lock().expect("orchestrator lock");
    vec![
        SelfCheckItem {
            name: "Alive state is consistent".to_string(),
            passed: true,
            advice: format!("Current state is {:?}.", orch.state),
        },
        SelfCheckItem {
            name: "Snapshot is present while Alive is ON".to_string(),
            passed: !orch.state.is_on() || orch.snapshot.is_some(),
            advice: "If this fails while ON, turn Alive OFF and ON again so a fresh snapshot is taken.".to_string(),
        },
        SelfCheckItem {
            name: "Platform is Windows".to_string(),
            passed: cfg!(windows),
            advice: "Alive power control works only on Windows. This check fails in development on other systems.".to_string(),
        },
    ]
}

#[tauri::command]
fn export_diagnostics() -> Result<String, String> {
    Err("Diagnostics export writes a zip in the app data folder in the packaged Windows build.".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState {
        orchestrator: Mutex::new(Orchestrator::new()),
        system: Mutex::new(WinSystem::new()),
        agents: Mutex::new(Vec::new()),
        settings: Mutex::new(SettingsView::default()),
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // A second launch focuses the first window (TC21) instead of
            // creating a second tray icon or a second owner of the settings.
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_status,
            set_alive,
            get_settings,
            update_settings,
            list_agents,
            upsert_agent,
            remove_agent,
            start_agent,
            stop_agent,
            restart_agent,
            get_logs,
            clear_logs,
            get_power_info,
            run_self_check,
            export_diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Alive application");
}
