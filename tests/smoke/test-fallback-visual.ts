import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Show main menu and search for something that won't match any scripts
await arg({
  placeholder: "Type 'xyzfallback' to see fallback section...",
  choices: []
});

process.exit(0);
