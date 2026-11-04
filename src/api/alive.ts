// The one typed API module for Alive (spec section 8.2). Every Tauri invoke
// call and every event subscription in the frontend goes through this file.
// Command names match the IPC contract in spec section 7 exactly.

import type { AliveStateName } from "../logic/status";

export interface AliveStatus {
  state: AliveStateName;
  mode: ModeName;
  onAc: boolean;
  batteryPercent: number | null;
  batterySaver: boolean;
  modernStandby: boolean;
  runningAgents: number;
  workingAgents: number;
  onSinceSeconds: number | null;
}

export type ModeName = "Strict" | "Smart" | "PluggedInOnly";

export interface PowerInfo {
  onAc: boolean;
  batteryPercent: number | null;
  batterySaver: boolean;
  modernStandby: boolean;
  originalLidAc: number;
  originalLidDc: number;
  currentLidAc: number;
  currentLidDc: number;
  originalStandbyAcSeconds: number;
  appliedStandbyAcSeconds: number;
}

export interface AgentView {
  id: string;
  name: string;
  command: string;
  args: string[];
  workingDir: string;
  kind: "Cloud" | "Local";
  heavy: boolean;
  enabled: boolean;
  autostart: boolean;
  state: "Stopped" | "Running" | "Working" | "Failed" | "Paused";
  restartCount: number;
}

export interface SettingsView {
  mode: ModeName;
  pauseHeavyPercent: number;
  criticalPercent: number;
  hysteresisPercent: number;
  startWithWindows: boolean;
  turnOnAtStart: boolean;
  startMinimizedToTray: boolean;
  notifications: boolean;
}

export interface SelfCheckItem {
  name: string;
  passed: boolean;
  advice: string;
}

export interface LogPage {
  lines: string[];
  nextCursor: number;
}

type InvokeFn = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

let cachedInvoke: InvokeFn | null = null;

async function getInvoke(): Promise<InvokeFn | null> {
  if (cachedInvoke) {
    return cachedInvoke;
  }
  try {
    const mod = await import("@tauri-apps/api/core");
    cachedInvoke = mod.invoke as InvokeFn;
    return cachedInvoke;
  } catch {
    return null;
  }
}

/// True when the frontend is running inside the Tauri shell. In a plain
/// browser the UI still renders with local preview state, and every action
/// reports that it needs the desktop app instead of pretending to work.
export async function isDesktopApp(): Promise<boolean> {
  return (await getInvoke()) !== null;
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const invoke = await getInvoke();
  if (!invoke) {
    throw new Error("Alive desktop app is not connected. Open Alive on Windows to use this action.");
  }
  return invoke<T>(command, args);
}

export const aliveApi = {
  getStatus: () => call<AliveStatus>("get_status"),
  setAlive: (on: boolean) => call<AliveStatus>("set_alive", { on }),
  getSettings: () => call<SettingsView>("get_settings"),
  updateSettings: (settings: SettingsView) => call<void>("update_settings", { settings }),
  listAgents: () => call<AgentView[]>("list_agents"),
  upsertAgent: (agent: AgentView) => call<void>("upsert_agent", { agent }),
  removeAgent: (id: string) => call<void>("remove_agent", { id }),
  startAgent: (id: string) => call<void>("start_agent", { id }),
  stopAgent: (id: string) => call<void>("stop_agent", { id }),
  restartAgent: (id: string) => call<void>("restart_agent", { id }),
  getLogs: (agentId: string, cursor: number, limit: number) =>
    call<LogPage>("get_logs", { agentId, cursor, limit }),
  clearLogs: (agentId: string) => call<void>("clear_logs", { agentId }),
  getPowerInfo: () => call<PowerInfo>("get_power_info"),
  runSelfCheck: () => call<SelfCheckItem[]>("run_self_check"),
  exportDiagnostics: () => call<string>("export_diagnostics"),
};
