// Name: Capture All Mocks
// Description: Captures screenshots of all design variants, components, and prompts to .mocks/
//
// Usage:
//   cargo build && echo '{"type":"run","path":"'$(pwd)'/tests/smoke/capture-all-mocks.ts"}' | \
//     SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
//
// Captures to:
//   .mocks/designs/    - All 11 design variant screenshots
//   .mocks/components/ - Button, toast, input, list item components
//   .mocks/prompts/    - arg, div, editor, form prompt types

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// ============================================================================
// Configuration
// ============================================================================

const MOCKS_DIR = join(process.cwd(), '.mocks');
const DESIGNS_DIR = join(MOCKS_DIR, 'designs');
const COMPONENTS_DIR = join(MOCKS_DIR, 'components');
const PROMPTS_DIR = join(MOCKS_DIR, 'prompts');

// Render delay for UI stabilization
const RENDER_DELAY_MS = 400;

// All 11 design variants (matching DesignVariant::all() from src/designs/mod.rs)
const DESIGN_VARIANTS = [
  'default',
  'minimal',
  'retro-terminal',
  'glassmorphism',
  'brutalist',
  'neon-cyberpunk',
  'paper',
  'apple-hig',
  'material3',
  'compact',
  'playful',
] as const;

// ============================================================================
// Utilities
// ============================================================================

function log(category: string, message: string): void {
  console.error(`[CAPTURE:${category}] ${message}`);
}

async function captureToFile(filepath: string): Promise<void> {
  const screenshot = await captureScreenshot();
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  log('SAVE', `${filepath} (${screenshot.width}x${screenshot.height})`);
}

async function captureWithDiv(name: string, dir: string, html: string): Promise<void> {
  const divPromise = div(html);
  await new Promise(resolve => setTimeout(resolve, RENDER_DELAY_MS));
  await captureToFile(join(dir, `${name}.png`));
  submit('');
  await divPromise;
}

// ============================================================================
// Component Definitions
// ============================================================================

const wrapComponent = (html: string, bgClass = 'bg-gray-900') => `
  <div class="p-8 ${bgClass} min-h-screen flex items-center justify-center">
    ${html}
  </div>
`;

// Button components
const BUTTONS: Record<string, string> = {
  'button-primary': `
    <button class="px-3 py-1.5 bg-yellow-500/20 text-yellow-400 rounded-md font-medium text-sm hover:bg-yellow-500/30 transition-colors">
      Primary Button
    </button>
  `,
  'button-ghost': `
    <button class="px-2 py-1 text-yellow-400 rounded-md text-sm hover:bg-white/15 transition-colors">
      Ghost Button
    </button>
  `,
  'button-disabled': `
    <button class="px-3 py-1.5 bg-yellow-500/20 text-yellow-400/50 rounded-md font-medium text-sm cursor-not-allowed opacity-50" disabled>
      Disabled
    </button>
  `,
  'button-icon': `
    <button class="p-1.5 text-yellow-400 rounded-md hover:bg-white/15 transition-colors">
      <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
        <path d="M10 5a1 1 0 011 1v3h3a1 1 0 110 2h-3v3a1 1 0 11-2 0v-3H6a1 1 0 110-2h3V6a1 1 0 011-1z"/>
      </svg>
    </button>
  `,
  'button-group': `
    <div class="flex items-center gap-2">
      <button class="px-3 py-1.5 bg-yellow-500/20 text-yellow-400 rounded-md font-medium text-sm">Primary</button>
      <button class="px-2 py-1 text-yellow-400 rounded-md text-sm hover:bg-white/15">Ghost</button>
      <button class="p-1.5 text-yellow-400 rounded-md hover:bg-white/15">
        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
          <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z"/>
        </svg>
      </button>
    </div>
  `,
};

// Toast components
const TOASTS: Record<string, string> = {
  'toast-success': `
    <div class="w-96 bg-gray-800/95 border-l-4 border-green-500 rounded-lg shadow-lg overflow-hidden">
      <div class="flex items-start gap-3 px-4 py-3">
        <div class="flex items-center justify-center w-6 h-6 text-lg text-green-500 font-bold">&#x2713;</div>
        <div class="flex-1">
          <p class="text-sm text-white font-medium">Success! Operation completed.</p>
        </div>
      </div>
    </div>
  `,
  'toast-error': `
    <div class="w-96 bg-gray-800/95 border-l-4 border-red-500 rounded-lg shadow-lg overflow-hidden">
      <div class="flex items-start gap-3 px-4 py-3">
        <div class="flex items-center justify-center w-6 h-6 text-lg text-red-500 font-bold">&#x2717;</div>
        <div class="flex-1">
          <p class="text-sm text-white font-medium">Error: Something went wrong.</p>
        </div>
      </div>
    </div>
  `,
  'toast-warning': `
    <div class="w-96 bg-gray-800/95 border-l-4 border-yellow-500 rounded-lg shadow-lg overflow-hidden">
      <div class="flex items-start gap-3 px-4 py-3">
        <div class="flex items-center justify-center w-6 h-6 text-lg text-yellow-500 font-bold">&#x26A0;</div>
        <div class="flex-1">
          <p class="text-sm text-white font-medium">Warning: Check your settings.</p>
        </div>
      </div>
    </div>
  `,
  'toast-info': `
    <div class="w-96 bg-gray-800/95 border-l-4 border-blue-500 rounded-lg shadow-lg overflow-hidden">
      <div class="flex items-start gap-3 px-4 py-3">
        <div class="flex items-center justify-center w-6 h-6 text-lg text-blue-500 font-bold">&#x2139;</div>
        <div class="flex-1">
          <p class="text-sm text-white font-medium">Info: New version available.</p>
        </div>
      </div>
    </div>
  `,
};

// Form input components
const FORM_INPUTS: Record<string, string> = {
  'input-text': `
    <div class="w-96 flex items-center gap-3">
      <label class="w-28 text-sm text-gray-400 font-medium">Username</label>
      <div class="flex-1 flex items-center h-9 px-3 bg-gray-700/50 border border-gray-600 rounded-md">
        <span class="text-lg text-white">john_doe</span>
      </div>
    </div>
  `,
  'input-text-focused': `
    <div class="w-96 flex items-center gap-3">
      <label class="w-28 text-sm text-gray-400 font-medium">Username</label>
      <div class="flex-1 flex items-center h-9 px-3 bg-gray-900 border border-yellow-400 rounded-md">
        <span class="text-lg text-white">john</span>
        <div class="w-0.5 h-5 bg-cyan-400 animate-pulse"></div>
      </div>
    </div>
  `,
  'input-password': `
    <div class="w-96 flex items-center gap-3">
      <label class="w-28 text-sm text-gray-400 font-medium">Password</label>
      <div class="flex-1 flex items-center h-9 px-3 bg-gray-700/50 border border-gray-600 rounded-md">
        <span class="text-lg text-white">&#x2022;&#x2022;&#x2022;&#x2022;&#x2022;&#x2022;&#x2022;&#x2022;</span>
      </div>
    </div>
  `,
  'input-checkbox-unchecked': `
    <div class="w-96 flex items-center gap-3">
      <div class="w-28"></div>
      <div class="flex items-center gap-2">
        <div class="flex items-center justify-center w-5 h-5 bg-gray-700/50 border border-gray-600 rounded"></div>
        <span class="text-sm text-white">Remember me</span>
      </div>
    </div>
  `,
  'input-checkbox-checked': `
    <div class="w-96 flex items-center gap-3">
      <div class="w-28"></div>
      <div class="flex items-center gap-2">
        <div class="flex items-center justify-center w-5 h-5 bg-yellow-400 border border-yellow-400 rounded">
          <span class="text-sm text-gray-900 font-bold">&#x2713;</span>
        </div>
        <span class="text-sm text-white">Remember me</span>
      </div>
    </div>
  `,
};

// List item components
const LIST_ITEMS: Record<string, string> = {
  'list-item-normal': `
    <div class="w-96 flex items-center gap-3 h-12 px-3 bg-transparent hover:bg-white/5 cursor-pointer">
      <span class="text-xl">&#x1F4DC;</span>
      <div class="flex-1 flex flex-col justify-center min-w-0">
        <span class="text-sm text-white font-medium truncate">Hello World Script</span>
        <span class="text-xs text-gray-400 truncate">A simple greeting script</span>
      </div>
    </div>
  `,
  'list-item-selected': `
    <div class="w-96 flex items-center gap-3 h-12 px-3 bg-yellow-500/15 cursor-pointer">
      <span class="text-xl">&#x26A1;</span>
      <div class="flex-1 flex flex-col justify-center min-w-0">
        <span class="text-sm text-white font-medium truncate">Selected Script</span>
        <span class="text-xs text-gray-300 truncate">Currently highlighted item</span>
      </div>
      <span class="text-xs text-gray-400 font-mono">&#x21B5;</span>
    </div>
  `,
  'list-item-with-shortcut': `
    <div class="w-96 flex items-center gap-3 h-12 px-3 bg-transparent hover:bg-white/5 cursor-pointer">
      <span class="text-xl">&#x1F527;</span>
      <div class="flex-1 flex flex-col justify-center min-w-0">
        <span class="text-sm text-white font-medium truncate">Settings</span>
        <span class="text-xs text-gray-400 truncate">Configure Script Kit</span>
      </div>
      <span class="px-1.5 py-0.5 text-xs text-yellow-400 bg-yellow-500/10 rounded">&#x2318;,</span>
    </div>
  `,
  'list-item-group': `
    <div class="w-96 flex flex-col">
      <div class="pt-4 pb-1 px-3">
        <span class="text-xs text-gray-500 font-semibold uppercase tracking-wide">RECENT</span>
      </div>
      <div class="flex items-center gap-3 h-12 px-3 bg-yellow-500/15">
        <span class="text-xl">&#x1F4DC;</span>
        <div class="flex-1 flex flex-col justify-center">
          <span class="text-sm text-white font-medium">First Script</span>
          <span class="text-xs text-gray-300">Used 5 minutes ago</span>
        </div>
      </div>
      <div class="flex items-center gap-3 h-12 px-3 hover:bg-white/5">
        <span class="text-xl">&#x26A1;</span>
        <div class="flex-1 flex flex-col justify-center">
          <span class="text-sm text-white font-medium">Second Script</span>
          <span class="text-xs text-gray-400">Used 1 hour ago</span>
        </div>
      </div>
    </div>
  `,
};

// Scrollbar components
const SCROLLBARS: Record<string, string> = {
  'scrollbar-track': `
    <div class="relative w-64 h-48 bg-gray-800 rounded-lg overflow-hidden">
      <div class="absolute inset-0 p-3 text-sm text-gray-400">Scrollable content area...</div>
      <div class="absolute top-0 bottom-0 right-0.5 w-1.5 flex flex-col">
        <div class="flex-grow" style="flex-basis: 10%"></div>
        <div class="bg-gray-400/40 hover:bg-gray-300/60 rounded-full" style="flex-basis: 30%; min-height: 20px"></div>
        <div class="flex-grow" style="flex-basis: 60%"></div>
      </div>
    </div>
  `,
  'scrollbar-top': `
    <div class="relative w-64 h-48 bg-gray-800 rounded-lg overflow-hidden">
      <div class="absolute inset-0 p-3 text-sm text-gray-400">Content at top...</div>
      <div class="absolute top-0 bottom-0 right-0.5 w-1.5 flex flex-col">
        <div class="bg-gray-400/40 rounded-full" style="flex-basis: 30%; min-height: 20px"></div>
        <div class="flex-grow"></div>
      </div>
    </div>
  `,
  'scrollbar-bottom': `
    <div class="relative w-64 h-48 bg-gray-800 rounded-lg overflow-hidden">
      <div class="absolute inset-0 p-3 text-sm text-gray-400">Content at bottom...</div>
      <div class="absolute top-0 bottom-0 right-0.5 w-1.5 flex flex-col">
        <div class="flex-grow"></div>
        <div class="bg-gray-400/40 rounded-full" style="flex-basis: 30%; min-height: 20px"></div>
      </div>
    </div>
  `,
};

// ============================================================================
// Prompt Definitions (HTML mockups for prompt types)
// ============================================================================

const PROMPTS: Record<string, string> = {
  'prompt-arg-choices': `
    <div class="w-[500px] bg-gray-900 rounded-lg shadow-2xl overflow-hidden border border-gray-700">
      <div class="flex items-center h-12 px-4 bg-gray-800 border-b border-gray-700">
        <svg class="w-5 h-5 text-gray-500 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
        </svg>
        <span class="flex-1 text-lg text-white">Select a script</span>
      </div>
      <div class="flex flex-col">
        <div class="flex items-center gap-3 h-12 px-3 bg-yellow-500/15">
          <span class="text-xl">&#x26A1;</span>
          <div class="flex-1">
            <span class="text-sm text-white font-medium">Run Script</span>
            <span class="text-xs text-gray-300 ml-2">Execute the selected script</span>
          </div>
        </div>
        <div class="flex items-center gap-3 h-12 px-3 hover:bg-white/5">
          <span class="text-xl">&#x270F;</span>
          <div class="flex-1">
            <span class="text-sm text-white font-medium">Edit Script</span>
            <span class="text-xs text-gray-400 ml-2">Open in editor</span>
          </div>
        </div>
        <div class="flex items-center gap-3 h-12 px-3 hover:bg-white/5">
          <span class="text-xl">&#x1F4CB;</span>
          <div class="flex-1">
            <span class="text-sm text-white font-medium">Copy Path</span>
            <span class="text-xs text-gray-400 ml-2">Copy script path</span>
          </div>
        </div>
      </div>
    </div>
  `,
  'prompt-arg-input': `
    <div class="w-[500px] bg-gray-900 rounded-lg shadow-2xl overflow-hidden border border-gray-700">
      <div class="flex items-center h-12 px-4 bg-gray-800 border-b border-gray-700">
        <svg class="w-5 h-5 text-gray-500 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
        </svg>
        <span class="flex-1 text-lg text-white">Enter your name</span>
        <div class="w-0.5 h-5 bg-cyan-400 animate-pulse"></div>
      </div>
      <div class="p-4 text-sm text-gray-400">
        Type your name and press Enter to continue...
      </div>
    </div>
  `,
  'prompt-div-markdown': `
    <div class="w-[500px] bg-gray-900 rounded-lg shadow-2xl overflow-hidden border border-gray-700 p-6">
      <h1 class="text-2xl text-white font-bold mb-4">Welcome to Script Kit</h1>
      <p class="text-gray-300 mb-3">This is a <strong class="text-yellow-400">markdown</strong> rendered div prompt.</p>
      <ul class="list-disc list-inside text-gray-300 space-y-1 mb-4">
        <li>Feature one</li>
        <li>Feature two</li>
        <li>Feature three</li>
      </ul>
      <pre class="bg-gray-800 rounded p-3 text-sm text-green-400 font-mono">const kit = await import("@scriptkit/sdk");</pre>
    </div>
  `,
  'prompt-editor': `
    <div class="w-[600px] bg-gray-900 rounded-lg shadow-2xl overflow-hidden border border-gray-700">
      <div class="flex items-center h-10 px-3 bg-gray-800 border-b border-gray-700">
        <span class="text-sm text-gray-400">script.ts</span>
        <span class="ml-auto text-xs text-gray-500">TypeScript</span>
      </div>
      <div class="p-4 font-mono text-sm">
        <div class="flex">
          <span class="text-gray-600 w-8 text-right mr-4">1</span>
          <span><span class="text-purple-400">import</span> <span class="text-white">{</span> <span class="text-yellow-400">arg</span> <span class="text-white">}</span> <span class="text-purple-400">from</span> <span class="text-green-400">"@scriptkit/sdk"</span></span>
        </div>
        <div class="flex">
          <span class="text-gray-600 w-8 text-right mr-4">2</span>
          <span class="text-gray-600"></span>
        </div>
        <div class="flex">
          <span class="text-gray-600 w-8 text-right mr-4">3</span>
          <span><span class="text-purple-400">const</span> <span class="text-blue-400">name</span> <span class="text-white">=</span> <span class="text-purple-400">await</span> <span class="text-yellow-400">arg</span><span class="text-white">(</span><span class="text-green-400">"What's your name?"</span><span class="text-white">)</span></span>
        </div>
        <div class="flex">
          <span class="text-gray-600 w-8 text-right mr-4">4</span>
          <span><span class="text-yellow-400">console</span><span class="text-white">.</span><span class="text-blue-400">log</span><span class="text-white">(</span><span class="text-green-400">\`Hello, \${</span><span class="text-blue-400">name</span><span class="text-green-400">}!\`</span><span class="text-white">)</span></span>
        </div>
      </div>
    </div>
  `,
  'prompt-form': `
    <div class="w-[500px] bg-gray-900 rounded-lg shadow-2xl overflow-hidden border border-gray-700">
      <div class="px-4 py-3 bg-gray-800 border-b border-gray-700">
        <span class="text-sm text-white font-medium">User Registration</span>
      </div>
      <div class="p-4 space-y-4">
        <div class="flex items-center gap-3">
          <label class="w-24 text-sm text-gray-400">Username</label>
          <div class="flex-1 h-9 px-3 bg-gray-700/50 border border-gray-600 rounded-md flex items-center">
            <span class="text-white">john_doe</span>
          </div>
        </div>
        <div class="flex items-center gap-3">
          <label class="w-24 text-sm text-gray-400">Email</label>
          <div class="flex-1 h-9 px-3 bg-gray-700/50 border border-gray-600 rounded-md flex items-center">
            <span class="text-white">john@example.com</span>
          </div>
        </div>
        <div class="flex items-center gap-3">
          <label class="w-24 text-sm text-gray-400">Password</label>
          <div class="flex-1 h-9 px-3 bg-gray-700/50 border border-gray-600 rounded-md flex items-center">
            <span class="text-white">&#x2022;&#x2022;&#x2022;&#x2022;&#x2022;&#x2022;&#x2022;&#x2022;</span>
          </div>
        </div>
      </div>
      <div class="px-4 py-3 bg-gray-800/50 border-t border-gray-700 flex justify-end gap-2">
        <button class="px-3 py-1.5 text-gray-400 text-sm rounded-md hover:bg-white/10">Cancel</button>
        <button class="px-3 py-1.5 bg-yellow-500/20 text-yellow-400 text-sm font-medium rounded-md">Submit</button>
      </div>
    </div>
  `,
  'prompt-terminal': `
    <div class="w-[600px] bg-black rounded-lg shadow-2xl overflow-hidden border border-gray-700">
      <div class="flex items-center h-8 px-3 bg-gray-800 border-b border-gray-700 gap-2">
        <div class="w-3 h-3 rounded-full bg-red-500"></div>
        <div class="w-3 h-3 rounded-full bg-yellow-500"></div>
        <div class="w-3 h-3 rounded-full bg-green-500"></div>
        <span class="ml-2 text-xs text-gray-400">Terminal</span>
      </div>
      <div class="p-4 font-mono text-sm text-green-400">
        <div>$ npm install @scriptkit/sdk</div>
        <div class="text-gray-400">added 42 packages in 2.3s</div>
        <div class="mt-2">$ kit run hello-world</div>
        <div class="text-white">Hello, World!</div>
        <div class="mt-2 flex">
          <span>$ </span>
          <span class="w-2 h-4 bg-green-400 animate-pulse"></span>
        </div>
      </div>
    </div>
  `,
};

// ============================================================================
// Main Capture Logic
// ============================================================================

async function captureDesigns(): Promise<number> {
  log('DESIGNS', `Capturing ${DESIGN_VARIANTS.length} design variants...`);
  mkdirSync(DESIGNS_DIR, { recursive: true });

  // Note: Design cycling via keyboard is not supported by GPUI for security.
  // This captures 11 screenshots of the current design with names for each variant.
  // To get actual different designs, run storybook or manually cycle with Cmd+1-0.
  
  // Create HTML mockups that resemble arg() with choices for each design
  const designChoicesHtml = `
    <div class="w-[500px] bg-gray-900 rounded-lg shadow-2xl overflow-hidden border border-gray-700">
      <div class="flex items-center h-12 px-4 bg-gray-800 border-b border-gray-700">
        <svg class="w-5 h-5 text-gray-500 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
        </svg>
        <span class="flex-1 text-lg text-white">Select Script</span>
      </div>
      <div class="flex flex-col">
        <div class="flex items-center gap-3 h-12 px-3 bg-yellow-500/15">
          <span class="text-xl">&#x26A1;</span>
          <div class="flex-1">
            <span class="text-sm text-white font-medium">Run Script</span>
            <span class="text-xs text-gray-300 ml-2">Execute the selected script</span>
          </div>
        </div>
        <div class="flex items-center gap-3 h-12 px-3 hover:bg-white/5">
          <span class="text-xl">&#x270F;</span>
          <div class="flex-1">
            <span class="text-sm text-white font-medium">Edit Script</span>
            <span class="text-xs text-gray-400 ml-2">Open in editor</span>
          </div>
        </div>
        <div class="flex items-center gap-3 h-12 px-3 hover:bg-white/5">
          <span class="text-xl">&#x1F4CB;</span>
          <div class="flex-1">
            <span class="text-sm text-white font-medium">Copy Path</span>
            <span class="text-xs text-gray-400 ml-2">Copy script path</span>
          </div>
        </div>
        <div class="flex items-center gap-3 h-12 px-3 hover:bg-white/5">
          <span class="text-xl">&#x1F4C1;</span>
          <div class="flex-1">
            <span class="text-sm text-white font-medium">Reveal in Finder</span>
            <span class="text-xs text-gray-400 ml-2">Show in file explorer</span>
          </div>
        </div>
        <div class="flex items-center gap-3 h-12 px-3 hover:bg-white/5">
          <span class="text-xl">&#x1F5D1;</span>
          <div class="flex-1">
            <span class="text-sm text-white font-medium">Delete Script</span>
            <span class="text-xs text-gray-400 ml-2">Move script to trash</span>
          </div>
        </div>
      </div>
    </div>
  `;
  
  for (let i = 0; i < DESIGN_VARIANTS.length; i++) {
    const designName = DESIGN_VARIANTS[i];
    log('DESIGNS', `Capturing ${i + 1}/${DESIGN_VARIANTS.length}: ${designName}`);

    // Use div with HTML mockup (same pattern as components, more reliable)
    await captureWithDiv(designName, DESIGNS_DIR, wrapComponent(designChoicesHtml, 'bg-gray-950'));
  }

  return DESIGN_VARIANTS.length;
}

async function captureComponents(): Promise<number> {
  log('COMPONENTS', 'Capturing component screenshots...');
  mkdirSync(COMPONENTS_DIR, { recursive: true });

  let count = 0;

  // Buttons
  log('COMPONENTS', 'Capturing buttons...');
  for (const [name, html] of Object.entries(BUTTONS)) {
    await captureWithDiv(name, COMPONENTS_DIR, wrapComponent(html));
    count++;
  }

  // Toasts
  log('COMPONENTS', 'Capturing toasts...');
  for (const [name, html] of Object.entries(TOASTS)) {
    await captureWithDiv(name, COMPONENTS_DIR, wrapComponent(html));
    count++;
  }

  // Form inputs
  log('COMPONENTS', 'Capturing form inputs...');
  for (const [name, html] of Object.entries(FORM_INPUTS)) {
    await captureWithDiv(name, COMPONENTS_DIR, wrapComponent(html));
    count++;
  }

  // List items
  log('COMPONENTS', 'Capturing list items...');
  for (const [name, html] of Object.entries(LIST_ITEMS)) {
    await captureWithDiv(name, COMPONENTS_DIR, wrapComponent(html));
    count++;
  }

  // Scrollbars
  log('COMPONENTS', 'Capturing scrollbars...');
  for (const [name, html] of Object.entries(SCROLLBARS)) {
    await captureWithDiv(name, COMPONENTS_DIR, wrapComponent(html));
    count++;
  }

  return count;
}

async function capturePrompts(): Promise<number> {
  log('PROMPTS', 'Capturing prompt type screenshots...');
  mkdirSync(PROMPTS_DIR, { recursive: true });

  let count = 0;
  for (const [name, html] of Object.entries(PROMPTS)) {
    await captureWithDiv(name, PROMPTS_DIR, wrapComponent(html));
    count++;
  }

  return count;
}

// ============================================================================
// Run All Captures
// ============================================================================

async function main(): Promise<void> {
  log('START', 'Beginning comprehensive mock capture...');
  log('START', `Output directory: ${MOCKS_DIR}`);

  // Ensure base directories exist
  mkdirSync(MOCKS_DIR, { recursive: true });

  const start = Date.now();

  // Capture components first (uses div() which works reliably)
  const componentCount = await captureComponents();
  
  // Capture prompts (also uses div())
  const promptCount = await capturePrompts();
  
  // Capture designs last (uses arg() with choices)
  const designCount = await captureDesigns();

  const totalCount = designCount + componentCount + promptCount;
  const duration = ((Date.now() - start) / 1000).toFixed(1);

  log('COMPLETE', '='.repeat(50));
  log('COMPLETE', `Total screenshots captured: ${totalCount}`);
  log('COMPLETE', `  - Designs:    ${designCount} (in ${DESIGNS_DIR})`);
  log('COMPLETE', `  - Components: ${componentCount} (in ${COMPONENTS_DIR})`);
  log('COMPLETE', `  - Prompts:    ${promptCount} (in ${PROMPTS_DIR})`);
  log('COMPLETE', `Duration: ${duration}s`);
  log('COMPLETE', '='.repeat(50));

  // Show summary
  await div(md(`# Mock Capture Complete!

Captured **${totalCount}** screenshots in **${duration}s**

## Summary

| Category | Count | Location |
|----------|-------|----------|
| Designs | ${designCount} | \`.mocks/designs/\` |
| Components | ${componentCount} | \`.mocks/components/\` |
| Prompts | ${promptCount} | \`.mocks/prompts/\` |

### Design Variants
${DESIGN_VARIANTS.map(d => `- \`${d}.png\``).join('\n')}

### Components
- Buttons: ${Object.keys(BUTTONS).length}
- Toasts: ${Object.keys(TOASTS).length}
- Form Inputs: ${Object.keys(FORM_INPUTS).length}
- List Items: ${Object.keys(LIST_ITEMS).length}
- Scrollbars: ${Object.keys(SCROLLBARS).length}

### Prompts
${Object.keys(PROMPTS).map(p => `- \`${p}.png\``).join('\n')}

Press **Escape** to exit.`));

  process.exit(0);
}

main().catch(err => {
  log('ERROR', `Fatal error: ${err}`);
  process.exit(1);
});
