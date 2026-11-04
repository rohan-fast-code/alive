import * as Switch from "@radix-ui/react-switch";
import { Battery, PlugZap, Power } from "lucide-react";
import { formatBattery, isOnState, stateLabel } from "../logic/status";
import { useAliveStore } from "../stores/aliveStore";

export function DashboardPage() {
  const status = useAliveStore((s) => s.status);
  const busy = useAliveStore((s) => s.busy);
  const toggle = useAliveStore((s) => s.toggle);
  const error = useAliveStore((s) => s.error);

  if (!status) {
    return <p role="status">Loading Alive status.</p>;
  }
  const on = isOnState(status.state);

  return (
    <section aria-labelledby="dashboard-heading">
      <h1 id="dashboard-heading">Dashboard</h1>
      <div className="card flex flex-col gap-3">
        <label className="toggle-row">
          <span>
            <Power aria-hidden size={20} /> Alive
          </span>
          <Switch.Root
            className="switch"
            checked={on}
            disabled={busy}
            onCheckedChange={() => void toggle()}
            aria-label="Toggle Alive on or off"
          >
            <Switch.Thumb className="switch-thumb" />
          </Switch.Root>
        </label>
        <p role="status">{stateLabel(status.state)}</p>
        <dl className="facts">
          <div>
            <dt>Power</dt>
            <dd>
              {status.onAc ? <PlugZap aria-hidden size={16} /> : <Battery aria-hidden size={16} />}{" "}
              {formatBattery(status.batteryPercent, status.onAc)}
            </dd>
          </div>
          <div>
            <dt>Mode</dt>
            <dd>{status.mode}</dd>
          </div>
          <div>
            <dt>Agents running</dt>
            <dd>{status.runningAgents}</dd>
          </div>
          <div>
            <dt>Modern Standby machine</dt>
            <dd>{status.modernStandby ? "Yes" : "No or not detected"}</dd>
          </div>
        </dl>
        {error ? (
          <p role="alert" className="error-text">
            {error}
          </p>
        ) : null}
      </div>
      <p className="note">
        When Alive is ON, closing the lid does not sleep the laptop, and your agents keep running.
        When Alive is OFF, every power setting goes back to what it was before.
      </p>
    </section>
  );
}
