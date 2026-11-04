import { useState } from "react";
import { aliveApi } from "../api/alive";

export function ActivityPage() {
  const [lines, setLines] = useState<string[]>([]);
  const [message, setMessage] = useState<string | null>(null);

  async function load() {
    try {
      const page = await aliveApi.getLogs("alive", 0, 200);
      setLines(page.lines);
      setMessage(null);
    } catch (e) {
      setMessage(e instanceof Error ? e.message : "Loading activity failed.");
    }
  }

  return (
    <section aria-labelledby="activity-heading">
      <h1 id="activity-heading">Activity</h1>
      <p>State changes, power events and agent events, newest last.</p>
      <button type="button" onClick={() => void load()}>
        Load recent activity
      </button>
      {message ? (
        <p role="alert" className="error-text">
          {message}
        </p>
      ) : null}
      <ol className="log-list" aria-live="polite">
        {lines.map((line, index) => (
          <li key={`${index}-${line.slice(0, 24)}`}>
            <code>{line}</code>
          </li>
        ))}
      </ol>
    </section>
  );
}
