import "@scriptkit/sdk";

export const metadata = {
  name: "Choose From List",
  description: "Rich choices with descriptions and preview panels",
};

const colors = [
  {
    name: "Amber",
    description: "Warm golden yellow — Script Kit's signature",
    value: "#fbbf24",
    preview: `<div class="h-full" style="background: #fbbf24"></div>`,
  },
  {
    name: "Sky Blue",
    description: "Calm and professional",
    value: "#38bdf8",
    preview: `<div class="h-full" style="background: #38bdf8"></div>`,
  },
  {
    name: "Emerald",
    description: "Fresh and natural",
    value: "#34d399",
    preview: `<div class="h-full" style="background: #34d399"></div>`,
  },
  {
    name: "Rose",
    description: "Bold and energetic",
    value: "#fb7185",
    preview: `<div class="h-full" style="background: #fb7185"></div>`,
  },
];

const picked = await arg("Pick a color", colors);

await div(`
  <div class="flex flex-col items-center justify-center h-full p-8">
    <div class="w-32 h-32 rounded-2xl mb-6" style="background: ${picked}"></div>
    <p class="text-gray-300 text-lg">You chose <span class="font-bold" style="color: ${picked}">${picked}</span></p>
  </div>
`);
