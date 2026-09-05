/* Pan and zoom for Mermaid diagrams.
 *
 * Material renders each ```mermaid fence into a <div class="mermaid"> whose
 * SVG lives in a closed shadow root, so the SVG itself is out of reach. This
 * script wraps the div in a fixed-height viewport with a toolbar and applies
 * a CSS transform to the div, which carries the shadow content with it: wheel
 * or pinch to zoom, drag to pan, double-click or the reset button to go back.
 */
(function () {
  "use strict";

  var MIN = 0.5, MAX = 8, STEP = 1.2;

  /* Material swaps the <pre class="mermaid"> for a <div class="mermaid"> only
   * once the diagram is drawn, so a div with a height is a finished render. */
  function isRendered(el) {
    return el.tagName === "DIV" && el.getBoundingClientRect().height > 0;
  }

  function enhance(el) {
    if (el.dataset.tvZoom) return;
    el.dataset.tvZoom = "1";
    var height = el.getBoundingClientRect().height;

    var viewport = document.createElement("div");
    viewport.className = "tv-zoom";
    var bar = document.createElement("div");
    bar.className = "tv-zoom__bar";
    var hint = document.createElement("span");
    hint.className = "tv-zoom__hint";
    hint.textContent = "Scroll to zoom, drag to pan, double-click to reset";
    var out = button("−", "Zoom out");
    var pct = document.createElement("span");
    pct.className = "tv-zoom__pct";
    var inn = button("+", "Zoom in");
    var reset = button("↺", "Reset zoom");
    bar.appendChild(hint); bar.appendChild(out); bar.appendChild(pct);
    bar.appendChild(inn); bar.appendChild(reset);

    var stage = document.createElement("div");
    stage.className = "tv-zoom__stage";
    stage.style.height = Math.ceil(height) + "px";

    el.parentNode.insertBefore(viewport, el);
    viewport.appendChild(bar);
    viewport.appendChild(stage);
    stage.appendChild(el);

    var scale = 1, tx = 0, ty = 0;
    el.style.transformOrigin = "0 0";
    el.style.margin = "0";
    el.style.cursor = "grab";

    function apply() {
      el.style.transform = "translate(" + tx + "px," + ty + "px) scale(" + scale + ")";
      pct.textContent = Math.round(scale * 100) + "%";
      viewport.classList.toggle("tv-zoom--active", scale !== 1 || tx !== 0 || ty !== 0);
    }

    function zoomAt(factor, cx, cy) {
      var next = Math.min(MAX, Math.max(MIN, scale * factor));
      factor = next / scale;
      tx = cx - (cx - tx) * factor;
      ty = cy - (cy - ty) * factor;
      scale = next;
      apply();
    }

    function centre() {
      var r = stage.getBoundingClientRect();
      return { x: r.width / 2, y: r.height / 2 };
    }

    function doReset() { scale = 1; tx = 0; ty = 0; apply(); }

    inn.addEventListener("click", function () { var c = centre(); zoomAt(STEP, c.x, c.y); });
    out.addEventListener("click", function () { var c = centre(); zoomAt(1 / STEP, c.x, c.y); });
    reset.addEventListener("click", doReset);
    stage.addEventListener("dblclick", function (e) { e.preventDefault(); doReset(); });

    stage.addEventListener("wheel", function (e) {
      e.preventDefault();
      var r = stage.getBoundingClientRect();
      var factor = e.ctrlKey ? Math.exp(-e.deltaY / 100) : (e.deltaY < 0 ? STEP : 1 / STEP);
      zoomAt(factor, e.clientX - r.left, e.clientY - r.top);
    }, { passive: false });

    var drag = null;
    stage.addEventListener("pointerdown", function (e) {
      if (e.button !== 0) return;
      drag = { x: e.clientX - tx, y: e.clientY - ty };
      stage.setPointerCapture(e.pointerId);
      el.style.cursor = "grabbing";
    });
    stage.addEventListener("pointermove", function (e) {
      if (!drag) return;
      tx = e.clientX - drag.x; ty = e.clientY - drag.y; apply();
    });
    function endDrag() { drag = null; el.style.cursor = "grab"; }
    stage.addEventListener("pointerup", endDrag);
    stage.addEventListener("pointercancel", endDrag);

    /* Pinch on touch screens. */
    var pinch = null;
    stage.addEventListener("touchstart", function (e) {
      if (e.touches.length === 2) pinch = dist(e.touches);
    }, { passive: true });
    stage.addEventListener("touchmove", function (e) {
      if (e.touches.length !== 2 || !pinch) return;
      e.preventDefault();
      var d = dist(e.touches), r = stage.getBoundingClientRect();
      var cx = (e.touches[0].clientX + e.touches[1].clientX) / 2 - r.left;
      var cy = (e.touches[0].clientY + e.touches[1].clientY) / 2 - r.top;
      zoomAt(d / pinch, cx, cy);
      pinch = d;
    }, { passive: false });
    stage.addEventListener("touchend", function () { pinch = null; });

    apply();
  }

  function dist(t) {
    var dx = t[0].clientX - t[1].clientX, dy = t[0].clientY - t[1].clientY;
    return Math.hypot(dx, dy);
  }

  function button(label, title) {
    var b = document.createElement("button");
    b.type = "button"; b.className = "tv-zoom__btn"; b.textContent = label; b.title = title;
    b.setAttribute("aria-label", title);
    return b;
  }

  /* Material renders Mermaid asynchronously and swaps elements as it goes, so
   * scan now, then keep watching the article for the rendered divs. */
  function scan() {
    document.querySelectorAll(".md-typeset .mermaid").forEach(function (el) {
      if (!el.dataset.tvZoom && isRendered(el)) enhance(el);
    });
  }

  var observer = null;
  function watch() {
    if (observer) observer.disconnect();
    scan();
    var root = document.querySelector(".md-content") || document.body;
    observer = new MutationObserver(function () { scan(); });
    observer.observe(root, { childList: true, subtree: true });
    /* A late layout pass can give the div its height after the mutation. */
    setTimeout(scan, 500);
    setTimeout(scan, 2000);
  }

  if (window.document$ && typeof window.document$.subscribe === "function") {
    window.document$.subscribe(watch);
  } else if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", watch);
  } else {
    watch();
  }
})();
