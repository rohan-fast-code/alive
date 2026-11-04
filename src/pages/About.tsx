import { useState } from "react";
import { aliveApi, type SelfCheckItem } from "../api/alive";
import aliveLogo from "../assets/alive-logo.png";

export function AboutPage() {
  const [checks, setChecks] = useState<SelfCheckItem[]>([]);
  const [diagPath, setDiagPath] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  async function runCheck() {
    try {
      setChecks(await aliveApi.runSelfCheck());
      setMessage(null);
    } catch (e) {
      setMessage(e instanceof Error ? e.message : "The self check failed.");
    }
  }

  async function exportDiag() {
    try {
      setDiagPath(await aliveApi.exportDiagnostics());
      setMessage(null);
    } catch (e) {
      setMessage(e instanceof Error ? e.message : "Exporting diagnostics failed.");
    }
  }

  return (
    <section aria-labelledby="about-heading">
      <h1 id="about-heading">About and Diagnostics</h1>
      <div className="card">
        <img src={aliveLogo} alt="Alive logo" width={160} />
        <p>
          <strong>Alive</strong> version 0.1.0. Alive keeps your AI agents running when you close
          the laptop lid, and restores your original power settings when it turns OFF. Alive sends
          nothing over the network by itself and collects no analytics.
        </p>
        <div className="row">
          <button type="button" onClick={() => void runCheck()}>
            Run self check
          </button>
          <button type="button" onClick={() => void exportDiag()}>
            Export diagnostics
          </button>
        </div>
        {message ? (
          <p role="alert" className="error-text">
            {message}
          </p>
        ) : null}
        {diagPath ? <p>Diagnostics were saved to {diagPath}.</p> : null}
        <ul>
          {checks.map((item) => (
            <li key={item.name}>
              <strong>{item.passed ? "Pass" : "Fail"}:</strong> {item.name}. {item.advice}
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}
