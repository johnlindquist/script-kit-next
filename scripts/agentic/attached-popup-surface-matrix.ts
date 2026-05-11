#!/usr/bin/env bun
/**
 * State-first verification matrix for attached popup surfaces.
 *
 * Attached popups are rendered inside a parent native window. Screenshot proofs
 * must capture the parent window and preserve the popup crop bounds from
 * inspectAutomationWindow.
 */

import { type JsonObject, type MatrixAutomationTarget } from "./filterable-surface-matrix";

export type AttachedPopupKind = "ActionsDialog" | "PromptPopup";

export type AttachedPopupHostFixture =
  | {
      kind: "filterable-main";
      caseId: string;
    }
  | {
      kind: "acp-chat";
      trigger: "slash";
    };

export interface AttachedPopupSurfaceEntry {
  id: string;
  surfaceClass: "attachedPopup";
  viewName: string;
  imageLibraryName: string;
  windowKind: AttachedPopupKind;
  targetKind: "actionsDialog" | "promptPopup";
  targetIndex: 0;
  target: MatrixAutomationTarget;
  expectedAutomationWindowId?: string;
  hostFixture?: AttachedPopupHostFixture;
  expectedPopupCaptureStrategy: "parent_capture_with_crop";
  safeInteractions: {
    filter: false;
    selectFirstVisibleChoice: false;
    submit: false;
  };
}

const ACTIONS_DIALOG_TARGET: MatrixAutomationTarget = {
  type: "kind",
  kind: "actionsDialog",
  index: 0,
};

const PROMPT_POPUP_TARGET: MatrixAutomationTarget = {
  type: "kind",
  kind: "promptPopup",
  index: 0,
};

export const ATTACHED_POPUP_SURFACE_MATRIX: AttachedPopupSurfaceEntry[] = [
  {
    id: "actions-dialog-attached-popup",
    surfaceClass: "attachedPopup",
    viewName: "actions-dialog",
    imageLibraryName: "actions-dialog.png",
    windowKind: "ActionsDialog",
    targetKind: "actionsDialog",
    targetIndex: 0,
    target: ACTIONS_DIALOG_TARGET,
    expectedPopupCaptureStrategy: "parent_capture_with_crop",
    safeInteractions: {
      filter: false,
      selectFirstVisibleChoice: false,
      submit: false,
    },
  },
  // Hosted Actions Dialog cases must target live Cmd+K hosts. Stable
  // filterable surfaces that are generic built-in list views remain
  // main-surface-only until product behavior adds selection-specific shared
  // actions support. Do not add candidate entries for Current App Commands,
  // Design Gallery, or Process Manager.
  {
    id: "actions-dialog-on-clipboard-history",
    surfaceClass: "attachedPopup",
    viewName: "actions-dialog-on-clipboard-history",
    imageLibraryName: "actions-dialog-on-clipboard-history.png",
    windowKind: "ActionsDialog",
    targetKind: "actionsDialog",
    targetIndex: 0,
    target: ACTIONS_DIALOG_TARGET,
    hostFixture: { kind: "filterable-main", caseId: "clipboard-history-visible-rows" },
    expectedPopupCaptureStrategy: "parent_capture_with_crop",
    safeInteractions: {
      filter: false,
      selectFirstVisibleChoice: false,
      submit: false,
    },
  },
  {
    id: "actions-dialog-on-emoji-picker",
    surfaceClass: "attachedPopup",
    viewName: "actions-dialog-on-emoji-picker",
    imageLibraryName: "actions-dialog-on-emoji-picker.png",
    windowKind: "ActionsDialog",
    targetKind: "actionsDialog",
    targetIndex: 0,
    target: ACTIONS_DIALOG_TARGET,
    hostFixture: { kind: "filterable-main", caseId: "emoji-picker-visible-rows" },
    expectedPopupCaptureStrategy: "parent_capture_with_crop",
    safeInteractions: {
      filter: false,
      selectFirstVisibleChoice: false,
      submit: false,
    },
  },
  {
    id: "actions-dialog-on-app-launcher",
    surfaceClass: "attachedPopup",
    viewName: "actions-dialog-on-app-launcher",
    imageLibraryName: "actions-dialog-on-app-launcher.png",
    windowKind: "ActionsDialog",
    targetKind: "actionsDialog",
    targetIndex: 0,
    target: ACTIONS_DIALOG_TARGET,
    hostFixture: { kind: "filterable-main", caseId: "app-launcher-visible-rows" },
    expectedPopupCaptureStrategy: "parent_capture_with_crop",
    safeInteractions: {
      filter: false,
      selectFirstVisibleChoice: false,
      submit: false,
    },
  },
  {
    id: "prompt-popup-on-acp-chat-slash",
    surfaceClass: "attachedPopup",
    viewName: "prompt-popup-on-acp-chat-slash",
    imageLibraryName: "prompt-popup-on-acp-chat-slash.png",
    windowKind: "PromptPopup",
    targetKind: "promptPopup",
    targetIndex: 0,
    target: PROMPT_POPUP_TARGET,
    expectedAutomationWindowId: "acp-mention-popup",
    hostFixture: { kind: "acp-chat", trigger: "slash" },
    expectedPopupCaptureStrategy: "parent_capture_with_crop",
    safeInteractions: {
      filter: false,
      selectFirstVisibleChoice: false,
      submit: false,
    },
  },
];

export function selectedAttachedPopupCases(caseId: string): AttachedPopupSurfaceEntry[] {
  if (caseId === "all") {
    return ATTACHED_POPUP_SURFACE_MATRIX;
  }
  const entry = ATTACHED_POPUP_SURFACE_MATRIX.find((candidate) => candidate.id === caseId);
  if (!entry) {
    throw new Error(`Unknown attached popup surface matrix case: ${caseId}`);
  }
  return [entry];
}

async function main(): Promise<void> {
  if (process.argv.includes("--list")) {
    process.stdout.write(
      `${JSON.stringify({
        schemaVersion: 1,
        status: "pass",
        matrix: ATTACHED_POPUP_SURFACE_MATRIX as JsonObject[],
      })}\n`,
    );
  }
}

if (import.meta.main) {
  await main();
}
