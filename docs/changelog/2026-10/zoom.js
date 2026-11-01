/* Click-to-zoom glue for PhotoSwipe 5.4.4 (MIT). Content images open in a pinch/pan lightbox.
   Opt out: <img data-no-zoom>. Larger file for zoom: <img data-zoom-src="big.png">.
   Skipped automatically: images in header/nav/footer and images narrower than 160px (logos, icons). */
(function () {
  var css = 'img[data-zoomable]{cursor:zoom-in}img[data-zoomable]:focus-visible{outline:3px solid #2563eb;outline-offset:2px}';
  var st = document.createElement('style'); st.textContent = css; document.head.appendChild(st);
  var reduce = matchMedia('(prefers-reduced-motion: reduce)').matches;
  var imgs = [];
  function setup(img) {
    if (img.hasAttribute('data-no-zoom') || img.closest('header,nav,footer,a,button,[data-no-zoom]')) return;
    if (img.naturalWidth && img.naturalWidth < 160) return;
    if (img.hasAttribute('data-zoomable')) return;
    img.setAttribute('data-zoomable', ''); img.tabIndex = 0; img.setAttribute('role', 'button');
    var i = imgs.push(img) - 1;
    img.addEventListener('click', function () { open(i); });
    img.addEventListener('keydown', function (e) {
      if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); open(i); }
    });
  }
  function open(i) {
    var pswp = new PhotoSwipe({
      dataSource: imgs.map(function (im) {
        return { src: im.getAttribute('data-zoom-src') || im.currentSrc || im.src, alt: im.alt,
          width: im.naturalWidth, height: im.naturalHeight, element: im };
      }),
      index: i, bgOpacity: 0.92, wheelToZoom: true, imageClickAction: 'zoom-or-close',
      tapAction: 'toggle-controls', doubleTapAction: 'zoom', maxZoomLevel: 4,
      showHideAnimationType: reduce ? 'none' : 'zoom', zoomAnimationDuration: reduce ? 0 : 333,
      arrowKeys: imgs.length > 1, arrowPrev: imgs.length > 1, arrowNext: imgs.length > 1,
      counter: imgs.length > 1, closeOnVerticalDrag: true, returnFocus: true
    });
    pswp.addFilter('thumbEl', function (el, data) { return data.element || el; });
    pswp.addFilter('placeholderSrc', function (src, slide) { return slide.data.element ? slide.data.element.currentSrc : src; });
    pswp.on('uiRegister', function () {  // keep alt text on the large image
      pswp.on('contentActivate', function (e) { var im = e.content.element; if (im && !im.alt) im.alt = e.content.data.alt || ''; });
    });
    // Back button closes the zoom
    var popped = false;
    history.pushState({ pswp: 1 }, '');
    var onPop = function () { popped = true; pswp.close(); };
    addEventListener('popstate', onPop);
    pswp.on('destroy', function () {
      removeEventListener('popstate', onPop);
      if (!popped && history.state && history.state.pswp) history.back();
    });
    pswp.init();
  }
  function run() {
    document.querySelectorAll('img').forEach(function (img) {
      if (img.complete) setup(img); else img.addEventListener('load', function () { setup(img); }, { once: true });
    });
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', run); else run();
})();
