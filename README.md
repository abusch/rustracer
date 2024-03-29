# Rustracer

Rustracer is a toy
[raytracer](https://en.wikipedia.org/wiki/Ray_tracing_(graphics)) written in
[Rust](http://rust-lang.org).

[![Rust](https://github.com/abusch/rustracer/actions/workflows/rust.yml/badge.svg)](https://github.com/abusch/rustracer/actions/workflows/rust.yml)
[![dependency status](https://deps.rs/repo/github/abusch/rustracer/status.svg)](https://deps.rs/repo/github/abusch/rustracer)

> *Update (27/05/2020):* This project is not really developped anymore as I ran
> out of steam and spare time. I'm doing a little bit of maintainance
> occasionally, such as updating dependencies, to keep it from bitrotting but I
> suggest you look at [rs_pbrt](https://github.com/wahn/rs_pbrt) for a more
> full-featured port of `pbrt`.

## History

It started as a little playground to play with raytracing concepts that I found
on [Scratchapixel](http://www.scratchapixel.com). I then found out about
[tray_rust](http://github.com/TwinkleBear/tray_rust) and began refactoring my
code to follow its design, but then eventually bought a copy of
[Physically-Based Ray Tracing](http://www.pbrt.org) and decided to follow the
[C++ implementation](https://github.com/mmp/pbrt-v3) as closely as possible to
make it easier to follow along with the book. Consider this a port of PBRTv3 to
Rust.

**Note**: I am very much a beginner in Rust as well as in raytracing and this
project is a way for me to learn about both. If the code is ugly or broken, it
is very much my fault :) This is not helped by the fact that the C++
implementation is _very_ object-oriented, and doesn't always translate cleanly
or idiomatically to Rust... I haven't really paid attention to performance yet,
or number of allocations, so both are probably pretty bad :) That being said,
feedback is more than welcome! 

## Related projects
 * [tray_rust](http://github.com/TwinkleBear/tray_rust): if you want to see a
   good physically-based raytracer in Rust written by someone who knows what
   they're talking about, you should check this project instead of mine :) It's
   got some nice features, like distributed rendering, and the code is
   beautifully written and documented.

 * [rs_pbrt](https://github.com/wahn/rs_pbrt): another port of PBRT to Rust.

## Examples

More examples can be found in the [renders/](renders/) directory.

![balls](balls.png)

## How to build

To run the CLI, run `cargo run --release -p rustracer -- scene_file.pbrt`.

## Currently supported
 * Integrators:
     * Whitted
     * Direct Lighting (with "all" or "one" sampling strategies)
     * Path Tracing (with "uniform" or "spatial" sampling strategies)
     * Normal (for debuging)
     * Ambient Occlusion
 * Shapes:
     * Basic shapes: disc, sphere, cylinder
     * triangle meshes
 * Perspective camera with thin-lens depth-of-field
 * Lights:
     * Point lights
     * distant lights
     * diffuse area lights
     * infinite area light
 * BxDFs:
     * Lambertian
     * perfect specular reflection / transmission
     * microfacet (Oren-Nayar and Torrance-Sparrow models, Beckmann and
       Trowbridge-Heitz distributions, Smith masking-shadowing function)
 * Materials:
     * Matte
     * Plastic
     * Metal / RoughMetal
     * Glass / RoughGlass
     * Substrate (thin-coated)
     * Translucent
     * Uber
     * Disney (without SSS)
 * Textures (imagemaps, UV, CheckerBoard)
     * imagemap with mipmapping (with trilinear and EWA filtering)
     * UV
     * CheckerBoard
     * Fbm
 * BVH acceleration structure (MIDDLE and SAH splitting strategy)
 * multi-threaded rendering
 * Support for PBRT scene file format (still incomplete)
 * Read textures in PNG, TGA, PFM, HDR or EXR (thanks to [openexr-rs](https://github.com/cessen/openexr-rs))
 * Write images in PNG or EXR
 * Support for PLY meshes (thanks to [ply-rs](https://github.com/Fluci/ply-rs))
 * Bump mapping
 * Memory arena (thanks to [light_arena](https://github.com/Twinklebear/light_arena))

## TODO
 * More CLI parameters (e.g. output file name, thread numbers)
 * More camera types (orthographic, realistic, environment)
 * More shapes (cone, paraboloid, hyperboloid, NURBs...)
 * More samplers (Halton, Sobol...)
 * More BxDFs and materials
 * Implement subsurface scattering
 * Volumetric rendering
 * More integrators: bidirectional path tracing, MTL, SPPM
 * Animations
 * more light types
 * k-d trees?
 * Spectral rendering?
 * some SIMD optimisation?
 * ...

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

