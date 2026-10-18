// MapLibre style and theme helpers for the Protomaps-style PMTiles schema.

export const getMapPalette = (theme) => {
    if (theme === "dark") {
        return {
            background: "#121518",
            water: "#2f4f63",
            waterOutline: "#4f748a",
            roadMotorway: "#8b9aa6",
            roadTrunk: "#7f8d97",
            roadPrimary: "#73808b",
            roadSecondary: "#6a757f",
            roadOther: "#5f6973",
            landuse: {
                residential: "#22262b",
                suburb: "#1f2429",
                commercial: "#2a2f35",
                industrial: "#30343a",
                hospital: "#3a3030",
                military: "#2e3238",
                quarry: "#3a3f45",
                themePark: "#294036",
                cemetery: "#314238",
                track: "#3a332b",
                fallback: "#242a30",
            },
        };
    }

    return {
        background: "#e0e0e0",
        water: "#5da8d4",
        waterOutline: "#3a7fa0",
        roadMotorway: "#888888",
        roadTrunk: "#999999",
        roadPrimary: "#a0a0a0",
        roadSecondary: "#ffffff",
        roadOther: "#ffffff",
        landuse: {
            residential: "#d9d6cf",
            suburb: "#d3d0c9",
            commercial: "#ccc9c2",
            industrial: "#b8b2ab",
            hospital: "#e8c8b8",
            military: "#a8a8a8",
            quarry: "#959595",
            themePark: "#6ba86b",
            cemetery: "#a8c8a8",
            track: "#c8b89c",
            fallback: "#c8c8c8",
        },
    };
};

const buildLanduseColorExpression = (palette) => [
    "match",
    ["get", "class"],
    "residential", palette.landuse.residential,
    "suburb", palette.landuse.suburb,
    "neighbourhood", palette.landuse.suburb,
    "commercial", palette.landuse.commercial,
    "retail", palette.landuse.commercial,
    "industrial", palette.landuse.industrial,
    "hospital", palette.landuse.hospital,
    "military", palette.landuse.military,
    "quarry", palette.landuse.quarry,
    "theme_park", palette.landuse.themePark,
    "cemetery", palette.landuse.cemetery,
    "track", palette.landuse.track,
    palette.landuse.fallback,
];

const pmtilesSourceName = (filename) => filename.replace(/\.pmtiles$/, "");

export const buildSources = (files) => {
    const pmtilesSources = Object.fromEntries(
        files.map((filename) => [
            pmtilesSourceName(filename),
            { type: "vector", url: `pmtiles://${filename}` },
        ])
    );

    // Always present, even without map data.
    return {
        ...pmtilesSources,
        "waypoint-path": {
            type: "geojson",
            data: {
                type: "Feature",
                geometry: { type: "LineString", coordinates: [] },
                properties: {},
            },
        },
    };
};

const buildSourceLayers = (sourceName, palette) => [
    {
        id: `${sourceName}-landuse`,
        type: "fill",
        source: sourceName,
        "source-layer": "landuse",
        paint: {
            "fill-color": buildLanduseColorExpression(palette),
            "fill-opacity": 1,
        },
    },
    {
        id: `${sourceName}-water`,
        type: "fill",
        source: sourceName,
        "source-layer": "water",
        paint: {
            "fill-color": palette.water,
            "fill-opacity": 0.95,
        },
    },
    {
        id: `${sourceName}-water-outline`,
        type: "line",
        source: sourceName,
        "source-layer": "water",
        paint: {
            "line-color": palette.waterOutline,
            "line-width": ["interpolate", ["linear"], ["zoom"], 4, 0.5, 8, 0.5, 14, 1.5],
        },
    },
    {
        id: `${sourceName}-motorway`,
        type: "line",
        source: sourceName,
        "source-layer": "transportation",
        filter: ["==", ["get", "class"], "motorway"],
        layout: { "line-join": "round", "line-cap": "round" },
        paint: {
            "line-color": palette.roadMotorway,
            "line-width": ["interpolate", ["linear"], ["zoom"], 5, 0.5, 10, 2, 14, 4, 18, 10],
        },
    },
    {
        id: `${sourceName}-trunk`,
        type: "line",
        source: sourceName,
        "source-layer": "transportation",
        filter: ["==", ["get", "class"], "trunk"],
        layout: { "line-join": "round", "line-cap": "round" },
        paint: {
            "line-color": palette.roadTrunk,
            "line-width": ["interpolate", ["linear"], ["zoom"], 5, 0.4, 10, 1.5, 14, 3, 18, 8],
        },
    },
    {
        id: `${sourceName}-primary`,
        type: "line",
        source: sourceName,
        "source-layer": "transportation",
        filter: ["==", ["get", "class"], "primary"],
        layout: { "line-join": "round", "line-cap": "round" },
        paint: {
            "line-color": palette.roadPrimary,
            "line-width": ["interpolate", ["linear"], ["zoom"], 8, 0.5, 12, 1.5, 16, 3],
        },
    },
    {
        id: `${sourceName}-secondary`,
        type: "line",
        source: sourceName,
        "source-layer": "transportation",
        filter: ["==", ["get", "class"], "secondary"],
        layout: { "line-join": "round", "line-cap": "round" },
        paint: {
            "line-color": palette.roadSecondary,
            "line-width": ["interpolate", ["linear"], ["zoom"], 10, 0.5, 14, 1.5, 18, 4],
        },
    },
    {
        id: `${sourceName}-roads-other`,
        type: "line",
        source: sourceName,
        "source-layer": "transportation",
        filter: ["all", ["!=", ["get", "class"], "motorway"], ["!=", ["get", "class"], "trunk"], ["!=", ["get", "class"], "primary"], ["!=", ["get", "class"], "secondary"]],
        paint: {
            "line-color": palette.roadOther,
            "line-width": ["interpolate", ["linear"], ["zoom"], 12, 0.3, 16, 0.8, 18, 2],
            "line-opacity": ["interpolate", ["linear"], ["zoom"], 10, 0.3, 14, 0.7],
        },
    },
];

export const buildLayers = (files, theme) => {
    const palette = getMapPalette(theme);

    return [
        {
            id: "background",
            type: "background",
            paint: { "background-color": palette.background },
        },
        ...files.flatMap((filename) => buildSourceLayers(pmtilesSourceName(filename), palette)),
        // Rendered on top of all other layers.
        {
            id: "waypoint-path",
            type: "line",
            source: "waypoint-path",
            paint: {
                "line-color": "#e53935",
                "line-width": 5,
                "line-opacity": 0.9,
            },
        },
    ];
};

export const applyThemeToMap = (map, files, theme) => {
    const palette = getMapPalette(theme);
    const landuseColorExpression = buildLanduseColorExpression(palette);

    const setPaint = (layerId, property, value) => {
        if (map.getLayer(layerId)) {
            map.setPaintProperty(layerId, property, value);
        }
    };

    setPaint("background", "background-color", palette.background);

    for (const filename of files) {
        const sourceName = pmtilesSourceName(filename);

        setPaint(`${sourceName}-landuse`, "fill-color", landuseColorExpression);
        setPaint(`${sourceName}-water`, "fill-color", palette.water);
        setPaint(`${sourceName}-water-outline`, "line-color", palette.waterOutline);
        setPaint(`${sourceName}-motorway`, "line-color", palette.roadMotorway);
        setPaint(`${sourceName}-trunk`, "line-color", palette.roadTrunk);
        setPaint(`${sourceName}-primary`, "line-color", palette.roadPrimary);
        setPaint(`${sourceName}-secondary`, "line-color", palette.roadSecondary);
        setPaint(`${sourceName}-roads-other`, "line-color", palette.roadOther);
    }
};
