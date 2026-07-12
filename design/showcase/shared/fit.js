/* Scale the fixed-size .scene to fill the embedding iframe exactly. */
(function () {
  "use strict";
  function fit() {
    var scene = document.querySelector(".scene");
    if (!scene) return;
    var sw = scene.offsetWidth;
    var sh = scene.offsetHeight;
    if (!sw || !sh) return;
    var scale = Math.min(window.innerWidth / sw, window.innerHeight / sh);
    scene.style.transform = "scale(" + scale + ")";
    scene.style.transformOrigin = "top left";
  }
  window.addEventListener("resize", fit);
  window.addEventListener("load", fit);
  fit();
})();
