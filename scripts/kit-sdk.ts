import * as readline from 'node:readline';

// =============================================================================
// Types
// =============================================================================

export interface Choice {
  name: string;
  value: string;
  description?: string;
}

export interface FieldDef {
  name: string;
  label: string;
  type?: 'text' | 'password' | 'email' | 'number' | 'date' | 'time' | 'url' | 'tel' | 'color';
  placeholder?: string;
  value?: string;
}

export interface PathOptions {
  startPath?: string;
  hint?: string;
}

export interface HotkeyInfo {
  key: string;
  command: boolean;
  shift: boolean;
  option: boolean;
  control: boolean;
  shortcut: string;
  keyCode: string;
}

export interface FileInfo {
  path: string;
  name: string;
  size: number;
}

// =============================================================================
// System API Types
// =============================================================================

export interface NotifyOptions {
  title?: string;
  body?: string;
}

export interface StatusOptions {
  status: 'busy' | 'idle' | 'error';
  message: string;
}

export interface Position {
  x: number;
  y: number;
}

export interface ClipboardAPI {
  readText(): Promise<string>;
  writeText(text: string): Promise<void>;
  readImage(): Promise<Buffer>;
  writeImage(buffer: Buffer): Promise<void>;
}

export interface KeyboardAPI {
  type(text: string): Promise<void>;
  tap(...keys: string[]): Promise<void>;
}

export interface MouseAPI {
  move(positions: Position[]): Promise<void>;
  leftClick(): Promise<void>;
  rightClick(): Promise<void>;
  setPosition(position: Position): Promise<void>;
}

interface ArgMessage {
  type: 'arg';
  id: string;
  placeholder: string;
  choices: Choice[];
}

interface DivMessage {
  type: 'div';
  id: string;
  html: string;
  tailwind?: string;
}

interface EditorMessage {
  type: 'editor';
  id: string;
  content: string;
  language: string;
}

interface MiniMessage {
  type: 'mini';
  id: string;
  placeholder: string;
  choices: Choice[];
}

interface MicroMessage {
  type: 'micro';
  id: string;
  placeholder: string;
  choices: Choice[];
}

interface SelectMessage {
  type: 'select';
  id: string;
  placeholder: string;
  choices: Choice[];
  multiple: boolean;
}

interface FieldsMessage {
  type: 'fields';
  id: string;
  fields: FieldDef[];
}

interface FormMessage {
  type: 'form';
  id: string;
  html: string;
}

interface PathMessage {
  type: 'path';
  id: string;
  startPath?: string;
  hint?: string;
}

interface HotkeyMessage {
  type: 'hotkey';
  id: string;
  placeholder?: string;
}

interface DropMessage {
  type: 'drop';
  id: string;
}

interface TemplateMessage {
  type: 'template';
  id: string;
  template: string;
}

interface EnvMessage {
  type: 'env';
  id: string;
  key: string;
  secret?: boolean;
}

// System message types (fire-and-forget, no response needed)
interface BeepMessage {
  type: 'beep';
}

interface SayMessage {
  type: 'say';
  text: string;
  voice?: string;
}

interface NotifyMessage {
  type: 'notify';
  title?: string;
  body?: string;
}

interface SetStatusMessage {
  type: 'setStatus';
  status: 'busy' | 'idle' | 'error';
  message: string;
}

interface MenuMessage {
  type: 'menu';
  icon: string;
  scripts?: string[];
}

interface ClipboardMessage {
  type: 'clipboard';
  id: string;
  action: 'read' | 'write';
  format: 'text' | 'image';
  content?: string;
}

interface SetSelectedTextMessage {
  type: 'setSelectedText';
  text: string;
}

interface GetSelectedTextMessage {
  type: 'getSelectedText';
  id: string;
}

interface KeyboardMessage {
  type: 'keyboard';
  action: 'type' | 'tap';
  text?: string;
  keys?: string[];
}

interface MouseMessage {
  type: 'mouse';
  action: 'move' | 'click' | 'setPosition';
  positions?: Position[];
  button?: 'left' | 'right';
  position?: Position;
}

interface SubmitMessage {
  type: 'submit';
  id: string;
  value: string | null;
}

// =============================================================================
// Core Infrastructure
// =============================================================================

let messageId = 0;

const nextId = (): string => String(++messageId);

const pending = new Map<string, (msg: SubmitMessage) => void>();

function send(msg: object): void {
  process.stdout.write(`${JSON.stringify(msg)}\n`);
}

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
});

rl.on('line', (line: string) => {
  try {
    const msg = JSON.parse(line) as SubmitMessage;
    if (msg.type === 'submit' && pending.has(msg.id)) {
      const resolver = pending.get(msg.id);
      if (resolver) {
        pending.delete(msg.id);
        resolver(msg);
      }
    }
  } catch {
    // Silently ignore non-JSON lines
  }
});

// =============================================================================
// Global API Functions (Script Kit v1 pattern - no imports needed)
// =============================================================================

declare global {
  /**
   * Prompt user for input with optional choices
   */
  function arg(placeholder: string, choices: (string | Choice)[]): Promise<string>;
  
  /**
   * Display HTML content to user
   */
  function div(html: string, tailwind?: string): Promise<void>;
  
  /**
   * Convert Markdown to HTML
   */
  function md(markdown: string): string;
  
  /**
   * Opens a Monaco-style code editor
   * @param content - Initial content to display in the editor
   * @param language - Language for syntax highlighting (e.g., 'typescript', 'javascript', 'json')
   * @returns The edited content when user submits
   */
  function editor(content?: string, language?: string): Promise<string>;
  
  /**
   * Compact prompt variant - same API as arg() but with minimal UI
   * @param placeholder - Prompt text shown to user
   * @param choices - Array of string or Choice objects
   * @returns The selected value
   */
  function mini(placeholder: string, choices: (string | Choice)[]): Promise<string>;
  
  /**
   * Tiny prompt variant - same API as arg() but with smallest UI
   * @param placeholder - Prompt text shown to user
   * @param choices - Array of string or Choice objects
   * @returns The selected value
   */
  function micro(placeholder: string, choices: (string | Choice)[]): Promise<string>;
  
  /**
   * Multi-select prompt - allows selecting multiple choices
   * @param placeholder - Prompt text shown to user
   * @param choices - Array of string or Choice objects
   * @returns Array of selected values
   */
  function select(placeholder: string, choices: (string | Choice)[]): Promise<string[]>;
  
  /**
   * Multi-field form prompt
   * @param fieldDefs - Array of field definitions (strings become both name and label)
   * @returns Array of field values in order
   */
  function fields(fieldDefs: (string | FieldDef)[]): Promise<string[]>;
  
  /**
   * Custom HTML form prompt
   * @param html - HTML string containing form fields
   * @returns Object with form field names as keys and their values
   */
  function form(html: string): Promise<Record<string, string>>;
  
  /**
   * File/folder browser prompt
   * @param options - Optional path options (startPath, hint)
   * @returns The selected file/folder path
   */
  function path(options?: PathOptions): Promise<string>;
  
  /**
   * Capture keyboard shortcut
   * @param placeholder - Optional placeholder text
   * @returns HotkeyInfo with key details and modifier states
   */
  function hotkey(placeholder?: string): Promise<HotkeyInfo>;
  
  /**
   * Drag and drop zone
   * @returns Array of FileInfo for dropped files
   */
  function drop(): Promise<FileInfo[]>;
  
  /**
   * Tab-through template editor like VSCode snippets
   * @param template - Template string with variables ($1, $2, ${1:default})
   * @returns The filled-in template string
   */
  function template(template: string): Promise<string>;
  
  /**
   * Get/set environment variable
   * @param key - Environment variable key
   * @param promptFn - Optional function to prompt for value if not set
   * @returns The environment variable value
   */
  function env(key: string, promptFn?: () => Promise<string>): Promise<string>;
  
  // =============================================================================
  // System APIs (TIER 3)
  // =============================================================================
  
  /**
   * Play system beep sound
   */
  function beep(): Promise<void>;
  
  /**
   * Text-to-speech
   * @param text - Text to speak
   * @param voice - Optional voice name
   */
  function say(text: string, voice?: string): Promise<void>;
  
  /**
   * Show system notification
   * @param options - Notification options or body string
   */
  function notify(options: string | NotifyOptions): Promise<void>;
  
  /**
   * Set application status
   * @param options - Status options with status and message
   */
  function setStatus(options: StatusOptions): Promise<void>;
  
  /**
   * Set system menu icon and scripts
   * @param icon - Icon name/path
   * @param scripts - Optional array of script paths
   */
  function menu(icon: string, scripts?: string[]): Promise<void>;
  
  /**
   * Copy text to clipboard (alias for clipboard.writeText)
   * @param text - Text to copy
   */
  function copy(text: string): Promise<void>;
  
  /**
   * Paste text from clipboard (alias for clipboard.readText)
   * @returns Text from clipboard
   */
  function paste(): Promise<string>;
  
  /**
   * Set selected text (simulates paste at cursor)
   * @param text - Text to insert
   */
  function setSelectedText(text: string): Promise<void>;
  
  /**
   * Get selected text (simulates copy from selection)
   * @returns Selected text
   */
  function getSelectedText(): Promise<string>;
  
  /**
   * Clipboard API object
   */
  const clipboard: ClipboardAPI;
  
  /**
   * Keyboard API object
   */
  const keyboard: KeyboardAPI;
  
  /**
   * Mouse API object
   */
  const mouse: MouseAPI;
}

globalThis.arg = async function arg(
  placeholder: string,
  choices: (string | Choice)[]
): Promise<string> {
  const id = nextId();
  
  const normalizedChoices: Choice[] = choices.map((c) => {
    if (typeof c === 'string') {
      return { name: c, value: c };
    }
    return c;
  });

  return new Promise((resolve) => {
    pending.set(id, (msg: SubmitMessage) => {
      resolve(msg.value ?? '');
    });
    
    const message: ArgMessage = {
      type: 'arg',
      id,
      placeholder,
      choices: normalizedChoices,
    };
    
    send(message);
  });
};

globalThis.div = async function div(html: string, tailwind?: string): Promise<void> {
  const id = nextId();
  
  return new Promise((resolve) => {
    pending.set(id, () => {
      resolve();
    });
    
    const message: DivMessage = {
      type: 'div',
      id,
      html,
      tailwind,
    };
    
    send(message);
  });
};

globalThis.md = function md(markdown: string): string {
  let html = markdown;

  // Headings
  html = html.replace(/^### (.+)$/gm, '<h3>$1</h3>');
  html = html.replace(/^## (.+)$/gm, '<h2>$1</h2>');
  html = html.replace(/^# (.+)$/gm, '<h1>$1</h1>');

  // Bold
  html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');

  // Italic
  html = html.replace(/\*(.+?)\*/g, '<em>$1</em>');

  // Lists
  html = html.replace(/^- (.+)$/gm, '<li>$1</li>');
  html = html.replace(/(<li>.*<\/li>\n?)+/g, '<ul>$&</ul>');

  return html;
};

globalThis.editor = async function editor(
  content: string = '',
  language: string = 'text'
): Promise<string> {
  const id = nextId();

  return new Promise((resolve) => {
    pending.set(id, (msg: SubmitMessage) => {
      resolve(msg.value ?? '');
    });

    const message: EditorMessage = {
      type: 'editor',
      id,
      content,
      language,
    };

    send(message);
  });
};

globalThis.mini = async function mini(
  placeholder: string,
  choices: (string | Choice)[]
): Promise<string> {
  const id = nextId();

  const normalizedChoices: Choice[] = choices.map((c) => {
    if (typeof c === 'string') {
      return { name: c, value: c };
    }
    return c;
  });

  return new Promise((resolve) => {
    pending.set(id, (msg: SubmitMessage) => {
      resolve(msg.value ?? '');
    });

    const message: MiniMessage = {
      type: 'mini',
      id,
      placeholder,
      choices: normalizedChoices,
    };

    send(message);
  });
};

globalThis.micro = async function micro(
  placeholder: string,
  choices: (string | Choice)[]
): Promise<string> {
  const id = nextId();

  const normalizedChoices: Choice[] = choices.map((c) => {
    if (typeof c === 'string') {
      return { name: c, value: c };
    }
    return c;
  });

  return new Promise((resolve) => {
    pending.set(id, (msg: SubmitMessage) => {
      resolve(msg.value ?? '');
    });

    const message: MicroMessage = {
      type: 'micro',
      id,
      placeholder,
      choices: normalizedChoices,
    };

    send(message);
  });
};

globalThis.select = async function select(
  placeholder: string,
  choices: (string | Choice)[]
): Promise<string[]> {
  const id = nextId();

  const normalizedChoices: Choice[] = choices.map((c) => {
    if (typeof c === 'string') {
      return { name: c, value: c };
    }
    return c;
  });

  return new Promise((resolve) => {
    pending.set(id, (msg: SubmitMessage) => {
      // Value comes back as JSON array or empty
      const value = msg.value ?? '[]';
      try {
        const parsed = JSON.parse(value);
        resolve(Array.isArray(parsed) ? parsed : []);
      } catch {
        resolve([]);
      }
    });

    const message: SelectMessage = {
      type: 'select',
      id,
      placeholder,
      choices: normalizedChoices,
      multiple: true,
    };

    send(message);
  });
};

globalThis.fields = async function fields(
  fieldDefs: (string | FieldDef)[]
): Promise<string[]> {
  const id = nextId();

  const normalizedFields: FieldDef[] = fieldDefs.map((f) => {
    if (typeof f === 'string') {
      return { name: f, label: f };
    }
    return f;
  });

  return new Promise((resolve) => {
    pending.set(id, (msg: SubmitMessage) => {
      // Value comes back as JSON array of field values
      const value = msg.value ?? '[]';
      try {
        const parsed = JSON.parse(value);
        resolve(Array.isArray(parsed) ? parsed : []);
      } catch {
        resolve([]);
      }
    });

    const message: FieldsMessage = {
      type: 'fields',
      id,
      fields: normalizedFields,
    };

    send(message);
  });
};

globalThis.form = async function form(
  html: string
): Promise<Record<string, string>> {
  const id = nextId();

  return new Promise((resolve) => {
    pending.set(id, (msg: SubmitMessage) => {
      // Value comes back as JSON object with field names as keys
      const value = msg.value ?? '{}';
      try {
        const parsed = JSON.parse(value);
        resolve(typeof parsed === 'object' && parsed !== null ? parsed : {});
      } catch {
        resolve({});
      }
    });

    const message: FormMessage = {
      type: 'form',
      id,
      html,
    };

    send(message);
  });
};

globalThis.path = async function path(
  options?: PathOptions
): Promise<string> {
  const id = nextId();

  return new Promise((resolve) => {
    pending.set(id, (msg: SubmitMessage) => {
      resolve(msg.value ?? '');
    });

    const message: PathMessage = {
      type: 'path',
      id,
      startPath: options?.startPath,
      hint: options?.hint,
    };

    send(message);
  });
};

globalThis.hotkey = async function hotkey(
  placeholder?: string
): Promise<HotkeyInfo> {
  const id = nextId();

  return new Promise((resolve) => {
    pending.set(id, (msg: SubmitMessage) => {
      // Value comes back as JSON with hotkey info
      const value = msg.value ?? '{}';
      try {
        const parsed = JSON.parse(value);
        resolve({
          key: parsed.key ?? '',
          command: parsed.command ?? false,
          shift: parsed.shift ?? false,
          option: parsed.option ?? false,
          control: parsed.control ?? false,
          shortcut: parsed.shortcut ?? '',
          keyCode: parsed.keyCode ?? '',
        });
      } catch {
        resolve({
          key: '',
          command: false,
          shift: false,
          option: false,
          control: false,
          shortcut: '',
          keyCode: '',
        });
      }
    });

    const message: HotkeyMessage = {
      type: 'hotkey',
      id,
      placeholder,
    };

    send(message);
  });
};

globalThis.drop = async function drop(): Promise<FileInfo[]> {
  const id = nextId();

  return new Promise((resolve) => {
    pending.set(id, (msg: SubmitMessage) => {
      // Value comes back as JSON array of file info
      const value = msg.value ?? '[]';
      try {
        const parsed = JSON.parse(value);
        if (Array.isArray(parsed)) {
          resolve(parsed.map((f: { path?: string; name?: string; size?: number }) => ({
            path: f.path ?? '',
            name: f.name ?? '',
            size: f.size ?? 0,
          })));
        } else {
          resolve([]);
        }
      } catch {
        resolve([]);
      }
    });

    const message: DropMessage = {
      type: 'drop',
      id,
    };

    send(message);
  });
};

globalThis.template = async function template(
  templateStr: string
): Promise<string> {
  const id = nextId();

  return new Promise((resolve) => {
    pending.set(id, (msg: SubmitMessage) => {
      resolve(msg.value ?? '');
    });

    const message: TemplateMessage = {
      type: 'template',
      id,
      template: templateStr,
    };

    send(message);
  });
};

globalThis.env = async function env(
  key: string,
  promptFn?: () => Promise<string>
): Promise<string> {
  // First check if the env var is already set
  const existingValue = process.env[key];
  if (existingValue !== undefined && existingValue !== '') {
    return existingValue;
  }

  // If a prompt function is provided, use it to get the value
  if (promptFn) {
    const value = await promptFn();
    process.env[key] = value;
    return value;
  }

  // Otherwise, send a message to GPUI to prompt for the value
  const id = nextId();

  return new Promise((resolve) => {
    pending.set(id, (msg: SubmitMessage) => {
      const value = msg.value ?? '';
      process.env[key] = value;
      resolve(value);
    });

    const message: EnvMessage = {
      type: 'env',
      id,
      key,
      secret: key.toLowerCase().includes('secret') || 
              key.toLowerCase().includes('password') ||
              key.toLowerCase().includes('token') ||
              key.toLowerCase().includes('key'),
    };

    send(message);
  });
};

// =============================================================================
// TIER 3: System APIs (alerts, clipboard, keyboard, mouse)
// =============================================================================

// Fire-and-forget messages - send and resolve immediately (no response needed)
globalThis.beep = async function beep(): Promise<void> {
  const message: BeepMessage = { type: 'beep' };
  send(message);
};

globalThis.say = async function say(text: string, voice?: string): Promise<void> {
  const message: SayMessage = { type: 'say', text, voice };
  send(message);
};

globalThis.notify = async function notify(options: string | NotifyOptions): Promise<void> {
  const message: NotifyMessage = typeof options === 'string'
    ? { type: 'notify', body: options }
    : { type: 'notify', title: options.title, body: options.body };
  send(message);
};

globalThis.setStatus = async function setStatus(options: StatusOptions): Promise<void> {
  const message: SetStatusMessage = {
    type: 'setStatus',
    status: options.status,
    message: options.message,
  };
  send(message);
};

globalThis.menu = async function menu(icon: string, scripts?: string[]): Promise<void> {
  const message: MenuMessage = { type: 'menu', icon, scripts };
  send(message);
};

// Text operations that need responses
globalThis.setSelectedText = async function setSelectedText(text: string): Promise<void> {
  const message: SetSelectedTextMessage = { type: 'setSelectedText', text };
  send(message);
};

globalThis.getSelectedText = async function getSelectedText(): Promise<string> {
  const id = nextId();
  
  return new Promise((resolve) => {
    pending.set(id, (msg: SubmitMessage) => {
      resolve(msg.value ?? '');
    });
    
    const message: GetSelectedTextMessage = { type: 'getSelectedText', id };
    send(message);
  });
};

// Clipboard API object
globalThis.clipboard = {
  async readText(): Promise<string> {
    const id = nextId();
    
    return new Promise((resolve) => {
      pending.set(id, (msg: SubmitMessage) => {
        resolve(msg.value ?? '');
      });
      
      const message: ClipboardMessage = {
        type: 'clipboard',
        id,
        action: 'read',
        format: 'text',
      };
      send(message);
    });
  },
  
  async writeText(text: string): Promise<void> {
    const id = nextId();
    
    return new Promise((resolve) => {
      pending.set(id, () => {
        resolve();
      });
      
      const message: ClipboardMessage = {
        type: 'clipboard',
        id,
        action: 'write',
        format: 'text',
        content: text,
      };
      send(message);
    });
  },
  
  async readImage(): Promise<Buffer> {
    const id = nextId();
    
    return new Promise((resolve) => {
      pending.set(id, (msg: SubmitMessage) => {
        // Value comes back as base64-encoded string
        const base64 = msg.value ?? '';
        resolve(Buffer.from(base64, 'base64'));
      });
      
      const message: ClipboardMessage = {
        type: 'clipboard',
        id,
        action: 'read',
        format: 'image',
      };
      send(message);
    });
  },
  
  async writeImage(buffer: Buffer): Promise<void> {
    const id = nextId();
    
    return new Promise((resolve) => {
      pending.set(id, () => {
        resolve();
      });
      
      const message: ClipboardMessage = {
        type: 'clipboard',
        id,
        action: 'write',
        format: 'image',
        content: buffer.toString('base64'),
      };
      send(message);
    });
  },
};

// Clipboard aliases
globalThis.copy = async function copy(text: string): Promise<void> {
  return globalThis.clipboard.writeText(text);
};

globalThis.paste = async function paste(): Promise<string> {
  return globalThis.clipboard.readText();
};

// Keyboard API object
globalThis.keyboard = {
  async type(text: string): Promise<void> {
    const message: KeyboardMessage = {
      type: 'keyboard',
      action: 'type',
      text,
    };
    send(message);
  },
  
  async tap(...keys: string[]): Promise<void> {
    const message: KeyboardMessage = {
      type: 'keyboard',
      action: 'tap',
      keys,
    };
    send(message);
  },
};

// Mouse API object
globalThis.mouse = {
  async move(positions: Position[]): Promise<void> {
    const message: MouseMessage = {
      type: 'mouse',
      action: 'move',
      positions,
    };
    send(message);
  },
  
  async leftClick(): Promise<void> {
    const message: MouseMessage = {
      type: 'mouse',
      action: 'click',
      button: 'left',
    };
    send(message);
  },
  
  async rightClick(): Promise<void> {
    const message: MouseMessage = {
      type: 'mouse',
      action: 'click',
      button: 'right',
    };
    send(message);
  },
  
  async setPosition(position: Position): Promise<void> {
    const message: MouseMessage = {
      type: 'mouse',
      action: 'setPosition',
      position,
    };
    send(message);
  },
};
