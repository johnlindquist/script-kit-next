import "@scriptkit/sdk";

export const metadata = {
  name: "Clipboard Transform",
  description: "Read clipboard, transform text, and copy the result",
};

const text = await paste();

if (!text.trim()) {
  await div(`<div class="p-8 text-gray-400">Clipboard is empty</div>`);
} else {
  const transform = await arg("Transform clipboard text", [
    { name: "UPPERCASE", description: "Convert to uppercase", value: "upper" },
    { name: "lowercase", description: "Convert to lowercase", value: "lower" },
    { name: "Title Case", description: "Capitalize each word", value: "title" },
    { name: "Slug", description: "Convert to url-slug-format", value: "slug" },
    { name: "Trim", description: "Remove leading/trailing whitespace", value: "trim" },
  ]);

  let result: string;
  switch (transform) {
    case "upper":
      result = text.toUpperCase();
      break;
    case "lower":
      result = text.toLowerCase();
      break;
    case "title":
      result = text.replace(/\b\w/g, (c) => c.toUpperCase());
      break;
    case "slug":
      result = text.toLowerCase().replace(/\s+/g, "-").replace(/[^a-z0-9-]/g, "");
      break;
    case "trim":
      result = text.trim();
      break;
    default:
      result = text;
  }

  await copy(result);
  await notify(`Transformed and copied!`);
}
