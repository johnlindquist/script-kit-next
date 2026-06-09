import "../../scripts/kit-sdk";

const receiptPath =
  process.env.SCRIPT_KIT_SDK_CONFIRM_RECEIPT ??
  "/tmp/confirm-modal-sdk-confirm-script-result.json";

const startedAt = new Date().toISOString();

await Bun.write(
  receiptPath,
  JSON.stringify(
    {
      kind: "sdk.confirm.runtime.script",
      phase: "started",
      startedAt,
      api: "confirm",
      protocolType: "confirm",
      message: "SDK confirm runtime proof?",
      confirmText: "Confirm SDK",
      cancelText: "Cancel SDK",
    },
    null,
    2,
  ),
);

try {
  const result = await confirm({
    message: "SDK confirm runtime proof?",
    confirmText: "Confirm SDK",
    cancelText: "Cancel SDK",
  });

  await Bun.write(
    receiptPath,
    JSON.stringify(
      {
        kind: "sdk.confirm.runtime.script",
        phase: "resolved",
        startedAt,
        resolvedAt: new Date().toISOString(),
        api: "confirm",
        protocolType: "confirm",
        result,
        resultType: typeof result,
      },
      null,
      2,
    ),
  );
} catch (error) {
  await Bun.write(
    receiptPath,
    JSON.stringify(
      {
        kind: "sdk.confirm.runtime.script",
        phase: "error",
        startedAt,
        resolvedAt: new Date().toISOString(),
        api: "confirm",
        protocolType: "confirm",
        error: error instanceof Error ? error.message : String(error),
      },
      null,
      2,
    ),
  );
  throw error;
}
