import { useEffect, useState } from "react";
import { AboutPage } from "./pages/About";
import { ActivityPage } from "./pages/Activity";
import { AgentsPage } from "./pages/Agents";
import { DashboardPage } from "./pages/Dashboard";
import { PowerPolicyPage } from "./pages/PowerPolicy";
import { SettingsPage } from "./pages/Settings";
import { useAliveStore } from "./stores/aliveStore";

const pages = ["Dashboard", "Agents", "Power Policy", "Activity", "Settings", "About"] as const;
type PageName = (typeof pages)[number];

export default function App() {
  const [page, setPage] = useState<PageName>("Dashboard");
  const refresh = useAliveStore((s) => s.refresh);
  const error = useAliveStore((s) => s.error);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <div className="shell flex">
      <a className="skip-link" href="#main">
        Skip to content
      </a>
      <nav aria-label="Alive pages" className="nav">
        <span className="brand">Alive</span>
        {pages.map((name) => (
          <button
            key={name}
            type="button"
            aria-current={page === name ? "page" : undefined}
            className={page === name ? "nav-item active" : "nav-item"}
            onClick={() => setPage(name)}
          >
            {name}
          </button>
        ))}
      </nav>
      <main id="main" className="main">
        {error ? (
          <p role="alert" className="error-banner">
            {error}
          </p>
        ) : null}
        {page === "Dashboard" ? <DashboardPage /> : null}
        {page === "Agents" ? <AgentsPage /> : null}
        {page === "Power Policy" ? <PowerPolicyPage /> : null}
        {page === "Activity" ? <ActivityPage /> : null}
        {page === "Settings" ? <SettingsPage /> : null}
        {page === "About" ? <AboutPage /> : null}
      </main>
    </div>
  );
}
