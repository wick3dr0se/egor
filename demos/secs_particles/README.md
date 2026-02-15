# Egor Secs Particles Demo

A tiny demonstration of mass spawning & rapidly moving enities/particles with [secs](https://github.com/wick3dr0se/secs) ECS

![SECS Particles GIF](/media/secs_particles.gif)

This is a cross platform demo (same code runs everywhere). It's split into a [main.rs](src/main.rs) and a [lib.rs](src/lib.rs) so that the `android_main()` entry point can be handled (`egor::main!()` macro generates this) in a library as Android expects. Main.rs just calls the same code into the `main()` entry point
