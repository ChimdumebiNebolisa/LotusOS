import { invoke, isTauri } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

type SectionId = "home" | "projects" | "notes" | "files" | "ai-hub" | "settings";
type SnapshotStatus = "loading" | "ready" | "preview" | "error";
type LauncherStatus = "loading" | "ready" | "preview" | "error";

type Section = {
  id: SectionId;
  label: string;
  eyebrow: string;
  title: string;
  description: string;
  actions: string[];
  highlights: string[];
  statusLabel?: string;
};

type HomeAction = {
  label: string;
  description: string;
  target: Exclude<SectionId, "home">;
};

type SystemSnapshot = {
  lotusName: string;
  lotusPrettyName: string;
  lotusStage: string;
  osName: string;
  osPrettyName: string;
  baseId: string;
  versionCodename: string;
  username: string;
  hostname: string;
  sessionMode: string;
  sessionType: string;
  currentDesktop: string;
  desktopSession: string;
  displayProtocol: string;
  hasCalamaresLauncher: boolean;
};

type DetailItem = {
  label: string;
  value: string;
};

type LocalApp = {
  id: string;
  label: string;
  description: string;
  available: boolean;
  visible: boolean;
};

const sections: Section[] = [
  {
    id: "home",
    label: "Home",
    eyebrow: "Front Door",
    title: "Start in one clear place.",
    description:
      "Lotus Shell is the home and workspace layer for study, coding, and project flow inside LotusOS.",
    actions: ["Check this session", "Open local tools", "Review project spaces"],
    highlights: ["LotusOS preview shell", "Local-first desktop surface", "Read-only system context"]
  },
  {
    id: "projects",
    label: "Projects",
    eyebrow: "Projects",
    title: "Track active work without noise.",
    description: "This placeholder screen is reserved for repo launchers, task context, and active branches.",
    actions: ["Recent repositories", "Pinned workspaces", "Branch status"],
    highlights: ["Minimal scaffold", "No background services", "Local-first"],
    statusLabel: "Placeholder"
  },
  {
    id: "notes",
    label: "Notes",
    eyebrow: "Notes",
    title: "Keep research and ideas nearby.",
    description: "This placeholder screen is reserved for note capture, reading queues, and lightweight study context.",
    actions: ["Daily notes", "Study queue", "Reference snippets"],
    highlights: ["Placeholder only", "No sync required", "Offline-first direction"],
    statusLabel: "Placeholder"
  },
  {
    id: "files",
    label: "Files",
    eyebrow: "Files",
    title: "Surface working files, not clutter.",
    description: "This placeholder screen is reserved for project folders, recent downloads, and working sets.",
    actions: ["Project folders", "Recent files", "Pinned directories"],
    highlights: ["KDE integration planned", "No custom file manager", "Focused workflow"],
    statusLabel: "Placeholder"
  },
  {
    id: "ai-hub",
    label: "AI Hub",
    eyebrow: "AI Hub",
    title: "Reserve the AI entry point without shipping credentials.",
    description: "This placeholder protects the local-first boundary until the OS boot/install path is stable.",
    actions: ["Model launchers", "Prompt workspace", "Offline tools"],
    highlights: ["No API keys bundled", "No cloud account required", "Future phase"],
    statusLabel: "Placeholder"
  },
  {
    id: "settings",
    label: "Settings",
    eyebrow: "Settings",
    title: "Read the current system, not a speculative config panel.",
    description: "This surface shows a narrow local snapshot of the current LotusOS session without writing any preferences.",
    actions: ["Release identity", "Session details", "Installer availability"],
    highlights: ["Read-only overview", "No preferences stored", "Local-only snapshot"],
    statusLabel: "Read-only"
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

const browserPreviewSnapshot: SystemSnapshot = {
  lotusName: "LotusOS",
  lotusPrettyName: "LotusOS Preview",
  lotusStage: "preview",
  osName: "Unknown",
  osPrettyName: "Unknown",
  baseId: "Unknown",
  versionCodename: "Unknown",
  username: "Unknown",
  hostname: "Unknown",
  sessionMode: "preview",
  sessionType: "Unknown",
  currentDesktop: "Unknown",
  desktopSession: "Unknown",
  displayProtocol: "Unknown",
  hasCalamaresLauncher: false
};

const browserPreviewApps: LocalApp[] = [
  {
    id: "terminal",
    label: "Terminal",
    description: "Available inside LotusOS Preview when running in the packaged desktop session.",
    available: false,
    visible: true
  },
  {
    id: "files",
    label: "Files",
    description: "Available inside LotusOS Preview when Dolphin is present in the packaged session.",
    available: false,
    visible: true
  },
  {
    id: "browser",
    label: "Browser",
    description: "Available inside LotusOS Preview when Firefox ESR is present in the packaged session.",
    available: false,
    visible: true
  },
  {
    id: "editor",
    label: "Editor",
    description: "Available inside LotusOS Preview when Kate is present in the packaged session.",
    available: false,
    visible: true
  }
];

const destinationSections = sections.filter(
  (section): section is Section & { id: Exclude<SectionId, "home"> } => section.id !== "home"
);

const sectionLaunchers: Record<Exclude<SectionId, "home" | "settings">, string[]> = {
  projects: ["terminal", "editor"],
  notes: ["editor", "pdf"],
  files: ["files", "terminal"],
  "ai-hub": ["browser", "terminal"]
};

const formatSessionMode = (sessionMode: string) => {
  if (sessionMode === "live") {
    return "Live session";
  }

  if (sessionMode === "installed") {
    return "Installed system";
  }

  if (sessionMode === "preview") {
    return "Browser preview";
  }

  return "Unknown session";
};

const formatSnapshotStatus = (snapshotStatus: SnapshotStatus) => {
  if (snapshotStatus === "ready") {
    return "Runtime snapshot";
  }

  if (snapshotStatus === "preview") {
    return "Preview fallback";
  }

  if (snapshotStatus === "error") {
    return "Fallback snapshot";
  }

  return "Loading snapshot";
};

const formatInstallerAvailability = (snapshot: SystemSnapshot) => {
  if (!snapshot.hasCalamaresLauncher) {
    return "Installer launcher not detected in this session.";
  }

  if (snapshot.sessionMode === "live") {
    return "Installer launcher detected in the live session.";
  }

  if (snapshot.sessionMode === "installed") {
    return "Installer launcher detected on this installed system.";
  }

  return "Installer availability can only be confirmed inside the Tauri runtime.";
};

const formatSnapshotDescription = (snapshot: SystemSnapshot, snapshotStatus: SnapshotStatus) => {
  if (snapshotStatus === "loading") {
    return "Lotus Shell is loading a local system snapshot so Home and Settings can reflect the current session truthfully.";
  }

  if (snapshotStatus === "ready") {
    if (snapshot.sessionMode === "live") {
      return "Lotus Shell can see a live LotusOS preview session, its current desktop context, and whether the installer surface is present.";
    }

    return "Lotus Shell can see an installed LotusOS preview session, its current desktop context, and a narrow set of local release details.";
  }

  return "Lotus Shell is running outside the packaged Tauri runtime, so the UI is showing a safe fallback snapshot instead of local OS data.";
};

const buildHomeFacts = (snapshot: SystemSnapshot): DetailItem[] => [
  { label: "LotusOS identity", value: snapshot.lotusPrettyName },
  { label: "Base system", value: snapshot.osPrettyName },
  { label: "User and host", value: `${snapshot.username} @ ${snapshot.hostname}` },
  { label: "Desktop context", value: `${snapshot.displayProtocol} / ${snapshot.sessionType}` }
];

const buildSettingsFacts = (snapshot: SystemSnapshot): DetailItem[] => [
  { label: "LotusOS name", value: snapshot.lotusName },
  { label: "LotusOS release", value: snapshot.lotusPrettyName },
  { label: "LotusOS stage", value: snapshot.lotusStage },
  { label: "Base OS", value: snapshot.osName },
  { label: "Base release", value: snapshot.osPrettyName },
  { label: "Version codename", value: snapshot.versionCodename },
  { label: "Base ID", value: snapshot.baseId },
  { label: "User", value: snapshot.username },
  { label: "Hostname", value: snapshot.hostname },
  { label: "Session mode", value: formatSessionMode(snapshot.sessionMode) },
  { label: "Session type", value: snapshot.sessionType },
  { label: "Display protocol", value: snapshot.displayProtocol },
  { label: "Current desktop", value: snapshot.currentDesktop },
  { label: "Desktop session", value: snapshot.desktopSession },
  { label: "Calamares launcher", value: snapshot.hasCalamaresLauncher ? "Present" : "Not detected" }
];

const loadSystemSnapshot = async (): Promise<{ snapshot: SystemSnapshot; status: SnapshotStatus }> => {
  if (!isTauri()) {
    return { snapshot: browserPreviewSnapshot, status: "preview" };
  }

  try {
    const snapshot = await invoke<SystemSnapshot>("get_system_snapshot");

    return {
      snapshot: {
        ...browserPreviewSnapshot,
        ...snapshot
      },
      status: "ready"
    };
  } catch (error) {
    console.error("Failed to load system snapshot", error);

    return { snapshot: browserPreviewSnapshot, status: "error" };
  }
};

const loadLocalApps = async (): Promise<{ apps: LocalApp[]; status: LauncherStatus }> => {
  if (!isTauri()) {
    return { apps: browserPreviewApps, status: "preview" };
  }

  try {
    const apps = await invoke<LocalApp[]>("get_local_apps");

    return {
      apps,
      status: "ready"
    };
  } catch (error) {
    console.error("Failed to load local apps", error);

    return {
      apps: browserPreviewApps,
      status: "error"
    };
  }
};

const launchLocalApp = async (appId: string) => {
  await invoke("launch_local_app", { appId });
};

const formatLauncherStatus = (launcherStatus: LauncherStatus) => {
  if (launcherStatus === "ready") {
    return "Local launchers ready";
  }

  if (launcherStatus === "preview") {
    return "Preview launcher list";
  }

  if (launcherStatus === "error") {
    return "Launcher fallback";
  }

  return "Loading launchers";
};

const launcherCopy = (launcherStatus: LauncherStatus) => {
  if (launcherStatus === "ready") {
    return "These actions only target an allowlisted set of locally installed tools.";
  }

  return "Launcher actions are shown as a safe preview outside the packaged Tauri runtime.";
};

const filterAppsByIds = (apps: LocalApp[], ids: string[]) =>
  ids.map((id) => apps.find((app) => app.id === id)).filter((app): app is LocalApp => Boolean(app));

type HomeDashboardProps = {
  snapshot: SystemSnapshot;
  snapshotStatus: SnapshotStatus;
  apps: LocalApp[];
  launcherStatus: LauncherStatus;
  launchMessage: string | null;
  launchingAppId: string | null;
  onNavigate: (sectionId: Exclude<SectionId, "home">) => void;
  onLaunch: (appId: string) => void;
};

const HomeDashboard = ({
  snapshot,
  snapshotStatus,
  apps,
  launcherStatus,
  launchMessage,
  launchingAppId,
  onNavigate,
  onLaunch
}: HomeDashboardProps) => {
  return (
    <div className="home-dashboard">
      <section className="home-hero">
        <div className="home-hero-copy">
          <p className="eyebrow">LotusOS Home</p>
          <h2>A calm first place to start.</h2>
          <p className="description">
            Lotus Shell is the workspace layer inside LotusOS Preview. This pass keeps the shell narrow while making Home
            feel like a deliberate front door instead of a scaffold.
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
          <div className="status-header">
            <p className="eyebrow">Shell Status</p>
            <div className="badge-row">
              <span className="status-badge">{formatSessionMode(snapshot.sessionMode)}</span>
              <span className="status-badge subtle">{formatSnapshotStatus(snapshotStatus)}</span>
            </div>
          </div>

          <h3>{snapshotStatus === "ready" ? "Local system context loaded." : "Local system context is limited."}</h3>
          <p>{formatSnapshotDescription(snapshot, snapshotStatus)}</p>

          <dl className="fact-grid compact">
            {buildHomeFacts(snapshot).map((fact) => (
              <div key={fact.label} className="fact">
                <dt className="fact-label">{fact.label}</dt>
                <dd className="fact-value">{fact.value}</dd>
              </div>
            ))}
          </dl>

          <p className="panel-note">{formatInstallerAvailability(snapshot)}</p>
        </article>
      </section>

      <section className="home-grid">
        <article className="panel">
          <p className="eyebrow">Overview</p>
          <h3>What LotusOS Preview is trying to feel like</h3>
          <p>This surface should orient the session quickly instead of dropping the desktop into a bare app stub.</p>
          <ul className="list">
            <li>Calm first-run desktop</li>
            <li>Study and coding flow</li>
            <li>Local project organization</li>
          </ul>
        </article>

        <article className="panel">
          <p className="eyebrow">Local Launchers</p>
          <h3>Open installed tools without leaving the shell blind.</h3>
          <p>
            {launcherCopy(launcherStatus)}
          </p>
          <div className="launch-status-row">
            <span className="status-badge subtle">{formatLauncherStatus(launcherStatus)}</span>
            {launchMessage ? <span className="launch-feedback">{launchMessage}</span> : null}
          </div>
          <div className="launcher-grid">
            {apps.map((app) => (
              <button
                key={app.id}
                type="button"
                className={app.available ? "launcher-card" : "launcher-card disabled"}
                onClick={() => onLaunch(app.id)}
                disabled={!app.available || launchingAppId === app.id}
              >
                <span className="launcher-title-row">
                  <span className="launcher-title">{app.label}</span>
                  <span className={app.available ? "launcher-chip" : "launcher-chip muted"}>
                    {app.available ? "Ready" : "Unavailable"}
                  </span>
                </span>
                <span className="launcher-description">{app.description}</span>
              </button>
            ))}
          </div>
        </article>

        <article className="panel panel-wide">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">Next Destinations</p>
              <h3>Placeholder sections stay narrow while Settings becomes a read-only overview.</h3>
            </div>
            <p className="panel-copy">
              Navigation still stays inside Lotus Shell. No external launchers are added until a later phase.
            </p>
          </div>

          <div className="destination-grid">
            {destinationSections.map((section) => (
              <button
                key={section.id}
                type="button"
                className="destination-card"
                onClick={() => onNavigate(section.id)}
              >
                <span className="destination-topline">
                  <span className="eyebrow">{section.eyebrow}</span>
                  {section.statusLabel ? <span className="placeholder-chip">{section.statusLabel}</span> : null}
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
  apps: LocalApp[];
  launcherStatus: LauncherStatus;
  launchMessage: string | null;
  launchingAppId: string | null;
  onLaunch: (appId: string) => void;
};

const PlaceholderSection = ({
  section,
  apps,
  launcherStatus,
  launchMessage,
  launchingAppId,
  onLaunch
}: PlaceholderSectionProps) => {
  const relatedApps = filterAppsByIds(apps, sectionLaunchers[section.id as Exclude<SectionId, "home" | "settings">] ?? []);

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
          <p>This section stays intentionally lightweight while the shell exposes a small set of useful local actions.</p>
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
          <h3>Quick Actions</h3>
          {relatedApps.length > 0 ? (
            <>
              <div className="launch-status-row compact">
                <span className="status-badge subtle">{formatLauncherStatus(launcherStatus)}</span>
                {launchMessage ? <span className="launch-feedback">{launchMessage}</span> : null}
              </div>
              <div className="launcher-list">
                {relatedApps.map((app) => (
                  <button
                    key={app.id}
                    type="button"
                    className={app.available ? "launcher-row" : "launcher-row disabled"}
                    onClick={() => onLaunch(app.id)}
                    disabled={!app.available || launchingAppId === app.id}
                  >
                    <span className="launcher-row-copy">
                      <span className="launcher-title">{app.label}</span>
                      <span className="launcher-description">{app.description}</span>
                    </span>
                    <span className={app.available ? "launcher-chip" : "launcher-chip muted"}>
                      {app.available ? "Open" : "Unavailable"}
                    </span>
                  </button>
                ))}
              </div>
            </>
          ) : (
            <p>No local launcher is mapped to this placeholder yet.</p>
          )}
        </article>
      </section>
    </>
  );
};

type SettingsOverviewSectionProps = {
  section: Section;
  snapshot: SystemSnapshot;
  snapshotStatus: SnapshotStatus;
};

const SettingsOverviewSection = ({ section, snapshot, snapshotStatus }: SettingsOverviewSectionProps) => {
  return (
    <>
      <header className="hero">
        <div className="status-header">
          <div>
            <p className="eyebrow">{section.eyebrow}</p>
            <h2>{section.title}</h2>
          </div>
          <div className="badge-row">
            <span className="status-badge">{formatSessionMode(snapshot.sessionMode)}</span>
            <span className="status-badge subtle">{section.statusLabel}</span>
          </div>
        </div>

        <p className="description">{section.description}</p>
        <p className="panel-note">{formatSnapshotDescription(snapshot, snapshotStatus)}</p>
      </header>

      <section className="grid">
        <article className="panel panel-wide">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">System Snapshot</p>
              <h3>Read-only local context</h3>
            </div>
            <p className="panel-copy">{formatSnapshotStatus(snapshotStatus)}</p>
          </div>

          <dl className="fact-grid">
            {buildSettingsFacts(snapshot).map((fact) => (
              <div key={fact.label} className="fact">
                <dt className="fact-label">{fact.label}</dt>
                <dd className="fact-value">{fact.value}</dd>
              </div>
            ))}
          </dl>
        </article>

        <article className="panel accent-panel">
          <h3>Session Notes</h3>
          <ul className="list">
            <li>{snapshot.sessionMode === "live" ? "This session appears to be running from live media." : "This session appears to be running from an installed system."}</li>
            <li>{snapshot.currentDesktop === "Unknown" ? "Desktop metadata is limited in the current runtime." : `Current desktop reports as ${snapshot.currentDesktop}.`}</li>
            <li>{snapshot.desktopSession === "Unknown" ? "Desktop session name is not exposed right now." : `Desktop session reports as ${snapshot.desktopSession}.`}</li>
          </ul>
        </article>

        <article className="panel">
          <h3>Installer Surface</h3>
          <p>{formatInstallerAvailability(snapshot)}</p>
        </article>

        <article className="panel">
          <h3>Guardrails</h3>
          <ul className="list">
            <li>This view does not store preferences.</li>
            <li>This view does not launch external tools yet.</li>
            <li>This view only reflects a narrow local snapshot.</li>
          </ul>
        </article>
      </section>
    </>
  );
};

const App = () => {
  const [activeSection, setActiveSection] = useState<SectionId>("home");
  const [snapshot, setSnapshot] = useState<SystemSnapshot>(browserPreviewSnapshot);
  const [snapshotStatus, setSnapshotStatus] = useState<SnapshotStatus>("loading");
  const [apps, setApps] = useState<LocalApp[]>(browserPreviewApps);
  const [launcherStatus, setLauncherStatus] = useState<LauncherStatus>("loading");
  const [launchMessage, setLaunchMessage] = useState<string | null>(null);
  const [launchingAppId, setLaunchingAppId] = useState<string | null>(null);
  const section = sections.find((item) => item.id === activeSection) ?? sections[0];

  useEffect(() => {
    let cancelled = false;

    const hydrateRuntime = async () => {
      const [snapshotResult, appResult] = await Promise.all([loadSystemSnapshot(), loadLocalApps()]);

      if (cancelled) {
        return;
      }

      setSnapshot(snapshotResult.snapshot);
      setSnapshotStatus(snapshotResult.status);
      setApps(appResult.apps.filter((app) => app.visible));
      setLauncherStatus(appResult.status);
    };

    hydrateRuntime();

    return () => {
      cancelled = true;
    };
  }, []);

  const handleLaunch = async (appId: string) => {
    setLaunchMessage(null);
    setLaunchingAppId(appId);

    try {
      await launchLocalApp(appId);
      setLaunchMessage("Launcher command sent.");
    } catch (error) {
      const message = error instanceof Error ? error.message : "Launcher failed.";
      setLaunchMessage(message);
    } finally {
      setLaunchingAppId(null);
    }
  };

  return (
    <div className="app-shell">
      <aside className="rail">
        <div className="brand-mark">LO</div>
        <div className="brand-copy">
          <p className="eyebrow">LotusOS</p>
          <h1>Shell</h1>
          <p className="caption">Preview workspace surface</p>
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
        {activeSection === "home" ? (
          <HomeDashboard
            snapshot={snapshot}
            snapshotStatus={snapshotStatus}
            apps={apps}
            launcherStatus={launcherStatus}
            launchMessage={launchMessage}
            launchingAppId={launchingAppId}
            onNavigate={setActiveSection}
            onLaunch={handleLaunch}
          />
        ) : null}
        {activeSection === "settings" ? (
          <SettingsOverviewSection section={section} snapshot={snapshot} snapshotStatus={snapshotStatus} />
        ) : null}
        {activeSection !== "home" && activeSection !== "settings" ? (
          <PlaceholderSection
            section={section}
            apps={apps}
            launcherStatus={launcherStatus}
            launchMessage={launchMessage}
            launchingAppId={launchingAppId}
            onLaunch={handleLaunch}
          />
        ) : null}
      </main>
    </div>
  );
};

export default App;
