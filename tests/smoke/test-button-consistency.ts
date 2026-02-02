import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync, statSync } from 'fs';
import { join } from 'path';

export const metadata = {
  name: 'Test Button Consistency',
  description: 'Visual smoke test for button states and styling',
};

const correlationId =
  globalThis.crypto?.randomUUID?.() ??
  `button-consistency-${Date.now()}-${Math.random().toString(16).slice(2)}`;

const issues: string[] = [];

function log(
  level: 'info' | 'warn' | 'error',
  message: string,
  extra: Record<string, unknown> = {}
) {
  console.error(
    JSON.stringify({
      correlation_id: correlationId,
      level,
      message,
      timestamp: new Date().toISOString(),
      ...extra,
    })
  );
}

log('info', 'test-start');

await div(`
  <div class="p-6 flex flex-col gap-4 bg-white text-gray-900">
    <div class="text-sm font-semibold">Button Consistency Test</div>

    <div class="flex flex-col gap-3">
      <button class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 cursor-pointer">
        Normal Button
      </button>

      <button class="px-4 py-2 bg-blue-600 text-white rounded cursor-pointer">
        Hovered Button (Simulated)
      </button>

      <button class="px-4 py-2 bg-gray-300 text-gray-500 rounded cursor-not-allowed" disabled>
        Disabled Button
      </button>

      <button class="p-2 rounded hover:bg-gray-200 cursor-pointer w-10 h-10 flex items-center justify-center">
        &#128295;
      </button>
    </div>
  </div>
`);

await new Promise((resolve) => setTimeout(resolve, 500));

try {
  const screenshot = await captureScreenshot();
  if (!screenshot?.data) {
    issues.push('screenshot data missing');
  }

  const dir = join(process.cwd(), 'test-screenshots');
  mkdirSync(dir, { recursive: true });

  const filePath = join(dir, 'button-consistency.png');
  writeFileSync(filePath, Buffer.from(screenshot.data, 'base64'));

  const sizeBytes = statSync(filePath).size;
  if (sizeBytes < 10_000) {
    issues.push(`screenshot unexpectedly small (${sizeBytes} bytes)`);
  }

  if (issues.length > 0) {
    log('warn', 'issues-found', { issues, screenshot_path: filePath, screenshot_size_bytes: sizeBytes });
  } else {
    log('info', 'screenshot-saved', { screenshot_path: filePath, screenshot_size_bytes: sizeBytes });
  }
} catch (error) {
  issues.push(`screenshot capture failed: ${String(error)}`);
  log('error', 'screenshot-capture-failed', { error: String(error) });
  process.exit(1);
}

log('info', 'test-complete', { issues });
process.exit(0);
