#!/usr/bin/env bun
/**
 * Compatibility wrapper for shortcut recorder writes.
 *
 * Usage:
 *   bun scripts/update-config-shortcut.ts <command_id> <key> <cmd> <ctrl> <alt> <shift>
 */

process.argv.splice(2, 0, 'set-command-shortcut');
await import('./config-cli.ts');
