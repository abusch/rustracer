# Rustracer

Rustracer is a toy [raytracer](https://en.wikipedia.org/wiki/Ray_tracing_(graphics)) written in [Rust](http://rust-lang.org).

It started as a little playground to play with raytracing concepts that I found on [Scratchapixel](http://www.scratchapixel.com). I then found out about [tray_rust](http://github.com/TwinkleBear/tray_rust) and began refactoring my code to follow its design, but then eventually bought a copy of [Physically-Based Ray Tracing](http://www.pbrt.org) and decided to follow the [C++ implementation](https://github.com/mmp/pbrt-v3) as closely as possible to make it easier to follow along with the book.

**Note**: I am very much a beginner in Rust as well as in raytracing and this project is a way for me to learn about both. If the code is ugly or broken, it is very much my fault :) This is not helped by the fact that the C++ implementation is _very_ object-oriented, and doesn't always translate cleanly or idomatically to Rust... If you want to see a good, clean, well documented raytracer written in Rust by someone who knows what they're talking about, I definitely recommend you look at [tray_rust](http://github.com/TwinkleBear/tray_rust) instead :) That being said, feedback is more than welcome!

## Examples

The dragon model is from [The Stanford Scanning Repository](http://graphics.stanford.edu/data/3Dscanrep/).

![example1](example1.png)

## Currently supported
[x] Whitted and Direct Lighting integrators
[x] Basic shapes (disc, sphere) + triangle meshes
[x] Perspective camera with thin-lens depth-of-field
[x] Point lights, distant lights
[x] diffuse area lights (only with UniformSampleOne strategy), 
[x] infinite light (only plain colour, no HDR textures yet)
[x] BxDF for lambertian, perfect specular reflection / transmission, microfacet (Oren-Nayar and Torrance-Sparrow models, Beckmann and Trowbridge-Heitz distributions, Smith masking-shadowing function)
[x] Matte, Plastic, Metal materials
[x] Textures (imagemaps with MIPMapping, UV, CheckerBoard)
[x] BVH (only with MIDDLE splitting strategy)
[x] multi-threaded rendering

## TODO
[ ] More camera types (orthographic, realistic, environment)
[ ] More shapes (cone, paraboloid, hyperboloid, NURBs...)
[ ] More samplers
[ ] More BxDFs and materials
[ ] Volumetric rendering
[ ] More integrators: path-tracing and bidirectional path tracing
[ ] Animations
[ ] Support for PBRT scene file format
[ ] Support for reading/writing EXR files
[ ] Support for PLY meshes
[ ] more light types
[ ] Finish ray-differential support
[ ] Add support for EWA filtering for MIPMap
[ ] SAH strategy for BVH
[ ] k-d trees?
[ ] Spectral rendering?
[ ] ...
