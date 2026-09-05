/* Full-screen pan and zoom for Mermaid diagrams.
 *
 * Material renders each ```mermaid fence into a <div class="mermaid"> whose
 * SVG lives in a closed shadow root, so the SVG itself is out of reach. Moving
 * the host div keeps its shadow content, and a CSS transform on the div scales
 * it. Clicking a diagram (or its corner button) moves it into a <dialog> that
 * covers the viewport: wheel or pinch to zoom around the cursor, drag to pan,
 * double-click or the reset button to fit, Escape or the close button to
 * return the diagram to the page. The dialog is opened with show() rather
 * than showModal(): a fixed, full-viewport element needs no top layer, and
 * staying out of it keeps focus and pointer handling ordinary.
 */
(function () {
  "use strict";

  var MIN = 0.25, MAX = 10, STEP = 1.2;

  /* Material swaps the <pre class="mermaid"> for a <div class="mermaid"> only
   * once the diagram is drawn, so a div with a height is a finished render. */
  function isRendered(el) {
    return el.tagName === "DIV" && el.getBoundingClientRect().height > 0;
  }

  function button(label, title, cls) {
    var b = document.createElement("button");
    b.type = "button";
    b.className = "tv-zoom__btn" + (cls ? " " + cls : "");
    b.textContent = label;
    b.title = title;
    b.setAttribute("aria-label", title);
    return b;
  }

  function dist(t) {
    return Math.hypot(t[0].clientX - t[1].clientX, t[0].clientY - t[1].clientY);
  }

  /* One dialog is shared by every diagram on the page. */
  var dialog = null, stage = null, pct = null, current = null;
  var scale = 1, tx = 0, ty = 0;

  function ensureDialog() {
    if (dialog) return;
    dialog = document.createElement("dialog");
    dialog.className = "tv-zoom-dialog";
    dialog.setAttribute("aria-label", "Diagram viewer");

    var bar = document.createElement("div");
    bar.className = "tv-zoom-dialog__bar";
    var hint = document.createElement("span");
    hint.className = "tv-zoom-dialog__hint";
    hint.textContent = "Scroll or pinch to zoom, drag to pan, double-click to fit";
    var out = button("−", "Zoom out");
    pct = document.createElement("span");
    pct.className = "tv-zoom-dialog__pct";
    var inn = button("+", "Zoom in");
    var reset = button("⤢", "Fit to window");
    var close = button("✕", "Close", "tv-zoom__btn--close");
    bar.appendChild(hint); bar.appendChild(out); bar.appendChild(pct);
    bar.appendChild(inn); bar.appendChild(reset); bar.appendChild(close);

    stage = document.createElement("div");
    stage.className = "tv-zoom-dialog__stage";

    dialog.appendChild(bar);
    dialog.appendChild(stage);
    document.body.appendChild(dialog);

    function apply() {
      if (!current) return;
      current.style.transform = "translate(" + tx + "px," + ty + "px) scale(" + scale + ")";
      pct.textContent = Math.round(scale * 100) + "%";
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

    /* Fit the diagram inside the stage and centre it. The div takes the
     * stage width on its own; only a diagram taller than the stage needs
     * shrinking. */
    function fit() {
      if (!current) return;
      current.style.transform = "none";
      var s = stage.getBoundingClientRect();
      var d = current.getBoundingClientRect();
      scale = Math.min(1, s.height / d.height, s.width / d.width);
      tx = (s.width - d.width * scale) / 2;
      ty = (s.height - d.height * scale) / 2;
      apply();
    }
    dialog.tvFit = fit;

    inn.addEventListener("click", function () { var c = centre(); zoomAt(STEP, c.x, c.y); });
    out.addEventListener("click", function () { var c = centre(); zoomAt(1 / STEP, c.x, c.y); });
    reset.addEventListener("click", fit);
    stage.addEventListener("dblclick", function (e) { e.preventDefault(); fit(); });

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
      stage.classList.add("tv-zoom-dialog__stage--dragging");
    });
    stage.addEventListener("pointermove", function (e) {
      if (!drag) return;
      tx = e.clientX - drag.x; ty = e.clientY - drag.y; apply();
    });
    function endDrag() { drag = null; stage.classList.remove("tv-zoom-dialog__stage--dragging"); }
    stage.addEventListener("pointerup", endDrag);
    stage.addEventListener("pointercancel", endDrag);

    var pinch = null;
    stage.addEventListener("touchstart", function (e) {
      if (e.touches.length === 2) pinch = dist(e.touches);
    }, { passive: true });
    stage.addEventListener("touchmove", function (e) {
      if (e.touches.length !== 2 || !pinch) return;
      e.preventDefault();
      var d = dist(e.touches), r = stage.getBoundingClientRect();
      zoomAt(d / pinch,
        (e.touches[0].clientX + e.touches[1].clientX) / 2 - r.left,
        (e.touches[0].clientY + e.touches[1].clientY) / 2 - r.top);
      pinch = d;
    }, { passive: false });
    stage.addEventListener("touchend", function () { pinch = null; });

    /* Escape and the close button both end here: put the diagram back where
     * it came from. Done by hand rather than from the dialog's close event,
     * which does not fire reliably for a dialog opened with show(). */
    dialog.tvClose = function () {
      if (current && current.tvHome) {
        current.style.transform = "";
        current.style.transformOrigin = "";
        current.tvHome.appendChild(current);
      }
      current = null;
      document.documentElement.classList.remove("tv-zoom-open");
      if (dialog.open) dialog.close();
      if (dialog.tvOpener && dialog.tvOpener.focus) dialog.tvOpener.focus();
    };
    close.addEventListener("click", dialog.tvClose);
    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape" && dialog.open) { e.preventDefault(); dialog.tvClose(); }
    });
    window.addEventListener("resize", function () { if (dialog.open) fit(); });
  }

  function open(el, opener) {
    ensureDialog();
    if (dialog.open) return;
    current = el;
    el.tvHome = el.parentNode;
    el.style.transformOrigin = "0 0";
    stage.appendChild(el);
    document.documentElement.classList.add("tv-zoom-open");
    dialog.tvOpener = opener || null;
    dialog.show();
    dialog.tvFit();
    dialog.querySelector(".tv-zoom__btn--close").focus();
  }

  function enhance(el) {
    if (el.dataset.tvZoom) return;
    el.dataset.tvZoom = "1";

    var frame = document.createElement("div");
    frame.className = "tv-zoom";
    el.parentNode.insertBefore(frame, el);
    frame.appendChild(el);

    var openBtn = button("⤢", "Open full screen", "tv-zoom__open");
    frame.appendChild(openBtn);

    el.title = "Click to open full screen";
    el.addEventListener("click", function () { open(el, openBtn); });
    openBtn.addEventListener("click", function () { open(el, openBtn); });
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
    if (dialog && dialog.open) dialog.tvClose();
    scan();
    var root = document.querySelector(".md-content") || document.body;
    observer = new MutationObserver(function () { scan(); });
    observer.observe(root, { childList: true, subtree: true });
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
