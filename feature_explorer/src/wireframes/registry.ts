import type { Feature } from "../state/featureMachine";
import {
  rootUnifiedLauncherRegistration,
  type RootLauncherEvent,
  type RootLauncherSnapshot
} from "./rootUnifiedLauncher";

export interface WireframeRenderInput<TSnapshot = unknown> {
  activeFeature: Feature;
  features: Feature[];
  snapshot: TSnapshot | undefined;
}

export interface WireframeRegistration<TSnapshot = unknown, TEvent = unknown> {
  id: string;
  title: string;
  featureIds: string[];
  summary: string;
  render(input: WireframeRenderInput<TSnapshot>): string;
  bind(root: ParentNode, send: (event: TEvent) => void, selectFeature: (id: string) => void): void;
}

export type WireframeEvent = RootLauncherEvent;

export const wireframeRegistrations = [rootUnifiedLauncherRegistration] as WireframeRegistration<
  RootLauncherSnapshot,
  WireframeEvent
>[];

export function wireframeForFeature(featureId: string) {
  return wireframeRegistrations.find((registration) => registration.featureIds.includes(featureId));
}

