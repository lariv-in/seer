/** Live Map: MapLibre GL JS + /seer-intel/map/data WebSocket. */
(function () {
  var MAPLIBRE_CSS = "https://unpkg.com/maplibre-gl@^6.6.0/dist/maplibre-gl.css";
  var MAPLIBRE_JS = "https://unpkg.com/maplibre-gl@^6.6.0/dist/maplibre-gl.mjs";
  var DEFAULT_STYLE = "https://tiles.openfreemap.org/styles/liberty";
  var SOURCE_ID = "seer-intel-points";
  var LAYER_CLUSTER = "seer-intel-clusters";
  var LAYER_COUNT = "seer-intel-cluster-count";
  var LAYER_POINT = "seer-intel-unclustered";
  var DEBOUNCE_MS = 250;
  var RECONNECT_MS = 2000;

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
    // MapLibre v6 is ESM-only; dynamic import works from this classic script.
    loadPromise = import(MAPLIBRE_JS).then(function (mod) {
      window.maplibregl = mod;
      return mod;
    });
    return loadPromise;
  }

  function wsUrl(path) {
    var proto = location.protocol === "https:" ? "wss:" : "ws:";
    return proto + "//" + location.host + path;
  }

  function setStatus(root, text) {
    var el = root.querySelector("[data-seer-map-status]");
    if (el) {
      el.textContent = text || "";
    }
  }

  function boundsPayload(map) {
    var b = map.getBounds();
    return {
      north: b.getNorth(),
      south: b.getSouth(),
      east: b.getEast(),
      west: b.getWest(),
    };
  }

  function toFeatureCollection(points) {
    return {
      type: "FeatureCollection",
      features: (points || []).map(function (p) {
        return {
          type: "Feature",
          geometry: { type: "Point", coordinates: [p.lng, p.lat] },
          properties: {
            id: p.id,
            intel_id: p.intel_id,
            address: p.address || "",
          },
        };
      }),
    };
  }

  function ensureLayers(map, maplibregl) {
    if (map.getSource(SOURCE_ID)) {
      return;
    }
    map.addSource(SOURCE_ID, {
      type: "geojson",
      data: { type: "FeatureCollection", features: [] },
      cluster: true,
      clusterMaxZoom: 14,
      clusterRadius: 42,
    });
    map.addLayer({
      id: LAYER_CLUSTER,
      type: "circle",
      source: SOURCE_ID,
      filter: ["has", "point_count"],
      paint: {
        "circle-color": "#2563eb",
        "circle-radius": ["step", ["get", "point_count"], 16, 25, 20, 100, 26],
        "circle-opacity": 0.85,
      },
    });
    map.addLayer({
      id: LAYER_COUNT,
      type: "symbol",
      source: SOURCE_ID,
      filter: ["has", "point_count"],
      layout: {
        "text-field": "{point_count_abbreviated}",
        "text-size": 12,
      },
      paint: { "text-color": "#ffffff" },
    });
    map.addLayer({
      id: LAYER_POINT,
      type: "circle",
      source: SOURCE_ID,
      filter: ["!", ["has", "point_count"]],
      paint: {
        "circle-color": "#dc2626",
        "circle-radius": 7,
        "circle-stroke-width": 1.5,
        "circle-stroke-color": "#ffffff",
      },
    });

    map.on("click", LAYER_CLUSTER, function (e) {
      var features = map.queryRenderedFeatures(e.point, { layers: [LAYER_CLUSTER] });
      if (!features.length) {
        return;
      }
      var clusterId = features[0].properties.cluster_id;
      map
        .getSource(SOURCE_ID)
        .getClusterExpansionZoom(clusterId)
        .then(function (zoom) {
          map.easeTo({ center: features[0].geometry.coordinates, zoom: zoom });
        })
        .catch(function () {});
    });

    map.on("click", LAYER_POINT, function (e) {
      var f = e.features && e.features[0];
      if (!f) {
        return;
      }
      var props = f.properties || {};
      var address = props.address || "Intel event";
      var intelId = props.intel_id;
      var href = intelId != null ? "/seer-intel/" + encodeURIComponent(String(intelId)) : null;
      var html =
        "<div style=\"max-width:16rem\">" +
        "<div style=\"font-weight:600;margin-bottom:0.25rem\">" +
        escapeHtml(address) +
        "</div>" +
        (href
          ? '<a href="' +
            href +
            '" style="text-decoration:underline">Open intel #' +
            escapeHtml(String(intelId)) +
            "</a>"
          : "") +
        "</div>";
      new maplibregl.Popup({ offset: 12 })
        .setLngLat(f.geometry.coordinates)
        .setHTML(html)
        .addTo(map);
    });

    map.on("mouseenter", LAYER_CLUSTER, function () {
      map.getCanvas().style.cursor = "pointer";
    });
    map.on("mouseleave", LAYER_CLUSTER, function () {
      map.getCanvas().style.cursor = "";
    });
    map.on("mouseenter", LAYER_POINT, function () {
      map.getCanvas().style.cursor = "pointer";
    });
    map.on("mouseleave", LAYER_POINT, function () {
      map.getCanvas().style.cursor = "";
    });
  }

  function escapeHtml(s) {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function applyPoints(map, maplibregl, points) {
    ensureLayers(map, maplibregl);
    var src = map.getSource(SOURCE_ID);
    if (src) {
      src.setData(toFeatureCollection(points));
    }
    setStatus(
      map.getContainer().closest("[data-seer-live-map]") || document,
      (points && points.length ? points.length : 0) + " events in view"
    );
  }

  function connectWs(root, map, maplibregl) {
    var path = root.getAttribute("data-ws-path") || "/seer-intel/map/data";
    var state = root._seerMapState;
    if (!state) {
      return;
    }
    if (state.ws) {
      try {
        state.ws.close();
      } catch (_e) {}
      state.ws = null;
    }

    var socket = new WebSocket(wsUrl(path));
    state.ws = socket;
    socket.binaryType = "arraybuffer";

    socket.onopen = function () {
      setStatus(root, "Connected — pan to load events");
      sendBounds(root, map);
    };

    socket.onmessage = function (ev) {
      var points = [];
      try {
        if (typeof ev.data === "string") {
          points = JSON.parse(ev.data);
        } else {
          var text = new TextDecoder("utf-8").decode(ev.data);
          points = JSON.parse(text);
        }
      } catch (_e) {
        setStatus(root, "Bad map data payload");
        return;
      }
      if (!Array.isArray(points)) {
        points = [];
      }
      applyPoints(map, maplibregl, points);
    };

    socket.onclose = function () {
      if (state.destroyed) {
        return;
      }
      setStatus(root, "Reconnecting…");
      state.reconnectTimer = setTimeout(function () {
        connectWs(root, map, maplibregl);
      }, RECONNECT_MS);
    };

    socket.onerror = function () {
      try {
        socket.close();
      } catch (_e2) {}
    };
  }

  function sendBounds(root, map) {
    var state = root._seerMapState;
    if (!state || !state.ws || state.ws.readyState !== WebSocket.OPEN) {
      return;
    }
    state.ws.send(JSON.stringify(boundsPayload(map)));
  }

  function scheduleBounds(root, map) {
    var state = root._seerMapState;
    if (!state) {
      return;
    }
    if (state.debounceTimer) {
      clearTimeout(state.debounceTimer);
    }
    state.debounceTimer = setTimeout(function () {
      sendBounds(root, map);
    }, DEBOUNCE_MS);
  }

  function destroy(root) {
    var state = root._seerMapState;
    if (!state) {
      return;
    }
    state.destroyed = true;
    if (state.debounceTimer) {
      clearTimeout(state.debounceTimer);
    }
    if (state.reconnectTimer) {
      clearTimeout(state.reconnectTimer);
    }
    if (state.ws) {
      try {
        state.ws.close();
      } catch (_e) {}
    }
    if (state.map) {
      try {
        state.map.remove();
      } catch (_e2) {}
    }
    root._seerMapState = null;
    root.removeAttribute("data-mounted");
  }

  async function mount(root) {
    if (!root || root.getAttribute("data-mounted") === "1") {
      return;
    }
    root.setAttribute("data-mounted", "1");

    var canvas = root.querySelector("[data-seer-map-canvas]");
    if (!canvas) {
      return;
    }

    var styleUrl = (root.getAttribute("data-map-style") || "").trim() || DEFAULT_STYLE;

    setStatus(root, "Loading MapLibre…");
    var maplibregl;
    try {
      maplibregl = await loadMapLibre();
    } catch (err) {
      setStatus(root, "Failed to load MapLibre GL JS");
      canvas.innerHTML =
        '<div class="flex h-full items-center justify-center p-6 text-center opacity-70">' +
        "Could not load MapLibre GL JS from the CDN." +
        "</div>";
      return;
    }

    var map = new maplibregl.Map({
      container: canvas,
      style: styleUrl,
      center: [0, 20],
      zoom: 1.6,
      attributionControl: true,
    });
    map.addControl(new maplibregl.NavigationControl(), "top-right");

    var state = {
      map: map,
      ws: null,
      debounceTimer: null,
      reconnectTimer: null,
      destroyed: false,
    };
    root._seerMapState = state;

    map.on("load", function () {
      ensureLayers(map, maplibregl);
      map.resize();
      connectWs(root, map, maplibregl);
    });
    map.on("moveend", function () {
      scheduleBounds(root, map);
    });
    // HTMX pane swaps can leave the canvas at 0×0 until layout settles.
    requestAnimationFrame(function () {
      if (!state.destroyed) {
        map.resize();
      }
    });

    // HTMX may swap this node away — clean up WebGL + socket.
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
      ? (scope || document).querySelectorAll("[data-seer-live-map]")
      : [];
    if (scope && scope.matches && scope.matches("[data-seer-live-map]")) {
      mount(scope);
    }
    for (var i = 0; i < roots.length; i++) {
      mount(roots[i]);
    }
  }

  window.SeerLiveMap = { mount: mount, mountAll: mountAll, destroy: destroy };

  if (!window.__seerLiveMapBootstrapped) {
    window.__seerLiveMapBootstrapped = true;
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
