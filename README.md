# Speedwalk

This is a web tool to assess sidewalk data quality in OpenStreetMap and conveniently edit it. It's
intended for use only by OSM mappers that have a basic understanding of [how to map
sidewalks](https://wiki.openstreetmap.org/wiki/Sidewalks).

## Goals and status

The first version:

- [x] Display sidewalk and crossing data clearly
- Detect possible tagging errors
  - [x] `footway=crossing` ways that may be missing a crossing node
  - [x] `highway=footway` ways that may need to be split when they cross a road
  - [x] Roads parallel to separately mapped sidewalks, but maybe missing `sidewalk:{left,right,both} = separate`
  - [ ] A `highway=footway` that's parallel to a road, but maybe missing `footway=sidewalk`

Later work:

- [ ] https://github.com/a-b-street/speedwalk/issues/4
- [ ] https://github.com/a-b-street/speedwalk/issues/5

### Automated sidewalk generation

Speedwalk also has some tools to generate separate sidewalk geometry when they are missing. This is
**not** intended to be uploaded to OSM and act as real data. It is only intended for other tools
that consume OSM data and make assumptions about consistent mapping styles. This feature is very
experimental and often wrong.

Please also check out https://github.com/kauevestena/osm_sidewalkreator for a similar effort.

## Local development

To run locally, you'll need [npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm),
[wasm-pack](https://github.com/rustwasm/wasm-pack), and
[cargo](https://www.rust-lang.org/tools/install).

`cd web`, and then:

- `npm ci` to install dependencies (`ci` to make sure the versions in
  `package-lock.json` are used)
- `npm run wasm` to rebuild the Rust backend
   vite doesn't automatically rebuild when you edit things
- `npm run dev` to run locally
  - Changes to the Svelte/CSS usually auto-reload in your browser
- `npm run fmt` to auto-format code
- `npm run check` to see TypeScript errors
