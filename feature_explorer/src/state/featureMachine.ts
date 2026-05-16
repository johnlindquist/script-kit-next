import { assign, setup } from "xstate";
import data from "../data/features.generated.json";

export interface AtlasCoverage {
  indexedFeatureCount: number;
  rawOracleFeatureCount: number;
  chapterFeatureCount: number;
  pendingIndexRows: { id: string; feature: string; cluster: string; owner: string }[];
  pendingRawOracleRows: { id: string; slug: string }[];
}

export const atlasCoverage = data.coverage as AtlasCoverage;

export interface Feature {
  id: string;
  slug: string;
  file: string;
  title: string;
  summary: string;
  capabilities: string[];
  concepts: Record<string, string>[];
  entryPoints: Record<string, string>[];
  workflows: { title: string; body: string }[];
  interactions: Record<string, string>[];
  stateRows: Record<string, string>[];
  keystrokes: Record<string, string>[];
  visualStates: string[];
  risks: string[];
  gaps: string[];
  sections: Record<string, string>;
  tables?: Record<string, Record<string, string>[]>;
}
export type ExplorerMode = "overview" | "machine" | "wireframe" | "states" | "workflows" | "interactions" | "keystrokes" | "risks";

export interface FeatureExplorerContext {
  features: Feature[];
  selectedFeatureId: string;
  selectedState: string;
  selectedWorkflow: string;
  selectedInteraction: string;
  filter: string;
  mode: ExplorerMode;
}

export type FeatureExplorerEvent =
  | { type: "FILTER"; value: string }
  | { type: "SELECT_FEATURE"; id: string }
  | { type: "SELECT_STATE"; id: string }
  | { type: "SELECT_WORKFLOW"; id: string }
  | { type: "SELECT_INTERACTION"; id: string }
  | { type: "SET_MODE"; mode: ExplorerMode }
  | { type: "NEXT_FEATURE" }
  | { type: "PREV_FEATURE" };

const firstFeature = data.features[0];
const features = data.features as Feature[];

function nextFeatureId(context: FeatureExplorerContext, direction: 1 | -1) {
  const index = context.features.findIndex((feature) => feature.id === context.selectedFeatureId);
  const nextIndex = (index + direction + context.features.length) % context.features.length;
  return context.features[nextIndex]?.id ?? context.selectedFeatureId;
}

function resetSelection(feature: Feature) {
  return {
    selectedState: feature.stateRows[0]?.State ?? "",
    selectedWorkflow: feature.workflows[0]?.title ?? "",
    selectedInteraction: interactionLabel(feature.interactions[0])
  };
}

export function interactionLabel(row: Record<string, string> | undefined) {
  if (!row) return "";
  return row["User intent"] ?? row.Interaction ?? Object.values(row)[0] ?? "";
}

export const featureExplorerMachine = setup({
  types: {
    context: {} as FeatureExplorerContext,
    events: {} as FeatureExplorerEvent
  }
}).createMachine({
  id: "featureExplorer",
  initial: "exploring",
  context: {
    features,
    selectedFeatureId: firstFeature?.id ?? "",
    ...resetSelection(firstFeature as Feature),
    filter: "",
    mode: "overview"
  },
  states: {
    exploring: {
      on: {
        FILTER: {
          actions: assign({
            filter: ({ event }) => event.value
          })
        },
        SELECT_FEATURE: {
          actions: assign(({ context, event }) => {
            const feature = context.features.find((candidate) => candidate.id === event.id) ?? context.features[0];
            return {
              selectedFeatureId: feature.id,
              ...resetSelection(feature)
            };
          })
        },
        NEXT_FEATURE: {
          actions: assign(({ context }) => {
            const id = nextFeatureId(context, 1);
            const feature = context.features.find((candidate) => candidate.id === id) ?? context.features[0];
            return {
              selectedFeatureId: feature.id,
              ...resetSelection(feature)
            };
          })
        },
        PREV_FEATURE: {
          actions: assign(({ context }) => {
            const id = nextFeatureId(context, -1);
            const feature = context.features.find((candidate) => candidate.id === id) ?? context.features[0];
            return {
              selectedFeatureId: feature.id,
              ...resetSelection(feature)
            };
          })
        },
        SELECT_STATE: {
          actions: assign({
            selectedState: ({ event }) => event.id,
            mode: "states"
          })
        },
        SELECT_WORKFLOW: {
          actions: assign({
            selectedWorkflow: ({ event }) => event.id,
            mode: "workflows"
          })
        },
        SELECT_INTERACTION: {
          actions: assign({
            selectedInteraction: ({ event }) => event.id,
            mode: "interactions"
          })
        },
        SET_MODE: {
          actions: assign({
            mode: ({ event }) => event.mode
          })
        }
      }
    }
  }
});

export function selectedFeature(context: FeatureExplorerContext) {
  return context.features.find((feature) => feature.id === context.selectedFeatureId) ?? context.features[0];
}

export function filteredFeatures(context: FeatureExplorerContext) {
  const query = context.filter.trim().toLowerCase();
  if (!query) return context.features;
  return context.features.filter((feature) =>
    [feature.id, feature.title, feature.summary, feature.file, ...feature.capabilities]
      .join(" ")
      .toLowerCase()
      .includes(query)
  );
}
