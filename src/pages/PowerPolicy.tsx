import { useEffect, useState } from "react";
import { aliveApi, type ModeName, type PowerInfo } from "../api/alive";
import { lidActionLabel } from "../logic/status";
import { useAliveStore } from "../stores/aliveStore";

export function PowerPolicyPage() {
  const settings = useAliveStore((s) => s.settings);
  const setError = useAliveStore((s) => s.setError);
  const [power, setPower] = useState<PowerInfo | null>(null);
  const [mode, setMode] = useState<ModeName>("Strict");
  const [pauseHeavy, setPauseHeavy] = useState(30);
  const [critical, setCritical] = useState(15);

  useEffect(() => {
    if (settings) {
      setMode(settings.mode);
      setPauseHeavy(settings.pauseHeavyPercent);
      setCritical(settings.criticalPercent);
    }
    aliveApi
      .getPowerInfo()
      .then(setPower)
      .catch(() => setPower(null));
  }, [settings]);

  async function save() {
    if (!settings) {
      return;
    }
    if (critical >= pauseHeavy) {
      setError("The critical battery level must be below the pause heavy level.");
      return;
    }
    try {
      await aliveApi.updateSettings({
        ...settings,
        mode,
        pauseHeavyPercent: pauseHeavy,
        criticalPercent: critical,
      });
    } catch (e) {
      setError(e instanceof Error ? e.message : "Saving the power policy failed.");
    }
  }

  return (
    <section aria-labelledby="policy-heading">
      <h1 id="policy-heading">Power Policy</h1>
      <div className="card">
        <label>
          Mode
          <select value={mode} onChange={(e) => setMode(e.target.value as ModeName)}>
            <option value="Strict">Strict: keep everything alive until the critical level</option>
            <option value="Smart">Smart: pause heavy agents on battery by the levels below</option>
            <option value="PluggedInOnly">Plugged in only: Alive waits while on battery</option>
          </select>
        </label>
        <label>
          Pause heavy agents below battery percent: {pauseHeavy}
          <input
            type="range"
            min={16}
            max={90}
            value={pauseHeavy}
            onChange={(e) => setPauseHeavy(Number(e.target.value))}
          />
          <input
            type="number"
            min={16}
            max={90}
            value={pauseHeavy}
            onChange={(e) => setPauseHeavy(Number(e.target.value))}
            aria-label="Pause heavy agents below battery percent, number"
          />
        </label>
        <label>
          Turn Alive OFF below battery percent: {critical}
          <input
            type="range"
            min={5}
            max={50}
            value={critical}
            onChange={(e) => setCritical(Number(e.target.value))}
          />
          <input
            type="number"
            min={5}
            max={50}
            value={critical}
            onChange={(e) => setCritical(Number(e.target.value))}
            aria-label="Turn Alive off below battery percent, number"
          />
        </label>
        <button type="button" onClick={() => void save()}>
          Save power policy
        </button>
      </div>

      <div className="card" aria-labelledby="preview-heading">
        <h2 id="preview-heading">What Alive will change</h2>
        {power ? (
          <dl className="facts">
            <div>
              <dt>Lid close, plugged in</dt>
              <dd>
                Now {lidActionLabel(power.currentLidAc)}. Original {lidActionLabel(power.originalLidAc)}.
                Alive applies Do nothing.
              </dd>
            </div>
            <div>
              <dt>Lid close, on battery</dt>
              <dd>
                Now {lidActionLabel(power.currentLidDc)}. Original {lidActionLabel(power.originalLidDc)}.
                Alive applies Do nothing.
              </dd>
            </div>
            <div>
              <dt>Sleep after, plugged in</dt>
              <dd>
                Original {power.originalStandbyAcSeconds} seconds. Alive applies Never.
              </dd>
            </div>
          </dl>
        ) : (
          <p>Power details appear when the desktop app is connected.</p>
        )}
        <p>
          The restore guarantee: before Alive changes anything it saves your current values. When
          Alive turns OFF, and even if Alive crashes, those exact values are put back.
        </p>
      </div>
    </section>
  );
}
