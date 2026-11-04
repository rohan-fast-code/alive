import { create } from "zustand";
import { aliveApi, type AgentView, type AliveStatus, type SettingsView } from "../api/alive";
import type { AliveStateName } from "../logic/status";

interface AliveStore {
  status: AliveStatus | null;
  agents: AgentView[];
  settings: SettingsView | null;
  error: string | null;
  busy: boolean;
  refresh: () => Promise<void>;
  toggle: () => Promise<void>;
  setError: (message: string | null) => void;
}

const previewStatus: AliveStatus = {
  state: "Off" as AliveStateName,
  mode: "Strict",
  onAc: true,
  batteryPercent: null,
  batterySaver: false,
  modernStandby: false,
  runningAgents: 0,
  workingAgents: 0,
  onSinceSeconds: null,
};

export const useAliveStore = create<AliveStore>((set, get) => ({
  status: previewStatus,
  agents: [],
  settings: null,
  error: null,
  busy: false,

  setError: (message) => set({ error: message }),

  refresh: async () => {
    try {
      const [status, agents, settings] = await Promise.all([
        aliveApi.getStatus(),
        aliveApi.listAgents(),
        aliveApi.getSettings(),
      ]);
      set({ status, agents, settings, error: null });
    } catch (e) {
      // In a plain browser preview there is no backend. Keep the preview
      // state on screen and stay silent; actions still report the truth.
      set({ status: get().status ?? previewStatus });
      void e;
    }
  },

  toggle: async () => {
    const current = get().status;
    if (!current || get().busy) {
      return;
    }
    const turningOn = current.state === "Off";
    set({ busy: true, error: null });
    try {
      const next = await aliveApi.setAlive(turningOn);
      set({ status: next });
    } catch (e) {
      set({ error: e instanceof Error ? e.message : "The toggle failed." });
    } finally {
      set({ busy: false });
    }
  },
}));
