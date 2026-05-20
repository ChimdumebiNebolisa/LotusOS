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

type HomeAction = {
  label: string;
  description: string;
  target: Exclude<SectionId, "home">;
};

type HomeCard = {
  title: string;
  description: string;
  items: string[];
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
    description: "This placeholder screen is reserved for note capture, reading queues, and lightweight study context.",
    actions: ["Daily notes", "Study queue", "Reference snippets"],
    highlights: ["Placeholder only", "No sync required", "Offline-first direction"]
  },
  {
    id: "files",
    label: "Files",
    eyebrow: "Files",
    title: "Surface working files, not clutter.",
    description: "This placeholder screen is reserved for project folders, recent downloads, and working sets.",
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
    description: "This placeholder screen is reserved for workspace preferences, startup behavior, and local tool paths.",
    actions: ["Startup", "Workspace defaults", "Local integrations"],
    highlights: ["Deliberately small", "Source-controlled scaffold", "Expandable later"]
  }
];

const homeActions: HomeAction[] = [
  {
    label: "Projects",
    description: "Jump to the placeholder for repo launchers, pinned workspaces, and branch context.",
    target: "projects"
  },
  {
    label: "Notes",
    description: "Open the notes placeholder for local-first study and idea capture.",
    target: "notes"
  },
  {
    label: "Files",
    description: "Review the files placeholder for working sets and recent project folders.",
    target: "files"
  }
];

const homeCards: HomeCard[] = [
  {
    title: "What Lotus Shell is for",
    description: "This surface should help a user orient quickly instead of dumping them into a bare scaffold.",
    items: ["Study flow", "Coding sessions", "Local project organization"]
  },
  {
    title: "Current shell state",
    description: "This phase keeps the shell static and reliable while the broader OS path remains unchanged.",
    items: ["Packaged into the ISO", "Autostart already verified", "No new services or backends"]
  }
];

const placeholderSections = sections.filter(
  (section): section is Section & { id: Exclude<SectionId, "home"> } => section.id !== "home"
);

type HomeDashboardProps = {
  onNavigate: (sectionId: Exclude<SectionId, "home">) => void;
};

const HomeDashboard = ({ onNavigate }: HomeDashboardProps) => {
  return (
    <div className="home-dashboard">
      <section className="home-hero">
        <div className="home-hero-copy">
          <p className="eyebrow">LotusOS Home</p>
          <h2>A calm first place to start.</h2>
          <p className="description">
            Lotus Shell is the workspace layer inside LotusOS. Phase 5A turns Home into a real first-run surface with clear
            next steps, while the rest of the shell stays intentionally lightweight.
          </p>

          <div className="hero-actions">
            {homeActions.map((action) => (
              <button
                key={action.target}
                type="button"
                className="hero-action"
                onClick={() => onNavigate(action.target)}
              >
                <span className="hero-action-label">{action.label}</span>
                <span className="hero-action-copy">{action.description}</span>
              </button>
            ))}
          </div>
        </div>

        <article className="panel status-panel">
          <p className="eyebrow">Shell Status</p>
          <h3>Static, local, and ready to orient the session.</h3>
          <p>
            Lotus Shell remains a packaged app inside LotusOS. This pass only improves the first-run surface and does not
            change install, boot, or backend behavior.
          </p>
          <ul className="list status-list">
            <li>Home dashboard polish only</li>
            <li>No external app launchers added</li>
            <li>No dynamic system probing</li>
          </ul>
        </article>
      </section>

      <section className="home-grid">
        {homeCards.map((card) => (
          <article key={card.title} className="panel">
            <p className="eyebrow">Overview</p>
            <h3>{card.title}</h3>
            <p>{card.description}</p>
            <ul className="list">
              {card.items.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </article>
        ))}

        <article className="panel panel-wide">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">Next Destinations</p>
              <h3>Placeholder sections remain visible, but Home leads the flow.</h3>
            </div>
            <p className="panel-copy">Everything below stays static and local. These links only switch sections inside Lotus Shell.</p>
          </div>

          <div className="destination-grid">
            {placeholderSections.map((section) => (
              <button
                key={section.id}
                type="button"
                className="destination-card"
                onClick={() => onNavigate(section.id)}
              >
                <span className="destination-topline">
                  <span className="eyebrow">{section.eyebrow}</span>
                  <span className="placeholder-chip">Placeholder</span>
                </span>
                <span className="destination-title">{section.label}</span>
                <span className="destination-description">{section.description}</span>
              </button>
            ))}
          </div>
        </article>
      </section>
    </div>
  );
};

type PlaceholderSectionProps = {
  section: Section;
};

const PlaceholderSection = ({ section }: PlaceholderSectionProps) => {
  return (
    <>
      <header className="hero">
        <p className="eyebrow">{section.eyebrow}</p>
        <h2>{section.title}</h2>
        <p className="description">{section.description}</p>
        <p className="placeholder-label">Placeholder section</p>
      </header>

      <section className="grid">
        <article className="panel panel-wide">
          <h3>Planned Surface</h3>
          <ul className="list">
            {section.actions.map((action) => (
              <li key={action}>{action}</li>
            ))}
          </ul>
        </article>

        <article className="panel accent-panel">
          <h3>Current State</h3>
          <p>This placeholder section stays intentionally lightweight during Phase 5A while Home becomes the primary first-run surface.</p>
        </article>

        <article className="panel">
          <h3>Highlights</h3>
          <ul className="list">
            {section.highlights.map((highlight) => (
              <li key={highlight}>{highlight}</li>
            ))}
          </ul>
        </article>

        <article className="panel">
          <h3>Guardrails</h3>
          <p>No backend, sync, or OS-level integrations are introduced in this placeholder during the Home polish pass.</p>
        </article>
      </section>
    </>
  );
};

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
          <p className="caption">Phase 5A home dashboard</p>
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
        {activeSection === "home" ? <HomeDashboard onNavigate={setActiveSection} /> : <PlaceholderSection section={section} />}
      </main>
    </div>
  );
};

export default App;
