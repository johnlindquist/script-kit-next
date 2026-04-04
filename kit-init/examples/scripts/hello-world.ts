import "@scriptkit/sdk";

export const metadata = {
  name: "Hello World",
  description: "Verification-friendly starter script",
};

const isVerify = process.env.SK_VERIFY === "1";

const name = isVerify
  ? "verification"
  : await arg("Who should I greet?");

const greeting = `Hello, ${name}!`;

if (isVerify) {
  console.log(JSON.stringify({ ok: true, greeting }));
} else {
  await div(`<div class="p-8 text-2xl">${greeting}</div>`);
}
