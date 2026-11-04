import { describe, expect, it } from "vitest";
import {
  countTransitions,
  formatBattery,
  isOnState,
  lidActionLabel,
  stateLabel,
  trayTooltip,
} from "./status";

describe("status logic", () => {
  it("knows which states count as ON", () => {
    expect(isOnState("OnActive")).toBe(true);
    expect(isOnState("OnIdle")).toBe(true);
    expect(isOnState("OnThrottled")).toBe(true);
    expect(isOnState("Off")).toBe(false);
    expect(isOnState("Faulted")).toBe(false);
  });

  it("labels every lid action value from the spec", () => {
    expect(lidActionLabel(0)).toBe("Do nothing");
    expect(lidActionLabel(1)).toBe("Sleep");
    expect(lidActionLabel(2)).toBe("Hibernate");
    expect(lidActionLabel(3)).toBe("Shut down");
    expect(lidActionLabel(9)).toBe("Unknown");
  });

  it("formats battery text with full words and no guessing", () => {
    expect(formatBattery(82, false)).toBe("On battery, 82 percent");
    expect(formatBattery(null, true)).toBe("Plugged in");
    expect(formatBattery(null, false)).toBe("On battery");
  });

  it("builds a tray tooltip with state, power and agent count", () => {
    expect(trayTooltip("OnActive", false, 64, 2)).toContain("64 percent");
    expect(trayTooltip("OnActive", false, 64, 2)).toContain("2 agents running");
    expect(trayTooltip("Off", true, null, 1)).toContain("1 agent running");
  });

  it("counts transitions the way the hysteresis check needs", () => {
    expect(countTransitions(["a", "a", "b", "b", "a"])).toBe(2);
    expect(countTransitions([])).toBe(0);
    expect(stateLabel("Restoring")).toContain("restoring");
  });
});
