import { useState } from "react";
import { aliveApi, type AgentView } from "../api/alive";
import { useAliveStore } from "../stores/aliveStore";

const emptyAgent: AgentView = {
  id: "",
  name: "",
  command: "",
  args: [],
  workingDir: "",
  kind: "Cloud",
  heavy: false,
  enabled: true,
  autostart: false,
  state: "Stopped",
  restartCount: 0,
};

export function AgentsPage() {
  const agents = useAliveStore((s) => s.agents);
  const refresh = useAliveStore((s) => s.refresh);
  const setError = useAliveStore((s) => s.setError);
  const [draft, setDraft] = useState<AgentView>(emptyAgent);
  const [formError, setFormError] = useState<string | null>(null);

  async function save() {
    // Live validation mirrors the Rust config validation (TC12 and TC27).
    if (draft.name.trim() === "") {
      setFormError("Give the agent a name.");
      return;
    }
    if (draft.command.trim() === "") {
      setFormError("Enter the program to run. Alive runs it directly, with no shell.");
      return;
    }
    setFormError(null);
    try {
      await aliveApi.upsertAgent({ ...draft, id: draft.id || draft.name });
      setDraft(emptyAgent);
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Saving the agent failed.");
    }
  }

  return (
    <section aria-labelledby="agents-heading">
      <h1 id="agents-heading">Agents</h1>
      <p className="warning" role="note">
        Warning: an agent definition runs a program on your machine. Only add programs you trust,
        and check the full command before you save it.
      </p>
      {agents.length === 0 ? <p>No agents yet. Add your first agent below.</p> : null}
      <ul className="agent-list">
        {agents.map((agent) => (
          <li key={agent.id} className="card">
            <strong>{agent.name}</strong> <span className="badge">{agent.state}</span>
            <div className="muted">
              {agent.command} {agent.args.join(" ")}
            </div>
            <div className="row">
              <button type="button" onClick={() => void aliveApi.startAgent(agent.id).then(refresh)}>
                Start
              </button>
              <button type="button" onClick={() => void aliveApi.stopAgent(agent.id).then(refresh)}>
                Stop
              </button>
              <button type="button" onClick={() => void aliveApi.restartAgent(agent.id).then(refresh)}>
                Restart
              </button>
              <button
                type="button"
                onClick={() => void aliveApi.removeAgent(agent.id).then(refresh)}
              >
                Remove
              </button>
            </div>
          </li>
        ))}
      </ul>
      <form
        className="card"
        onSubmit={(e) => {
          e.preventDefault();
          void save();
        }}
      >
        <h2>Add or edit an agent</h2>
        <label>
          Name
          <input
            value={draft.name}
            onChange={(e) => setDraft({ ...draft, name: e.target.value })}
            aria-invalid={formError !== null}
          />
        </label>
        <label>
          Program
          <input
            value={draft.command}
            onChange={(e) => setDraft({ ...draft, command: e.target.value })}
            placeholder="Full path to the program"
          />
        </label>
        <label>
          Kind
          <select
            value={draft.kind}
            onChange={(e) => setDraft({ ...draft, kind: e.target.value as AgentView["kind"] })}
          >
            <option value="Cloud">Cloud agent</option>
            <option value="Local">Local agent</option>
          </select>
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={draft.heavy}
            onChange={(e) => setDraft({ ...draft, heavy: e.target.checked })}
          />
          Heavy work (paused first on battery in Smart mode)
        </label>
        {formError ? (
          <p role="alert" className="error-text">
            {formError}
          </p>
        ) : null}
        <button type="submit">Save agent</button>
      </form>
    </section>
  );
}
