#!/usr/bin/env bun

import { readFile } from "node:fs/promises";

type CoverageRule = {
  file: string;
  mustInclude: string[];
};

const RULES: CoverageRule[] = [
  {
    file: "kit-init/examples/README.md",
    mustInclude: [
      "scriptlets/acp-chat/main.md",
      "scriptlets/custom-actions/main.md",
      "scriptlets/notes/main.md",
    ],
  },
  {
    file: "kit-init/skills/scriptlets/SKILL.md",
    mustInclude: [
      "~/.scriptkit/kit/examples/scriptlets/acp-chat/main.md",
      "~/.scriptkit/kit/examples/scriptlets/custom-actions/main.md",
      "~/.scriptkit/kit/examples/scriptlets/custom-actions/main.actions.md",
      "~/.scriptkit/kit/examples/scriptlets/notes/main.md",
    ],
  },
  {
    file: "kit-init/skills/acp-chat/SKILL.md",
    mustInclude: [
      "## Related Examples",
      "~/.scriptkit/kit/examples/scriptlets/acp-chat/main.md",
      "~/.scriptkit/kit/examples/scriptlets/acp-chat.md",
    ],
  },
  {
    file: "kit-init/skills/custom-actions/SKILL.md",
    mustInclude: [
      "## Related Examples",
      "~/.scriptkit/kit/examples/scriptlets/custom-actions/main.md",
      "~/.scriptkit/kit/examples/scriptlets/custom-actions/main.actions.md",
      "~/.scriptkit/kit/examples/scriptlets/custom-actions.md",
      "~/.scriptkit/kit/examples/scriptlets/custom-actions.actions.md",
    ],
  },
  {
    file: "kit-init/skills/notes/SKILL.md",
    mustInclude: [
      "## Related Examples",
      "~/.scriptkit/kit/examples/scriptlets/notes/main.md",
      "~/.scriptkit/kit/examples/scriptlets/notes.md",
    ],
  },
];

async function main(): Promise<void> {
  let failures = 0;

  for (const rule of RULES) {
    const text = await readFile(rule.file, "utf8");
    const missing = rule.mustInclude.filter(
      (needle) => !text.includes(needle)
    );

    console.log(
      JSON.stringify({
        type: "extension_example_coverage",
        file: rule.file,
        ok: missing.length === 0,
        missing,
      })
    );

    if (missing.length > 0) failures += 1;
  }

  if (failures > 0) {
    process.exitCode = 1;
  }
}

await main();
