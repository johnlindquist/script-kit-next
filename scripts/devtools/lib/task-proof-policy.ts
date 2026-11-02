import type { EvidenceClass } from "./evidence-class.ts";

export type TaskProofRequirement =
  | "static-inventory"
  | "unit-behavior"
  | "fixture-contract"
  | "direct-runtime";

export interface TaskProofPolicy {
  taskId: string;
  requirement: TaskProofRequirement;
  acceptedEvidenceClasses: readonly EvidenceClass[];
  provesRuntimeBehavior: boolean;
}

// GOV-003 compares the two shipped quantizers, not copies in a fixture. Bind
// the dependency, implementations, compatibility exports and compiler owner.
export const AUTHORED_ALPHA_PROOF_SOURCES: readonly string[] = [
  "Cargo.toml",
  "Cargo.lock",
  "crates/sk-color/Cargo.toml",
  "crates/sk-color/src/lib.rs",
  "crates/sk-color/src/alpha.rs",
  "src/theme/alpha.rs",
  "src/theme/types.rs",
  "scripts/agentic/agent-cargo.sh",
  "scripts/agentic/build-artifact.ts",
  "rust-toolchain.toml",
];

const runtimeClasses = [
  "RUNTIME_HIDDEN",
  "RUNTIME_VISIBLE",
  "PACKAGED_APP",
  "DIRECT_RUNTIME_PROOF",
] as const satisfies readonly EvidenceClass[];

function policy(taskId: string): TaskProofPolicy {
  if (taskId === "RPT-001" || taskId === "PF-009") {
    return {
      taskId,
      requirement: "static-inventory",
      acceptedEvidenceClasses: ["STATIC_INVENTORY", "UNIT_BEHAVIOR"],
      provesRuntimeBehavior: false,
    };
  }
  if (taskId === "PF-010") {
    return {
      taskId,
      requirement: "fixture-contract",
      acceptedEvidenceClasses: ["FIXTURE_CONTRACT", "UNIT_BEHAVIOR", ...runtimeClasses],
      provesRuntimeBehavior: false,
    };
  }
  if (
    taskId.startsWith("GOV-") ||
    ["PF-001", "PF-002", "PF-003", "PF-011", "GEO-001"].includes(taskId)
  ) {
    return {
      taskId,
      requirement: "unit-behavior",
      acceptedEvidenceClasses: ["UNIT_BEHAVIOR", ...runtimeClasses],
      provesRuntimeBehavior: false,
    };
  }
  return {
    taskId,
    requirement: "direct-runtime",
    acceptedEvidenceClasses: runtimeClasses,
    provesRuntimeBehavior: true,
  };
}

function range(prefix: string, total: number): string[] {
  return Array.from(
    { length: total },
    (_, index) => `${prefix}-${String(index + 1).padStart(3, "0")}`,
  );
}

export const TASK_PROOF_POLICIES: ReadonlyMap<string, TaskProofPolicy> = new Map(
  [
    "RPT-001",
    ...range("SAFE", 4),
    ...range("PF", 12),
    ...range("UX", 18),
    ...range("WF", 24),
    ...range("GEO", 9),
    ...range("GOV", 7),
  ].map((taskId) => [taskId, policy(taskId)]),
);

export function taskProofPolicy(taskId: string): TaskProofPolicy | null {
  return TASK_PROOF_POLICIES.get(taskId) ?? null;
}
