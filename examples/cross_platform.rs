use egor::{Color, app::App};

fn main() {
    App::new().run(|g| {
        let [cx, cy] = [g.screen_size()[0] / 2.0, g.screen_size()[1] / 2.0];
        let size = 128.0;
        let half = size / 2.0;

        g.tri().at(cx - half, cy - half).color(Color::GREEN);
        g.tri().at(cx + half, cy - half);
        g.tri().at(cx + half, cy + half).color(Color::GREEN);
        g.tri().at(cx - half, cy + half);
        g.rect().at(cx, cy).size(size, size);
    });
}
