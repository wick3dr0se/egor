use dioxus_devtools::{connect_subsecond, subsecond};
use egor_app::{Graphics, input::Input, time::FrameTimer};

/// Enables hot reloading for the given update function
///
/// Wraps your game loop so code changes reload automatically during development
///
/// ```rust
/// App::new().run(with_hot_reload(|graphics, input, timer| {
///     // will hot reload on changes
/// }));
/// ```
pub fn with_hot_reload<F>(mut f: F) -> impl FnMut(&mut Graphics, &Input, &FrameTimer)
where
    F: FnMut(&mut Graphics, &Input, &FrameTimer) + 'static,
{
    // Connect to subsecond once when wrapper is created
    connect_subsecond();

    // Return a closure that wraps the update in subsecond::call
    move |graphics, input, timer| {
        subsecond::call(|| f(graphics, input, timer));
    }
}
