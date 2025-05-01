use egor::{App, Color};

fn main() {
    let mut last_fps = 0;

    App::init(|ctx| {
        ctx.window().title("Egor");
    })
    .run(move |ctx| {
        ctx.clear(Color::GREEN);

        let fps = ctx.fps();
        if fps != last_fps {
            last_fps = fps;
            println!("FPS: {}\x1b[A\x1b[2K", ctx.fps());
        }
    });
}
