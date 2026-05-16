import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";

const repoRoot = resolve("..");
const featuresDir = join(repoRoot, "feature-map", "features");
const rawOracleDir = join(repoRoot, "feature-map", "raw-oracle");
const indexPath = join(repoRoot, "feature-map", "index.md");
const outPath = resolve("src", "data", "features.generated.json");

const wantedSections = [
  "Executive Summary",
  "What Users Can Do",
  "Human Capabilities",
  "Core Concepts",
  "Entry Points",
  "Source Head Matrix",
  "User Workflows",
  "Interaction Matrix",
  "State Machine",
  "Visual And Focus States",
  "Keystrokes And Commands",
  "Actions And Menus",
  "Automation And Protocol Surface",
  "Data, Storage, And Privacy Boundaries",
  "Error, Empty, Loading, And Disabled States",
  "Code Ownership",
  "Invariants And Regression Risks",
  "Verification Recipes",
  "Open Questions And Gaps"
];

function sectionBody(markdown, heading) {
  const lines = markdown.replace(/\r/g, "").split("\n");
  const start = lines.findIndex((line) => line.trim() === `## ${heading}`);
  if (start === -1) return "";
  const end = lines.findIndex((line, index) => index > start && /^##\s+/.test(line));
  return lines.slice(start + 1, end === -1 ? undefined : end).join("\n").trim();
}

function childSections(markdown, heading) {
  const body = sectionBody(markdown, heading);
  if (!body) return [];
  const result = [];
  const re = /^### (.+?)\s*$([\s\S]*?)(?=^### |$(?![\s\S]))/gm;
  for (const match of body.matchAll(re)) {
    result.push({
      title: match[1].trim(),
      body: compactMarkdown(match[2])
    });
  }
  return result;
}

function compactMarkdown(markdown) {
  return markdown
    .replace(/\r/g, "")
    .replace(/[ \t]+$/gm, "")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function listItems(markdown) {
  return markdown
    .split("\n")
    .map((line) => line.match(/^- (.+)$/)?.[1]?.trim())
    .filter(Boolean);
}

function parseTable(markdown) {
  const lines = markdown
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.startsWith("|") && line.endsWith("|"));
  if (lines.length < 2) return [];
  const header = splitRow(lines[0]);
  return lines.slice(2).map((line) => {
    const cells = splitRow(line);
    return Object.fromEntries(header.map((key, index) => [key, cells[index] ?? ""]));
  });
}

function splitRow(line) {
  return line
    .slice(1, -1)
    .split("|")
    .map((cell) => cell.trim().replace(/`/g, ""));
}

function proseParagraph(markdown) {
  return markdown
    .split(/\n{2,}/)
    .find((paragraph) => paragraph.trim() && !paragraph.trim().startsWith("|"))
    ?.replace(/\n/g, " ")
    .trim() ?? "";
}

function featureFromFile(file) {
  const markdown = readFileSync(join(featuresDir, file), "utf8");
  const title = markdown.match(/^# (.+)$/m)?.[1]?.trim() ?? basename(file, ".md");
  const id = file.match(/^(\d+)/)?.[1] ?? basename(file, ".md");
  const sections = Object.fromEntries(
    wantedSections.map((section) => [section, compactMarkdown(sectionBody(markdown, section))])
  );
  const workflows = childSections(markdown, "User Workflows");
  const stateRows = parseTable(sections["State Machine"]);
  const interactions = parseTable(sections["Interaction Matrix"]);
  const keystrokes = parseTable(sections["Keystrokes And Commands"]);
  const entryPoints = parseTable(sections["Entry Points"]);
  const concepts = parseTable(sections["Core Concepts"]);
  const capabilities = listItems(sections["What Users Can Do"]);
  const visualStates = listItems(sections["Visual And Focus States"]);
  const risks = listItems(sections["Invariants And Regression Risks"]);
  const gaps = listItems(sections["Open Questions And Gaps"]);
  const tables = Object.fromEntries(
    wantedSections.map((section) => [section, parseTable(sections[section])])
  );

  return {
    id,
    slug: basename(file, ".md"),
    file: `feature-map/features/${file}`,
    title,
    summary: proseParagraph(sections["Executive Summary"] || markdown),
    capabilities,
    concepts,
    entryPoints,
    workflows,
    interactions,
    stateRows,
    keystrokes,
    visualStates,
    risks,
    gaps,
    sections,
    tables
  };
}

const features = readdirSync(featuresDir)
  .filter((file) => /^\d+-.+\.md$/.test(file))
  .sort()
  .map(featureFromFile);

const indexMarkdown = readFileSync(indexPath, "utf8");
const indexedRows = indexMarkdown
  .split("\n")
  .filter((line) => /^\|\s*\d+\s*\|/.test(line))
  .map((line) => {
    const cells = splitRow(line);
    return { id: cells[0], feature: cells[1], cluster: cells[2], owner: cells[4] };
  });
const indexedIds = new Set(indexedRows.map((row) => row.id));
const chapterIds = new Set(features.map((feature) => feature.id));
const pendingIndexRows = indexedRows.filter((row) => !chapterIds.has(row.id));
const rawOracleDirs = readdirSync(rawOracleDir, { withFileTypes: true })
  .filter((entry) => entry.isDirectory() && /^\d+-/.test(entry.name))
  .map((entry) => ({
    id: entry.name.match(/^(\d+)/)?.[1] ?? entry.name,
    slug: entry.name,
    hasAnswer: existsSync(join(rawOracleDir, entry.name, "answer.md"))
  }))
  .sort((a, b) => a.slug.localeCompare(b.slug));
const rawOracleRows = rawOracleDirs.filter((row) => row.hasAnswer);
const rawOracleIds = new Set(rawOracleRows.map((row) => row.id));
const pendingRawOracleRows = rawOracleRows.filter((row) => !chapterIds.has(row.id));
const incompleteRawOracleRows = rawOracleDirs.filter((row) => !row.hasAnswer);

writeFileSync(
  outPath,
  `${JSON.stringify(
    {
      generatedAt: new Date().toISOString(),
      source: "feature-map/features/*.md",
      featureCount: features.length,
      coverage: {
        indexedFeatureCount: indexedIds.size,
        rawOracleFeatureCount: rawOracleIds.size,
        chapterFeatureCount: chapterIds.size,
        pendingIndexRows,
        pendingRawOracleRows,
        incompleteRawOracleRows
      },
      features
    },
    null,
    2
  )}\n`
);

console.log(`Generated ${features.length} features -> ${outPath}`);
