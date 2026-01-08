#!/usr/bin/env bun
/**
 * remove-config-shortcut.ts
 * 
 * AST-based config.ts shortcut remover.
 * 
 * This script modifies ~/.scriptkit/kit/config.ts to remove a command shortcut
 * by line-based parsing to preserve formatting and comments.
 * 
 * Usage:
 *   bun run scripts/remove-config-shortcut.ts <command_id>
 * 
 * Example:
 *   bun run scripts/remove-config-shortcut.ts "builtin/clipboard-history"
 */

import * as fs from 'node:fs';
import * as path from 'node:path';

// Parse command line arguments
const args = process.argv.slice(2);
if (args.length < 1) {
  console.error('Usage: remove-config-shortcut.ts <command_id>');
  console.error('Example: remove-config-shortcut.ts "builtin/clipboard-history"');
  process.exit(1);
}

const [commandId] = args;

// Config file path
const configPath = path.join(process.env.HOME || '', '.scriptkit', 'kit', 'config.ts');

/**
 * Remove a command entry from config.ts by line-based parsing.
 */
function removeShortcut(): void {
  // Check if config file exists
  if (!fs.existsSync(configPath)) {
    console.error(`Config file not found: ${configPath}`);
    process.exit(1);
  }

  const content = fs.readFileSync(configPath, 'utf-8');
  const lines = content.split('\n');
  const outputLines: string[] = [];
  
  let inCommands = false;
  let inTargetCommand = false;
  let braceCount = 0;
  let commandsStart = -1;
  let commandsEnd = -1;
  let targetStart = -1;
  let targetEnd = -1;
  let foundTarget = false;
  
  // First pass: find the commands section and the target command
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();
    
    // Skip commented lines when looking for commands start
    if (!inCommands && trimmed.startsWith('//')) {
      continue;
    }
    
    // Look for commands: { (not commented)
    if (!inCommands && /^commands\s*:\s*\{/.test(trimmed)) {
      inCommands = true;
      commandsStart = i;
      braceCount = (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
      continue;
    }
    
    if (inCommands && !inTargetCommand) {
      // Look for our target command ID
      if (trimmed.includes(`"${commandId}"`) && trimmed.includes(':')) {
        inTargetCommand = true;
        targetStart = i;
        foundTarget = true;
        // Count braces in this line
        braceCount += (line.match(/\{/g) || []).length;
        braceCount -= (line.match(/\}/g) || []).length;
        continue;
      }
      
      // Track end of commands section
      braceCount += (line.match(/\{/g) || []).length;
      braceCount -= (line.match(/\}/g) || []).length;
      
      if (braceCount <= 0) {
        commandsEnd = i;
        inCommands = false;
      }
      continue;
    }
    
    if (inTargetCommand) {
      braceCount += (line.match(/\{/g) || []).length;
      braceCount -= (line.match(/\}/g) || []).length;
      
      // Check if we've closed the target command
      if (braceCount <= 1) { // Back to commands level
        targetEnd = i;
        inTargetCommand = false;
      }
    }
  }
  
  if (!foundTarget) {
    console.log(`Command "${commandId}" not found in config - nothing to remove`);
    return;
  }
  
  // Second pass: build output excluding the target command
  let skipUntil = -1;
  for (let i = 0; i < lines.length; i++) {
    if (i === targetStart) {
      // Start skipping
      skipUntil = targetEnd;
      continue;
    }
    
    if (i <= skipUntil) {
      continue;
    }
    
    // Handle trailing comma on the line before targetStart
    // If the previous included line ends with comma and next is closing brace, remove comma
    outputLines.push(lines[i]);
  }
  
  // Clean up: remove empty lines between command entries and trailing commas before }
  let result = outputLines.join('\n');
  
  // Remove trailing comma before closing brace in commands section
  result = result.replace(/,(\s*\n\s*\})/g, '$1');
  
  // Remove excess newlines
  result = result.replace(/\n{3,}/g, '\n\n');
  
  // Clean up empty commands section: commands: {\n  },
  result = result.replace(/commands\s*:\s*\{\s*\n?\s*\},?\n?/g, '');
  
  fs.writeFileSync(configPath, result, 'utf-8');
  console.log(`Removed shortcut for ${commandId}`);
}

// Run the removal
removeShortcut();
