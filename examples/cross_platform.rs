use egor::{App, Color, Context};

fn init(ctx: Context) {
    ctx.window.set_title("egor-cross-platform");
}

fn update(ctx: &mut Context) {
    ctx.renderer.clear(Color::GREEN);
    println!("FPS: {}", ctx.renderer.fps());
}

fn main() {
    App::new(init).run(update);
}
