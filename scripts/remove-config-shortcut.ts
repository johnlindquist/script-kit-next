#!/usr/bin/env bun
/**
 * Compatibility wrapper for shortcut recorder removals.
 *
 * Usage:
 *   bun scripts/remove-config-shortcut.ts <command_id>
 */

process.argv.splice(2, 0, 'remove-command-shortcut');
await import('./config-cli.ts');
