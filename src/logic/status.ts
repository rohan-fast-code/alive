// Pure display logic for Alive status values. Kept free of React and Tauri
// imports so it can be unit tested with Vitest on any machine.

export type AliveStateName =
  | "Off"
  | "Arming"
  | "OnActive"
  | "OnIdle"
  | "OnThrottled"
  | "Restoring"
  | "Faulted";

export function isOnState(state: AliveStateName): boolean {
  return state === "OnActive" || state === "OnIdle" || state === "OnThrottled";
}

export function stateLabel(state: AliveStateName): string {
  switch (state) {
    case "Off":
      return "Alive is OFF. The laptop behaves normally.";
    case "Arming":
      return "Alive is turning ON.";
    case "OnActive":
      return "Alive is ON. Agents are working.";
    case "OnIdle":
      return "Alive is ON. No agent is working right now.";
    case "OnThrottled":
      return "Alive is ON. Heavy agents are paused or throttled by the battery policy.";
    case "Restoring":
      return "Alive is turning OFF and restoring your original power settings.";
    case "Faulted":
      return "Alive hit a problem while restoring. Your original settings may not be fully restored yet.";
  }
}

export function lidActionLabel(value: number): string {
  switch (value) {
    case 0:
      return "Do nothing";
    case 1:
      return "Sleep";
    case 2:
      return "Hibernate";
    case 3:
      return "Shut down";
    default:
      return "Unknown";
  }
}

export function formatBattery(percent: number | null, onAc: boolean): string {
  if (onAc) {
    return percent === null ? "Plugged in" : `Plugged in, ${percent} percent`;
  }
  return percent === null ? "On battery" : `On battery, ${percent} percent`;
}

export function trayTooltip(state: AliveStateName, onAc: boolean, percent: number | null, runningAgents: number): string {
  const power = formatBattery(percent, onAc);
  const agents = runningAgents === 1 ? "1 agent running" : `${runningAgents} agents running`;
  return `Alive: ${state}. ${power}. ${agents}.`;
}

/// Count state transitions in a decision list. The hysteresis tests use the
/// same idea on the TypeScript side for the battery sparkline smoothing.
export function countTransitions(values: readonly string[]): number {
  let count = 0;
  for (let i = 1; i < values.length; i += 1) {
    if (values[i] !== values[i - 1]) {
      count += 1;
    }
  }
  return count;
}
