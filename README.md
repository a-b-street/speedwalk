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
  - [x] Roads with `sidewalk=separate`, which is ambiguous about the side
  - [ ] A `highway=footway` that's parallel to a road, but maybe missing `footway=sidewalk`
  - [ ] A road tagged with `sidewalk=separate` that's ambiguous about the side

Later work:

- [ ] https://github.com/a-b-street/speedwalk/issues/4 Split ways within the tool
- [ ] https://github.com/a-b-street/speedwalk/issues/5 Integrate into the iD editor

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

### Running tests

Rust unit tests are located in `backend/src/export.rs` (nudge removal tests) and `backend/src/classify.rs` (classification tests).

**From the `web` directory:**
- `npm run test` - Run all Rust tests
- `npm run test:export` - Run only the export/nudge removal tests
- `npm run test:watch` - Run tests in watch mode (requires `cargo-watch`: `cargo install cargo-watch`)

**From the `backend` directory (Rust way):**
- `cargo test` - Run all tests
- `cargo test --lib export::tests` - Run only export module tests
- `cargo test --lib export::tests::test_nudge_keep_case` - Run a specific test
- `cargo test --lib -- --nocapture` - Run tests with output (println! visible)

Test data files are in `backend/src/export/test_data/`:
- `speedwalk_debug_*.osm.xml` - OSM XML files exported from the debug feature for testing nudge removal

Check recent changesets using Speedwalk: https://changesets.mapki.com/?tags=created_by%3DSpeedwalk
