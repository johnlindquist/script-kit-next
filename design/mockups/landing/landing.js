/**
 * Script Kit landing: IntersectionObserver autoplay for the most-visible
 * walkthrough iframe. Stories load with marketing=1&autoplay=0, then play
 * when scrolled into view.
 */
(function () {
  "use strict";

  var frames = [];
  var reduced =
    window.matchMedia &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  function frameApi(iframe) {
    try {
      return iframe.contentWindow && iframe.contentWindow.__SK_STORY__;
    } catch (_) {
      return null;
    }
  }

  function pauseAllExcept(except) {
    frames.forEach(function (iframe) {
      if (iframe === except) return;
      var api = frameApi(iframe);
      if (api && api.pause) api.pause();
    });
  }

  function playFrame(iframe) {
    if (reduced) return;
    var api = frameApi(iframe);
    if (!api) return;
    pauseAllExcept(iframe);
    if (api.restart) api.restart();
    else if (api.play) api.play();
  }

  function bindControls(section) {
    var iframe = section.querySelector("iframe[data-landing-story]");
    if (!iframe) return;
    frames.push(iframe);
    var playBtn = section.querySelector("[data-replay]");
    if (playBtn) {
      playBtn.addEventListener("click", function () {
        playFrame(iframe);
      });
    }
  }

  document.querySelectorAll("[data-feature]").forEach(bindControls);
  var hero = document.querySelector("[data-hero-story]");
  if (hero) bindControls(hero);

  if (!("IntersectionObserver" in window)) {
    // Fallback: play hero only
    if (frames[0]) {
      frames[0].addEventListener("load", function () {
        setTimeout(function () {
          playFrame(frames[0]);
        }, 400);
      });
    }
    return;
  }

  var observer = new IntersectionObserver(
    function (entries) {
      var best = null;
      var bestRatio = 0;
      entries.forEach(function (entry) {
        if (entry.isIntersecting && entry.intersectionRatio > bestRatio) {
          bestRatio = entry.intersectionRatio;
          best = entry.target;
        }
      });
      if (best && bestRatio >= 0.45) {
        var iframe = best.querySelector("iframe[data-landing-story]");
        if (iframe) playFrame(iframe);
      }
    },
    { threshold: [0.25, 0.45, 0.6, 0.8] },
  );

  document.querySelectorAll("[data-feature], [data-hero-story]").forEach(function (el) {
    observer.observe(el);
  });
})();
