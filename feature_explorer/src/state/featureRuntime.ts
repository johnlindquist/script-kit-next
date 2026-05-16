import { createMachine } from "xstate";
import { authoredFeatureMachines, type AuthoredFeatureMachineConfig, type AuthoredStateNode } from "./authoredFeatureMachines";
import { type Feature, interactionLabel } from "./featureMachine";

export interface FeatureRuntimeEvent {
  id: string;
  label: string;
  source: "interaction" | "keystroke";
  target?: string;
  detail: Record<string, string>;
}

export interface FeatureRuntimeTransition {
  id: string;
  label: string;
  source: "state";
  from: string;
  target: string;
  detail: Record<string, string>;
}

export interface FeatureRuntimeModel {
  initial: string;
  states: { id: string; label: string; row: Record<string, string> }[];
  events: FeatureRuntimeEvent[];
  transitions: FeatureRuntimeTransition[];
  authored?: AuthoredFeatureMachineConfig;
  coverage: {
    stateCount: number;
    interactionEventCount: number;
    keystrokeEventCount: number;
    explicitTransitionCount: number;
    inferredEventTargetCount: number;
    fallbackEventCount: number;
  };
}

function authoredForFeature(feature: Feature) {
  return authoredFeatureMachines[feature.id as keyof typeof authoredFeatureMachines] ?? authoredFeatureMachines[feature.slug as keyof typeof authoredFeatureMachines];
}

function slug(value: string, fallback: string) {
  const normalized = value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "");
  return normalized || fallback;
}

function splitTargets(value: string | undefined) {
  if (!value) return [];
  return value
    .split(/,|\bor\b|;|\//i)
    .map((part) => part.trim())
    .filter((part) => part && !/^none$/i.test(part));
}

function words(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .split(/\s+/)
    .filter((word) => word.length > 2);
}

function targetForText(text: string, states: FeatureRuntimeModel["states"]) {
  const normalizedText = text.toLowerCase();
  let best: { state: FeatureRuntimeModel["states"][number]; score: number } | undefined;

  for (const state of states) {
    const label = state.label.toLowerCase();
    const stateWords = words(state.label);
    let score = 0;
    if (normalizedText === label) score += 100;
    if (normalizedText.includes(label) || label.includes(normalizedText)) score += 50;
    score += stateWords.filter((word) => normalizedText.includes(word)).length * 10;
    if (!best || score > best.score) best = { state, score };
  }

  return best && best.score >= 20 ? best.state.id : undefined;
}

function nextStateId(states: FeatureRuntimeModel["states"], stateId: string) {
  const index = states.findIndex((state) => state.id === stateId);
  return states[(index + 1 + states.length) % states.length]?.id ?? stateId;
}

export function runtimeModelForFeature(feature: Feature): FeatureRuntimeModel {
  const authored = authoredForFeature(feature);
  if (authored) return runtimeModelFromAuthored(authored);

  const states = feature.stateRows.length
    ? feature.stateRows.map((row, index) => ({
        id: slug(row.State ?? Object.values(row)[0] ?? "", `state_${index + 1}`),
        label: row.State ?? Object.values(row)[0] ?? `State ${index + 1}`,
        row
      }))
    : [
        {
          id: "overview",
          label: "Overview",
          row: { State: "Overview", Guards: "No State Machine table was found in this chapter." }
        }
      ];

  const transitions: FeatureRuntimeTransition[] = [];
  states.forEach((state) => {
    splitTargets(state.row["Exits to"] ?? state.row.Exits).forEach((targetLabel, index) => {
      const target = targetForText(targetLabel, states);
      if (!target || target === state.id) return;
      transitions.push({
        id: `transition_${state.id}_${target}_${index + 1}`,
        label: `${state.label} -> ${states.find((candidate) => candidate.id === target)?.label ?? targetLabel}`,
        source: "state",
        from: state.id,
        target,
        detail: {
          from: state.label,
          target: states.find((candidate) => candidate.id === target)?.label ?? targetLabel,
          guard: state.row.Guards ?? "",
          rawExit: targetLabel
        }
      });
    });
  });

  const interactionEvents = feature.interactions.map((row, index) => ({
    id: `interaction_${slug(interactionLabel(row), String(index + 1))}`,
    label: interactionLabel(row) || `Interaction ${index + 1}`,
    source: "interaction" as const,
    target: targetForText([row["UI state"], row.Result, row["User intent"], row["Entry point"]].filter(Boolean).join(" "), states),
    detail: row
  }));

  const keyEvents = feature.keystrokes.map((row, index) => {
    const label = row.Key ?? row.Shortcut ?? row["Key/click"] ?? row.Context ?? Object.values(row)[0] ?? `Key ${index + 1}`;
    return {
      id: `key_${slug(label, String(index + 1))}`,
      label,
      source: "keystroke" as const,
      target: targetForText([row.Context, row.Behavior, label].filter(Boolean).join(" "), states),
      detail: row
    };
  });

  const events = [...interactionEvents, ...keyEvents];
  const inferredEventTargetCount = events.filter((event) => event.target).length;

  return {
    initial: states[0].id,
    states,
    events,
    transitions,
    coverage: {
      stateCount: states.length,
      interactionEventCount: interactionEvents.length,
      keystrokeEventCount: keyEvents.length,
      explicitTransitionCount: transitions.length,
      inferredEventTargetCount,
      fallbackEventCount: events.length - inferredEventTargetCount
    }
  };
}

function runtimeModelFromAuthored(authored: AuthoredFeatureMachineConfig): FeatureRuntimeModel {
  const states: FeatureRuntimeModel["states"] = [];
  const events = new Map<string, FeatureRuntimeEvent>();
  const transitions: FeatureRuntimeTransition[] = [];

  function visit(name: string, node: AuthoredStateNode, path: string[]) {
    const id = [...path, name].join(".");
    const wireframe = node.meta?.wireframe;
    states.push({
      id,
      label: titleFromStateName(name),
      row: {
        State: titleFromStateName(name),
        Visible: wireframe?.visibleRegions.join(", ") ?? "",
        Selected: wireframe?.selected ?? "",
        Input: wireframe?.inputText ?? "",
        Popup: wireframe?.activePopup ?? "none",
        Portal: wireframe?.activePortal ?? "none",
        Footer: wireframe?.footerOwner ?? "",
        Status: wireframe?.status ?? "",
        Proof: wireframe?.proof.join(", ") ?? ""
      }
    });

    Object.entries(node.on ?? {}).forEach(([eventName, transitionValue]) => {
      const transition = typeof transitionValue === "string" ? { target: transitionValue } : transitionValue;
      const target = transition.target ? targetIdForAuthoredTarget(transition.target) : undefined;
      events.set(eventName, {
        id: eventName,
        label: eventName,
        source: "interaction",
        target,
        detail: {
          Event: eventName,
          Target: target ?? "",
          Guard: transition.guard ?? "",
          Actions: Array.isArray(transition.actions) ? transition.actions.join(", ") : transition.actions ?? ""
        }
      });
      if (target) {
        transitions.push({
          id: eventName,
          label: `${titleFromStateName(name)} -> ${stateLabelFromId(target)}`,
          source: "state",
          from: id,
          target,
          detail: {
            from: titleFromStateName(name),
            target: stateLabelFromId(target),
            guard: transition.guard ?? "",
            rawExit: eventName
          }
        });
      }
    });

    Object.entries(node.states ?? {}).forEach(([childName, childNode]) => visit(childName, childNode, [...path, name]));
  }

  Object.entries(authored.states).forEach(([name, node]) => visit(name, node, []));
  const initial = targetIdForAuthoredTarget(authored.initial) ?? states[0]?.id ?? "overview";
  const eventList = [...events.values()];

  return {
    initial,
    states,
    events: eventList,
    transitions,
    authored,
    coverage: {
      stateCount: states.length,
      interactionEventCount: eventList.length,
      keystrokeEventCount: 0,
      explicitTransitionCount: transitions.length,
      inferredEventTargetCount: eventList.filter((event) => event.target).length,
      fallbackEventCount: eventList.filter((event) => !event.target).length
    }
  };
}

export function createFeatureRuntimeMachine(feature: Feature) {
  const model = runtimeModelForFeature(feature);
  const stateMap = Object.fromEntries(
    model.states.map((state) => {
      const fallback = nextStateId(model.states, state.id);
      const stateTransitions = model.transitions.filter((transition) => transition.from === state.id);
      return [
        state.id,
        {
          on: Object.fromEntries([
            ...stateTransitions.map((transition) => [transition.id, { target: transition.target }]),
            ...model.events.map((event) => [event.id, { target: event.target ?? fallback }])
          ])
        }
      ];
    })
  );

  return createMachine({
    id: `feature_${feature.id}`,
    initial: model.initial,
    context: {
      featureId: feature.id,
      featureTitle: feature.title
    },
    states: stateMap
  });
}

function targetIdForAuthoredTarget(target: string | string[] | undefined) {
  if (!target) return undefined;
  const value = Array.isArray(target) ? target[0] : target;
  return value.replace(/^#/, "").replace(/^\./, "").replace(/\//g, ".");
}

function titleFromStateName(value: string) {
  return value
    .replace(/[_-]+/g, " ")
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/\b\w/g, (char) => char.toUpperCase());
}

function stateLabelFromId(value: string) {
  return titleFromStateName(value.split(".").at(-1) ?? value);
}
