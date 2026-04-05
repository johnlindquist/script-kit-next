import '../../../scripts/kit-sdk';

export const MINI_MAIN_WINDOW_WIDTH = 480;
export const MINI_MAIN_WINDOW_HEIGHT = 440;

export async function expectMiniMainWindow(
  label: string,
  waitMs: number = 600
) {
  if (waitMs > 0) {
    await new Promise((resolve) => setTimeout(resolve, waitMs));
  }

  const layout = await getLayoutInfo();

  console.error(
    `[LAYOUT] ${label}: ${layout.windowWidth}x${layout.windowHeight} promptType=${layout.promptType}`
  );

  const widthMatches =
    Math.abs(layout.windowWidth - MINI_MAIN_WINDOW_WIDTH) <= 1;
  const heightMatches =
    Math.abs(layout.windowHeight - MINI_MAIN_WINDOW_HEIGHT) <= 1;

  if (!widthMatches || !heightMatches) {
    throw new Error(
      `${label} expected the Mini Main Window (${MINI_MAIN_WINDOW_WIDTH}x${MINI_MAIN_WINDOW_HEIGHT}), got ${layout.windowWidth}x${layout.windowHeight} (promptType=${layout.promptType}). Open the "Mini Main Window" builtin before running this smoke test.`
    );
  }

  return layout;
}
