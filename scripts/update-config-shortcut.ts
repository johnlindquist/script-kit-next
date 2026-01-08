#!/usr/bin/env bun
/**
 * update-config-shortcut.ts
 * 
 * AST-based config.ts shortcut updater.
 * 
 * This script modifies ~/.scriptkit/kit/config.ts to add/update command shortcuts
 * using TypeScript AST manipulation to preserve formatting and comments.
 * 
 * Usage:
 *   bun run scripts/update-config-shortcut.ts <command_id> <key> <cmd> <ctrl> <alt> <shift>
 * 
 * Example:
 *   bun run scripts/update-config-shortcut.ts "builtin/clipboard-history" "KeyV" true false false true
 */

import * as fs from 'node:fs';
import * as path from 'node:path';

// Types matching the config schema
interface HotkeyConfig {
  modifiers: string[];
  key: string;
}

interface CommandConfig {
  shortcut?: HotkeyConfig;
  hidden?: boolean;
  confirmationRequired?: boolean;
}

// Parse command line arguments
const args = process.argv.slice(2);
if (args.length < 6) {
  console.error('Usage: update-config-shortcut.ts <command_id> <key> <cmd> <ctrl> <alt> <shift>');
  console.error('Example: update-config-shortcut.ts "builtin/clipboard-history" "KeyV" true false false true');
  process.exit(1);
}

const [commandId, key, cmdStr, ctrlStr, altStr, shiftStr] = args;
const cmd = cmdStr === 'true';
const ctrl = ctrlStr === 'true';
const alt = altStr === 'true';
const shift = shiftStr === 'true';

// Build modifiers array
const modifiers: string[] = [];
if (cmd) modifiers.push('meta');
if (ctrl) modifiers.push('ctrl');
if (alt) modifiers.push('alt');
if (shift) modifiers.push('shift');

// Config file path
const configPath = path.join(process.env.HOME || '', '.scriptkit', 'kit', 'config.ts');

/**
 * Check if config has an actual (non-commented) commands section.
 * Look for "commands:" at the start of a line (with optional whitespace).
 */
function hasActiveCommandsSection(content: string): boolean {
  // Match "commands:" that's NOT preceded by "//" on the same line
  // Look for lines that start with whitespace + commands:
  const lines = content.split('\n');
  for (const line of lines) {
    const trimmed = line.trim();
    // Skip commented lines
    if (trimmed.startsWith('//')) continue;
    // Check for commands:
    if (/^commands\s*:/.test(trimmed)) {
      return true;
    }
  }
  return false;
}

/**
 * Parse existing commands from config.ts content.
 * Only parses actual (non-commented) commands section.
 */
function parseExistingCommands(content: string): Record<string, CommandConfig> {
  // Find the actual commands section (not commented)
  const lines = content.split('\n');
  let commandsStart = -1;
  let braceCount = 0;
  let commandsContent = '';
  let inCommands = false;
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();
    
    // Skip commented lines when looking for start
    if (!inCommands && trimmed.startsWith('//')) continue;
    
    // Check for commands: start
    if (!inCommands && /^commands\s*:\s*\{/.test(trimmed)) {
      commandsStart = i;
      inCommands = true;
      // Extract content after "commands: {"
      const match = line.match(/commands\s*:\s*(\{.*)/);
      if (match) {
        commandsContent = match[1];
        braceCount = (commandsContent.match(/\{/g) || []).length - (commandsContent.match(/\}/g) || []).length;
      }
      continue;
    }
    
    if (inCommands) {
      commandsContent += '\n' + line;
      braceCount += (line.match(/\{/g) || []).length;
      braceCount -= (line.match(/\}/g) || []).length;
      
      if (braceCount <= 0) {
        // Found the end of commands object
        break;
      }
    }
  }
  
  if (commandsStart === -1) {
    return {};
  }
  
  // Try to parse the commands object
  try {
    // Clean up for JSON parsing
    // 1. Remove trailing commas before } or ]
    let jsonStr = commandsContent.replace(/,(\s*[}\]])/g, '$1');
    
    // 2. Quote unquoted property names that aren't already quoted
    // Match word characters followed by : that aren't already in quotes
    jsonStr = jsonStr.replace(/(?<=[{,]\s*)(\w+)(?=\s*:)/g, '"$1"');
    
    // 3. Handle shortcut property which has nested object
    // The regex above might not handle all cases, so let's be more careful
    // First, handle array contents (modifiers array)
    // Already should be fine since they're quoted strings
    
    return JSON.parse(jsonStr);
  } catch (e) {
    // If JSON parsing fails, try a simpler approach: eval-like parsing
    // But for safety, we'll manually parse the structure
    console.error('Warning: Could not parse existing commands with JSON, trying manual parse');
    
    try {
      // Use a simple state machine to extract command entries
      const commands: Record<string, CommandConfig> = {};
      const lines = commandsContent.split('\n');
      let currentCommandId: string | null = null;
      let currentShortcut: HotkeyConfig | null = null;
      
      for (const line of lines) {
        const trimmed = line.trim();
        
        // Match command ID: "command/id": {
        const idMatch = trimmed.match(/^"([^"]+)"\s*:\s*\{/);
        if (idMatch) {
          currentCommandId = idMatch[1];
          continue;
        }
        
        // Match shortcut line: shortcut: { modifiers: [...], key: "..." }
        if (currentCommandId && trimmed.includes('shortcut:')) {
          const modMatch = trimmed.match(/modifiers:\s*\[(.*?)\]/);
          const keyMatch = trimmed.match(/key:\s*"([^"]+)"/);
          
          if (modMatch && keyMatch) {
            const mods = modMatch[1]
              .split(',')
              .map(s => s.trim().replace(/"/g, ''))
              .filter(s => s.length > 0);
            
            currentShortcut = {
              modifiers: mods,
              key: keyMatch[1]
            };
          }
        }
        
        // End of command entry
        if (currentCommandId && trimmed === '}') {
          if (currentShortcut) {
            commands[currentCommandId] = { shortcut: currentShortcut };
          }
          currentCommandId = null;
          currentShortcut = null;
        }
      }
      
      return commands;
    } catch (e2) {
      console.error('Warning: Manual parse also failed, starting fresh');
      return {};
    }
  }
}

/**
 * Generate a TypeScript commands object string with proper formatting.
 */
function generateCommandsObject(commands: Record<string, CommandConfig>): string {
  if (Object.keys(commands).length === 0) {
    return '{}';
  }

  const entries = Object.entries(commands).map(([id, config]) => {
    const parts: string[] = [];
    
    if (config.shortcut) {
      const mods = config.shortcut.modifiers.map(m => `"${m}"`).join(', ');
      parts.push(`      shortcut: { modifiers: [${mods}], key: "${config.shortcut.key}" }`);
    }
    if (config.hidden !== undefined) {
      parts.push(`      hidden: ${config.hidden}`);
    }
    if (config.confirmationRequired !== undefined) {
      parts.push(`      confirmationRequired: ${config.confirmationRequired}`);
    }

    return `    "${id}": {\n${parts.join(',\n')}\n    }`;
  });

  return `{\n${entries.join(',\n')}\n  }`;
}

/**
 * Find and replace the commands section in the content.
 * Returns null if no commands section was found.
 */
function replaceCommandsSection(content: string, newCommandsObj: string): string | null {
  const lines = content.split('\n');
  let commandsStart = -1;
  let commandsEnd = -1;
  let braceCount = 0;
  let inCommands = false;
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();
    
    // Skip commented lines when looking for start
    if (!inCommands && trimmed.startsWith('//')) continue;
    
    // Check for commands: start
    if (!inCommands && /^commands\s*:\s*\{/.test(trimmed)) {
      commandsStart = i;
      inCommands = true;
      braceCount = (line.match(/\{/g) || []).length - (line.match(/\}/g) || []).length;
      if (braceCount <= 0) {
        commandsEnd = i;
        break;
      }
      continue;
    }
    
    if (inCommands) {
      braceCount += (line.match(/\{/g) || []).length;
      braceCount -= (line.match(/\}/g) || []).length;
      
      if (braceCount <= 0) {
        commandsEnd = i;
        break;
      }
    }
  }
  
  if (commandsStart === -1) {
    return null;
  }
  
  // Get the indentation from the commands line
  const indentMatch = lines[commandsStart].match(/^(\s*)/);
  const indent = indentMatch ? indentMatch[1] : '  ';
  
  // Replace the lines from commandsStart to commandsEnd with the new commands
  const newCommandsLine = `${indent}commands: ${newCommandsObj},`;
  const newLines = [
    ...lines.slice(0, commandsStart),
    newCommandsLine,
    ...lines.slice(commandsEnd + 1)
  ];
  
  return newLines.join('\n');
}

/**
 * Update the config.ts file with the new shortcut.
 */
function updateConfig(): void {
  // Check if config file exists
  if (!fs.existsSync(configPath)) {
    console.error(`Config file not found: ${configPath}`);
    process.exit(1);
  }

  let content = fs.readFileSync(configPath, 'utf-8');

  // Create the new shortcut config
  const shortcutConfig: HotkeyConfig = {
    modifiers,
    key,
  };

  // Check if config already has an actual (non-commented) commands section
  const hasCommands = hasActiveCommandsSection(content);

  if (hasCommands) {
    // Parse existing commands
    const existingCommands = parseExistingCommands(content);
    
    // Update with new shortcut
    existingCommands[commandId] = {
      ...existingCommands[commandId],
      shortcut: shortcutConfig,
    };

    // Generate new commands object
    const newCommandsObj = generateCommandsObject(existingCommands);

    // Replace existing commands section
    const result = replaceCommandsSection(content, newCommandsObj);
    if (result) {
      content = result;
    } else {
      console.error('Failed to replace commands section');
      process.exit(1);
    }
  } else {
    // No commands section - need to add one
    const commands: Record<string, CommandConfig> = {
      [commandId]: { shortcut: shortcutConfig }
    };
    const commandsObj = generateCommandsObject(commands);
    const commandsSection = `\n  commands: ${commandsObj},`;

    // Find a good place to insert - before the closing } satisfies Config
    const satisfiesMatch = content.match(/(\n\s*}\s*satisfies\s*Config)/);
    if (satisfiesMatch) {
      // Insert before "} satisfies Config"
      const insertPos = content.lastIndexOf(satisfiesMatch[1]);
      content = content.slice(0, insertPos) + commandsSection + content.slice(insertPos);
    } else {
      // Fallback: try to find the closing } of export default
      // Look for the last line that's just "}" or "};" before potential TS syntax
      const lines = content.split('\n');
      let insertLineIdx = -1;
      
      // Find the line with "} satisfies Config;" or just closing brace
      for (let i = lines.length - 1; i >= 0; i--) {
        const trimmed = lines[i].trim();
        if (trimmed === '} satisfies Config;' || trimmed === '}' || trimmed === '};') {
          insertLineIdx = i;
          break;
        }
      }
      
      if (insertLineIdx >= 0) {
        lines.splice(insertLineIdx, 0, commandsSection.trim());
        content = lines.join('\n');
      } else {
        console.error('Could not find a valid insertion point in config.ts');
        process.exit(1);
      }
    }
  }

  // Write updated content
  fs.writeFileSync(configPath, content, 'utf-8');
  console.log(`Updated shortcut for ${commandId}: ${modifiers.join('+')}+${key}`);
}

// Run the update
updateConfig();
