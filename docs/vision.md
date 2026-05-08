# LotusOS Vision

## Product Niche

LotusOS is a custom Linux-based operating system for AI-native student and developer workflow.

It is for people who:

- Code often.
- Use AI tools heavily.
- Manage schoolwork, PDFs, notes, repositories, and projects.
- Want a clean workspace instead of a cluttered generic Linux install.
- Want useful defaults without manually configuring everything.
- Want local-first ownership instead of a cloud-only productivity app.

## Product Definition

LotusOS is the full bootable OS image.

Lotus Shell is the custom app/interface layer inside LotusOS. It should become the calm front door for projects, notes, files, study resources, and AI-assisted workflows, but it is not the operating system by itself.

## MVP Definition

The first successful MVP is a bootable Debian-based ISO that:

- Boots in QEMU.
- Reaches a working KDE Plasma desktop.
- Shows LotusOS branding where feasible.
- Includes basic productivity and developer tools.
- Includes Lotus Shell preinstalled or clearly stubbed as a first-class app.
- Documents build and test commands.
- Avoids breaking or hiding the difference between planned and verified features.

## Non-Goals For The MVP

- Custom kernel.
- Custom package manager.
- systemd replacement.
- New desktop environment.
- Real AI integrations.
- Cloud account system.
- Heavy package bundling without documented reason.
- Claims of installability before Calamares is wired and tested.

