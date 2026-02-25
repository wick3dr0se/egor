use egor::{
    app::{
        App, FrameContext,
        egui::{Context, Window},
    },
    math::{Rect, Vec2, vec2},
    render::{Align, Color, Graphics},
};
use rand::Rng;

const WORLD_SIZE: f32 = 2048.0;
const FOOD_COUNT: usize = (WORLD_SIZE / 2.0) as usize;

fn rand_range(lo: f32, hi: f32) -> f32 {
    rand::thread_rng().gen_range(lo..hi)
}

struct Cell {
    center: Vec2,
    radius: f32,
    absorbed: bool,
}

impl Cell {
    fn intersects(&self, other: &Cell) -> bool {
        let radius_sum = self.radius + other.radius;
        self.center.distance_squared(other.center) <= radius_sum * radius_sum
    }

    fn try_absorb(&mut self, other: &mut Cell) -> Option<f32> {
        if self.intersects(other) && self.radius > other.radius {
            other.absorbed = true;
            let growth = self.radius - (self.radius.powi(2) + other.radius.powi(2)).sqrt();
            self.radius -= growth;
            return Some(growth * 0.1);
        }
        None
    }
}

struct Food {
    cell: Cell,
}

impl Food {
    fn random(bounds: Vec2) -> Self {
        Self {
            cell: Cell {
                center: vec2(rand_range(0.0, bounds.x), rand_range(0.0, bounds.y)),
                radius: 5.0,
                absorbed: false,
            },
        }
    }

    fn render(&self, gfx: &mut Graphics) {
        draw_circle(gfx, self.cell.center, self.cell.radius, Color::RED);
    }
}

struct Player {
    cell: Cell,
    speed: f32,
    absorbed_food: i32,
}

impl Player {
    fn new(bounds: Vec2) -> Self {
        Self {
            cell: Cell {
                center: bounds / 2.0,
                radius: 10.0,
                absorbed: false,
            },
            speed: 80.0,
            absorbed_food: 0,
        }
    }

    fn update(&mut self, mouse_world: Vec2, food: &mut [Food], dt: f32) {
        let dir = (mouse_world - self.cell.center).normalize_or_zero();
        self.cell.center += dir * self.speed * dt;

        for f in food.iter_mut() {
            if let Some(growth_factor) = self.cell.try_absorb(&mut f.cell) {
                self.absorbed_food += 1;
                self.speed -= growth_factor;
            }
        }
    }

    fn render(&self, gfx: &mut Graphics) {
        draw_circle(
            gfx,
            self.cell.center,
            self.cell.radius,
            Color::new([0.0, 0.0, 0.55, 1.0]),
        );
    }
}

struct Creature {
    cell: Cell,
    speed: f32,
}

impl Creature {
    fn random(bounds: Vec2) -> Self {
        Self {
            cell: Cell {
                center: vec2(rand_range(0.0, bounds.x), rand_range(0.0, bounds.y)),
                radius: 8.0,
                absorbed: false,
            },
            speed: 65.0,
        }
    }

    fn update(
        &mut self,
        food: &mut [Food],
        player: &mut Option<Player>,
        opps: &mut [Creature],
        dt: f32,
    ) {
        if let Some(p) = player {
            self.speed -= self.cell.try_absorb(&mut p.cell).unwrap_or(0.0);
            p.speed -= p.cell.try_absorb(&mut self.cell).unwrap_or(0.0);
        }
        for o in opps.iter_mut() {
            self.speed -= self.cell.try_absorb(&mut o.cell).unwrap_or(0.0);
            o.speed -= o.cell.try_absorb(&mut self.cell).unwrap_or(0.0);
        }

        if let Some(nearby) = food
            .iter_mut()
            .filter(|f| !f.cell.absorbed)
            .min_by_key(|f| self.cell.center.distance_squared(f.cell.center) as i64)
        {
            let dir = (nearby.cell.center - self.cell.center).normalize_or_zero();
            self.cell.center += dir * self.speed * dt;
            self.speed -= self.cell.try_absorb(&mut nearby.cell).unwrap_or(0.0);
        }
    }

    fn render(&self, gfx: &mut Graphics) {
        draw_circle(
            gfx,
            self.cell.center,
            self.cell.radius,
            Color::new([1.0, 0.65, 0.0, 1.0]),
        );
    }
}

struct World {
    bounds: Vec2,
    food: Vec<Food>,
    player: Option<Player>,
    creatures: Vec<Creature>,
}

impl World {
    fn new(bounds: Vec2) -> Self {
        Self {
            bounds,
            food: (0..FOOD_COUNT).map(|_| Food::random(bounds)).collect(),
            player: Some(Player::new(bounds)),
            creatures: (0..99).map(|_| Creature::random(bounds)).collect(),
        }
    }
}

#[derive(PartialEq)]
enum GameState {
    Playing,
    Win,
    Lose,
}

struct Game {
    state: GameState,
    world: World,
    camera_target: Vec2,
    zoom: f32,
}

impl Game {
    fn new() -> Self {
        Self {
            state: GameState::Playing,
            world: World::new(Vec2::splat(WORLD_SIZE)),
            camera_target: Vec2::ZERO,
            zoom: 1.0,
        }
    }

    fn update(&mut self, mouse_world: Vec2, dt: f32) {
        if self.state != GameState::Playing {
            return;
        }

        if let Some(p) = &mut self.world.player {
            p.update(mouse_world, &mut self.world.food, dt);
            self.camera_target = p.cell.center;
        } else if let Some(c) = self
            .world
            .creatures
            .iter()
            .max_by(|a, b| a.cell.radius.partial_cmp(&b.cell.radius).unwrap())
        {
            self.camera_target = c.cell.center;
        }

        for c_idx in 0..self.world.creatures.len() + 1 {
            let (left, right) = self.world.creatures.split_at_mut(c_idx);
            if let Some(c) = left.last_mut() {
                c.update(&mut self.world.food, &mut self.world.player, right, dt);
            }
        }

        self.world.creatures.retain(|c| !c.cell.absorbed);
        self.world.food.retain(|f| !f.cell.absorbed);

        if self.world.player.as_ref().is_some_and(|p| p.cell.absorbed) {
            self.world.player = None;
        }

        while self.world.food.len() < FOOD_COUNT {
            self.world.food.push(Food::random(self.world.bounds));
        }

        if self.world.player.is_some() && self.world.creatures.is_empty() {
            self.state = GameState::Win;
        } else if self.world.player.is_none() && self.world.creatures.len() == 1 {
            self.state = GameState::Lose;
        }
    }

    fn render(&self, screen_size: Vec2, gfx: &mut Graphics, egui_ctx: &Context, fps: u32) {
        gfx.clear(Color::new([0.53, 0.81, 0.98, 1.0]));
        match self.state {
            GameState::Playing => {
                gfx.rect()
                    .at(Vec2::ZERO)
                    .size(self.world.bounds)
                    .color(Color::WHITE);

                for f in &self.world.food {
                    f.render(gfx);
                }
                for c in &self.world.creatures {
                    c.render(gfx);
                }
                if let Some(p) = &self.world.player {
                    p.render(gfx);
                }

                Window::new("Stats").show(egui_ctx, |ui| {
                    ui.label(format!("FPS: {}", fps));
                    ui.label(format!("Creatures: {}", self.world.creatures.len()));
                    if let Some(p) = &self.world.player {
                        ui.label(format!("Player Radius: {:.2}", p.cell.radius));
                        ui.label(format!("Food Eaten: {}", p.absorbed_food));
                    }
                });

                let mut board: Vec<(String, f32)> = self
                    .world
                    .creatures
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (format!("Creature {}", i + 1), c.cell.radius))
                    .collect();
                if let Some(p) = &self.world.player {
                    board.push(("Player".to_string(), p.cell.radius));
                }
                board.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                board.truncate(10);
                Window::new("Leaderboard").show(egui_ctx, |ui| {
                    for (i, (name, radius)) in board.iter().enumerate() {
                        ui.label(format!("{}. {}: {:.2}", i + 1, name, radius));
                    }
                });
            }

            GameState::Win | GameState::Lose => {
                let (text, color) = if self.state == GameState::Win {
                    ("You Win!", Color::GREEN)
                } else {
                    ("You Lose!", Color::RED)
                };
                gfx.text(text)
                    .color(color)
                    .size(100.0)
                    .bold()
                    .in_rect(Rect::new(Vec2::ZERO, screen_size), Align::MiddleCenter);
            }
        }
    }
}

fn draw_circle(gfx: &mut Graphics, center: Vec2, radius: f32, color: Color) {
    gfx.polygon()
        .at(center)
        .radius(radius)
        .segments(32)
        .color(color);
}

fn main() {
    let mut game = Game::new();

    App::new().title("Egor Agar Demo").run(
        move |FrameContext {
                  gfx,
                  input,
                  timer,
                  egui_ctx,
                  ..
              }| {
            let screen_size = gfx.screen_size();

            game.zoom *= 1.0 + input.mouse_scroll() * 0.1;
            gfx.camera().set_zoom(game.zoom);
            gfx.camera().center(game.camera_target, screen_size);

            let mouse_pos = gfx.camera().screen_to_world(input.mouse_position().into());

            game.update(mouse_pos, timer.delta);
            game.render(screen_size, gfx, egui_ctx, timer.fps);
        },
    );
}
