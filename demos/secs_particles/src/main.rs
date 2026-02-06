fn main() {
    // This is called from a public `main()` defined in lib.rs
    // `Egor_main()` macro expects it to be passed in so it can gen `android_main()` entry point there
    // We simply call that `main()` here as well since desktop and wasm expect the normal entry point
    demo_egor_secs_particles::main();
}
