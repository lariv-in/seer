/** AIS detail: single-point MapLibre map. */
(function () {
  var MAPLIBRE_CSS = "https://unpkg.com/maplibre-gl@^6.6.0/dist/maplibre-gl.css";
  var MAPLIBRE_JS = "https://unpkg.com/maplibre-gl@^6.6.0/dist/maplibre-gl.mjs";
  var DEFAULT_STYLE = "https://tiles.openfreemap.org/styles/liberty";

  var loadPromise = null;

  function ensureCss() {
    if (document.querySelector('link[data-seer-maplibre-css]')) {
      return;
    }
    var link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = MAPLIBRE_CSS;
    link.setAttribute("data-seer-maplibre-css", "1");
    document.head.appendChild(link);
  }

  function loadMapLibre() {
    if (window.maplibregl) {
      return Promise.resolve(window.maplibregl);
    }
    if (loadPromise) {
      return loadPromise;
    }
    ensureCss();
    loadPromise = import(MAPLIBRE_JS).then(function (mod) {
      window.maplibregl = mod;
      return mod;
    });
    return loadPromise;
  }

  function destroy(root) {
    var state = root._seerAisDetailMapState;
    if (!state) {
      return;
    }
    state.destroyed = true;
    if (state.map) {
      state.map.remove();
      state.map = null;
    }
    root._seerAisDetailMapState = null;
  }

  function mount(root) {
    if (!root || root._seerAisDetailMapState) {
      return;
    }
    var lat = parseFloat(root.getAttribute("data-lat"));
    var lng = parseFloat(root.getAttribute("data-lng"));
    if (!isFinite(lat) || !isFinite(lng)) {
      return;
    }

    var canvas = root.querySelector("[data-seer-map-canvas]");
    if (!canvas) {
      return;
    }

    var style = root.getAttribute("data-map-style") || DEFAULT_STYLE;
    var label = root.getAttribute("data-label") || "";

    var state = { destroyed: false, map: null };
    root._seerAisDetailMapState = state;

    loadMapLibre().then(function (maplibregl) {
      if (state.destroyed) {
        return;
      }
      var map = new maplibregl.Map({
        container: canvas,
        style: style,
        center: [lng, lat],
        zoom: 10,
      });
      state.map = map;
      map.on("load", function () {
        map.resize();
        new maplibregl.Marker().setLngLat([lng, lat]).addTo(map);
        if (label) {
          new maplibregl.Popup({ offset: 25, closeButton: false })
            .setLngLat([lng, lat])
            .setText(label)
            .addTo(map);
        }
      });
      requestAnimationFrame(function () {
        if (!state.destroyed) {
          map.resize();
        }
      });
    });

    root.addEventListener(
      "htmx:beforeCleanupElement",
      function () {
        destroy(root);
      },
      { once: true }
    );
  }

  function mountAll(scope) {
    var roots = (scope || document).querySelectorAll
      ? (scope || document).querySelectorAll("[data-seer-ais-detail-map]")
      : [];
    if (scope && scope.matches && scope.matches("[data-seer-ais-detail-map]")) {
      mount(scope);
    }
    for (var i = 0; i < roots.length; i++) {
      mount(roots[i]);
    }
  }

  window.SeerAisDetailMap = { mount: mount, mountAll: mountAll, destroy: destroy };

  if (!window.__seerAisDetailMapBootstrapped) {
    window.__seerAisDetailMapBootstrapped = true;
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", function () {
        mountAll(document);
      });
    } else {
      mountAll(document);
    }
    document.body.addEventListener("htmx:afterSwap", function (ev) {
      mountAll(ev.detail && ev.detail.elt ? ev.detail.elt : document);
    });
    document.body.addEventListener("htmx:load", function (ev) {
      mountAll(ev.detail && ev.detail.elt ? ev.detail.elt : document);
    });
  } else {
    mountAll(document);
  }
})();
