/**
 * Detect ?embed=story and mark the document before paint when possible.
 * Screen fixtures include this script so story iframes load the same HTML
 * with desktop chrome suppressed.
 */
(function () {
  "use strict";
  try {
    var params = new URLSearchParams(window.location.search || "");
    if (params.get("embed") !== "story") return;
    document.documentElement.setAttribute("data-story-embed", "true");
    document.documentElement.classList.add("story-embed");
    // Signal readiness to parent adapters.
    window.__SK_SCREEN_EMBED__ = {
      ready: true,
      screen: document.documentElement.getAttribute("data-fidelity-screen") || document.title || "",
      document: document,
    };
    // Parent may listen for load; also postMessage for clarity.
    try {
      if (window.parent && window.parent !== window) {
        window.parent.postMessage(
          { type: "sk-story-embed-ready", href: location.href },
          "*",
        );
      }
    } catch (_) {}
  } catch (_) {}
})();
