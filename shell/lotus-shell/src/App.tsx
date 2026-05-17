import { useState } from "react";

type SectionId = "home" | "projects" | "notes" | "files" | "ai-hub" | "settings";

type Section = {
  id: SectionId;
  label: string;
  eyebrow: string;
  title: string;
  description: string;
  actions: string[];
  highlights: string[];
};

const sections: Section[] = [
  {
    id: "home",
    label: "Home",
    eyebrow: "Front Door",
    title: "Start in one calm place.",
    description:
      "Lotus Shell is the workspace layer for study, coding, and project flow inside LotusOS.",
    actions: ["Resume current work", "Open recent project", "Review today"],
    highlights: ["Live session ready", "KDE desktop running", "Phase 3 scaffold verified"]
  },
  {
    id: "projects",
    label: "Projects",
    eyebrow: "Projects",
    title: "Track active work without noise.",
    description: "This placeholder screen is reserved for repo launchers, task context, and active branches.",
    actions: ["Recent repositories", "Pinned workspaces", "Branch status"],
    highlights: ["Minimal scaffold", "No background services", "Local-first"]
  },
  {
    id: "notes",
    label: "Notes",
    eyebrow: "Notes",
    title: "Keep research and ideas nearby.",
    description: "This screen is reserved for note capture, reading queues, and lightweight study context.",
    actions: ["Daily notes", "Study queue", "Reference snippets"],
    highlights: ["Placeholder only", "No sync required", "Offline-first direction"]
  },
  {
    id: "files",
    label: "Files",
    eyebrow: "Files",
    title: "Surface working files, not clutter.",
    description: "This screen is reserved for project folders, recent downloads, and working sets.",
    actions: ["Project folders", "Recent files", "Pinned directories"],
    highlights: ["KDE integration planned", "No custom file manager", "Focused workflow"]
  },
  {
    id: "ai-hub",
    label: "AI Hub",
    eyebrow: "AI Hub",
    title: "Reserve the AI entry point without shipping credentials.",
    description: "This placeholder protects the local-first boundary until the OS boot/install path is stable.",
    actions: ["Model launchers", "Prompt workspace", "Offline tools"],
    highlights: ["No API keys bundled", "No cloud account required", "Future phase"]
  },
  {
    id: "settings",
    label: "Settings",
    eyebrow: "Settings",
    title: "Keep Lotus Shell configurable, not sprawling.",
    description: "This screen is reserved for workspace preferences, startup behavior, and local tool paths.",
    actions: ["Startup", "Workspace defaults", "Local integrations"],
    highlights: ["Deliberately small", "Source-controlled scaffold", "Expandable later"]
  }
];

const App = () => {
  const [activeSection, setActiveSection] = useState<SectionId>("home");
  const section = sections.find((item) => item.id === activeSection) ?? sections[0];

  return (
    <div className="app-shell">
      <aside className="rail">
        <div className="brand-mark">Lotus</div>
        <div className="brand-copy">
          <p className="eyebrow">LotusOS</p>
          <h1>Shell</h1>
          <p className="caption">Phase 3 packaged live-session app</p>
        </div>

        <nav className="nav">
          {sections.map((item) => (
            <button
              key={item.id}
              type="button"
              className={item.id === activeSection ? "nav-item active" : "nav-item"}
              onClick={() => setActiveSection(item.id)}
            >
              <span>{item.label}</span>
            </button>
          ))}
        </nav>
      </aside>

      <main className="workspace">
        <header className="hero">
          <p className="eyebrow">{section.eyebrow}</p>
          <h2>{section.title}</h2>
          <p className="description">{section.description}</p>
        </header>

        <section className="grid">
          <article className="panel panel-wide">
            <h3>Focus Queue</h3>
            <ul className="list">
              {section.actions.map((action) => (
                <li key={action}>{action}</li>
              ))}
            </ul>
          </article>

          <article className="panel accent-panel">
            <h3>Build State</h3>
            <p>Lotus Shell is now a real packaged app inside the live image, replacing the previous launcher-only placeholder.</p>
          </article>

          <article className="panel">
            <h3>Current Highlights</h3>
            <ul className="list">
              {section.highlights.map((highlight) => (
                <li key={highlight}>{highlight}</li>
              ))}
            </ul>
          </article>

          <article className="panel">
            <h3>Guardrails</h3>
            <p>No AI credentials, no cloud-only assumptions, and no extra OS abstractions were added in this milestone.</p>
          </article>
        </section>
      </main>
    </div>
  );
};

export default App;
