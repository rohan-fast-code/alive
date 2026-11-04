import { aliveApi } from "../api/alive";
import { useAliveStore } from "../stores/aliveStore";

export function SettingsPage() {
  const settings = useAliveStore((s) => s.settings);
  const refresh = useAliveStore((s) => s.refresh);
  const setError = useAliveStore((s) => s.setError);

  if (!settings) {
    return (
      <section aria-labelledby="settings-heading">
        <h1 id="settings-heading">Settings</h1>
        <p>Settings appear when the desktop app is connected.</p>
      </section>
    );
  }

  async function update(patch: Partial<typeof settings>) {
    if (!settings) {
      return;
    }
    try {
      await aliveApi.updateSettings({ ...settings, ...patch });
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Saving settings failed.");
    }
  }

  return (
    <section aria-labelledby="settings-heading">
      <h1 id="settings-heading">Settings</h1>
      <div className="card">
        <label className="check">
          <input
            type="checkbox"
            checked={settings.startWithWindows}
            onChange={(e) => void update({ startWithWindows: e.target.checked })}
          />
          Start Alive with Windows
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={settings.turnOnAtStart}
            onChange={(e) => void update({ turnOnAtStart: e.target.checked })}
          />
          Turn Alive ON when it starts (off by default, for safety)
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={settings.startMinimizedToTray}
            onChange={(e) => void update({ startMinimizedToTray: e.target.checked })}
          />
          Start minimized to the tray
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={settings.notifications}
            onChange={(e) => void update({ notifications: e.target.checked })}
          />
          Show notifications for important events only
        </label>
        <p className="muted">
          Theme follows your Windows theme. Language is English in version 1, and the layout is
          ready for more languages later.
        </p>
      </div>
    </section>
  );
}
