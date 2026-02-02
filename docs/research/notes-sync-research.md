# Notes App Sync & Persistence Research

Date: 2026-02-01
Scope: saving, syncing, offline behavior, and conflict-resolution patterns in mainstream notes apps.

## Executive summary (cross-app patterns)

1) **Local-first autosave + background sync is the norm**
   - Most apps save edits immediately to local storage and then sync in the background when connectivity is available (Evernote, OneNote, Notion, RemNote). Offline edits are queued and synced once the device reconnects.

2) **Offline access is usually explicit or scoped**
   - Many apps require a per-note/page "Available offline" toggle or only guarantee offline access for previously opened content (Notion, Evernote mobile). Some desktop apps keep full local databases by default (Evernote desktop, OneNote).

3) **Conflict resolution varies widely**
   - **Automatic merges**: CRDT or algorithmic merges for text (Notion, Obsidian for Markdown). Some apps claim safe automatic merges even with multiple offline devices (RemNote).
   - **Last-write-wins**: used for non-text or non-mergeable files (Obsidian).
   - **User-mediated**: duplicate/conflict copies for manual review (Bear, OneNote) or side‑by‑side conflict UIs (Notesnook).

4) **Data safety often beats convenience**
   - Several apps intentionally avoid auto-merge in conflict scenarios to prevent data loss, preferring user review and explicit selection of the version to keep (Notesnook, Bear, OneNote).

## App-specific findings

### Apple Notes (iCloud)
- iCloud Notes syncs across iPhone/iPad/Mac and iCloud.com when the same Apple Account is used and Notes sync is enabled on each device.
- Apple docs focus on setup and access; they do not document conflict resolution behavior for Notes specifically.

Sources:
- https://support.apple.com/guide/icloud/set-up-notes-mm8685520792/icloud

### Evernote
- Evernote **automatically saves and syncs** notes across connected devices by default.
- Desktop apps keep a local database, so notes are available offline; changes made offline sync when the device reconnects.
- Mobile apps cache limited data unless notes/notebooks are marked for offline use; offline edits sync on reconnect.

Sources:
- https://help.evernote.com/hc/en-us/articles/34680763992467-How-to-sync-your-notes-across-devices
- https://help.evernote.com/hc/en-us/articles/209005917-Access-notes-offline
- https://help.evernote.com/hc/en-us/articles/209005177-Set-up-offline-notes-and-notebooks-on-mobile-devices

### Notion
- Notion’s offline mode relies on a **persistent local storage layer** that tracks offline pages and stores all data required to render them.
- Pages marked for offline access are migrated to a **CRDT data model** for conflict resolution, and Notion avoids showing partially cached pages offline.
- Offline changes save locally and **sync automatically** when the device reconnects; offline access is managed per page/device.

Sources:
- https://www.notion.com/blog/how-we-made-notion-available-offline
- https://www.notion.com/help/guides/working-offline-in-notion-everything-you-need-to-know

### Obsidian Sync
- Conflicts occur when the same file is edited on multiple devices before syncing, especially while offline.
- **Markdown files** are merged using Google’s diff‑match‑patch algorithm.
- **Other file types** use a **last‑modified‑wins** strategy.
- Users can choose between **automatic merge** (default) or **create conflict files** with timestamped names.

Sources:
- https://help.obsidian.md/sync/troubleshoot

### Bear
- Bear Pro **automatically syncs** notes; users don’t need to manually sync.
- When conflicts occur, Bear shows **all versions in the note list** so the user can review and keep the correct one; this avoids data loss.

Sources:
- https://bear.app/faq/sync-troubleshooting/
- https://bear.app/faq/how-bear-pro-handles-conflicted-notes/

### OneNote
- OneNote creates a **local copy** for offline editing and merges changes when reconnecting; it can merge without conflicts in many cases.
- When a conflict does occur (same paragraph edited concurrently), OneNote **creates multiple copies** of the page to avoid data loss, and the user merges/deletes versions.

Sources:
- https://support.microsoft.com/en-us/office/sync-a-notebook-in-onenote-1986c4cf-7716-4c78-b7e7-479be30992c7
- https://support.microsoft.com/en-gb/office/fix-issues-when-you-can-t-sync-onenote-299495ef-66d1-448f-90c1-b785a6968d45

### Notesnook
- Merge conflicts occur when the same note is edited on multiple devices; Notesnook **does not auto‑merge** to avoid data loss.
- Conflicted notes appear at the top of the list with a **side‑by‑side resolution UI**. Users can keep one version, discard the other, or save a copy.

Sources:
- https://help.notesnook.com/faqs/what-are-merge-conflicts

### RemNote
- Offline edits **sync automatically** on reconnect, and RemNote states it can **auto‑merge changes from multiple offline devices**.

Sources:
- https://help.remnote.com/en/articles/6752029-offline-mode

### Notability (iCloud)
- iCloud sync is automatic and **two‑way** across devices with the same Apple ID.
- Deleting a note from iCloud **also deletes it in Notability** (explicit two‑way behavior).

Sources:
- https://support.gingerlabs.com/hc/en-us/articles/206061487-Syncing-Notes-across-Devices-with-iCloud

### MyScript Notes / Nebo
- Notes can sync across devices via iCloud, Google Drive, or Dropbox with a MyScript account.
- **Auto‑sync is enabled by default** but can be switched to manual.
- Notes remain stored in the connected cloud service even if the app is uninstalled and reappear after reinstall and sign‑in.

Sources:
- https://help.myscript.com/notes/faq/
- https://help.myscript.com/notes/export-sync-and-back-up/cloud-sync/
- https://www.nebo.app/features/

### Quick‑note apps (example: Sticky Notes in Menubar)
- iCloud sync keeps notes backed up and synced across Macs; sync resumes once the device is back online.

Sources:
- https://quicknoteapp.com/docs/iCloud/iCloudSync/

## Conflict‑resolution pattern matrix

| App | Conflict strategy | User involvement | Notes |
| --- | --- | --- | --- |
| Notion | CRDT merge for offline‑enabled pages | Low | Avoids partial offline pages | 
| Obsidian | Auto‑merge for Markdown; LWW for other files | Low/Med | Optional conflict files | 
| Bear | Duplicate conflicted versions | High | User selects/merges content | 
| OneNote | Duplicate page copies | High | User merges content, deletes extra | 
| Notesnook | No auto‑merge; side‑by‑side resolver | High | Keep/discard/copy | 
| RemNote | Auto‑merge on reconnect | Low | Multi‑device offline edits supported | 

## Design implications (for Script Kit Notes)

- **Autosave locally, sync in background** with explicit “last sync” status.
- **Offline mode should be explicit** (per‑note or per‑collection), especially if partial data is possible.
- **Conflict strategy should be explicit and consistent**:
  - Auto‑merge for plaintext; last‑write‑wins for binary assets; or
  - Conflict files/versions with a simple side‑by‑side merge UI for safety.
- **Prefer data safety over silent overwrite** to avoid user distrust.

