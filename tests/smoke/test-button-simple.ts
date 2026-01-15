// Simple Button styling test
import '../../scripts/kit-sdk';

async function main() {
  // Schedule screenshot capture
  setTimeout(async () => {
    try {
      const screenshot = await captureScreenshot();
      const { writeFileSync, mkdirSync } = await import('fs');
      const { join } = await import('path');
      
      const dir = join(process.cwd(), 'test-screenshots');
      mkdirSync(dir, { recursive: true });
      
      const path = join(dir, `button-simple-${Date.now()}.png`);
      writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
      console.error(`[SCREENSHOT] ${path}`);
      process.exit(0);
    } catch (e) {
      console.error(`[ERROR] ${e}`);
      process.exit(1);
    }
  }, 800);

  // Simpler content
  div(`
    <div style="padding: 24px;">
      <h2 style="color: white; font-size: 18px; margin-bottom: 16px;">Button Focus States</h2>
      
      <div style="display: flex; gap: 12px; margin-bottom: 24px;">
        <div style="flex: 1; padding: 10px 16px; border-radius: 6px; border: 1px solid rgba(255,255,255,0.2); color: #fbbf24; text-align: center; font-size: 14px;">
          Cancel (unfocused)
        </div>
        <div style="flex: 1; padding: 10px 16px; border-radius: 6px; border: 2px solid rgba(251,191,36,0.6); background: rgba(255,255,255,0.1); color: #fbbf24; text-align: center; font-size: 14px; font-weight: 500;">
          Confirm (FOCUSED) ✓
        </div>
      </div>
      
      <div style="display: flex; gap: 12px;">
        <div style="flex: 1; padding: 10px 16px; border-radius: 6px; border: 2px solid rgba(251,191,36,0.6); background: rgba(251,191,36,0.1); color: #fbbf24; text-align: center; font-size: 14px; font-weight: 500;">
          Cancel (FOCUSED) ✓
        </div>
        <div style="flex: 1; padding: 10px 16px; border-radius: 6px; border: 1px solid rgba(251,191,36,0.4); background: rgba(50,50,50,0.5); color: #fbbf24; text-align: center; font-size: 14px;">
          Confirm (unfocused)
        </div>
      </div>
      
      <p style="color: #888; font-size: 12px; margin-top: 16px;">
        Focus ring = 2px yellow border + subtle background tint
      </p>
    </div>
  `);
}

main();
