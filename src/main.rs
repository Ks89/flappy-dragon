#![warn(clippy::pedantic)]

use bracket_lib::prelude::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 40.0;

const DRAGON_FRAMES: [u16; 6] = [64, 1, 2, 3, 2, 1];

// |    0,0  1,0  2,0  3,0  4,0
// |    0,1  1,1  2,1  3,1  4,1
// |    0,2  1,2  2,2  3,2  4,2
// y __________________________> x

struct Player {
    x: i32,
    y: f32,
    velocity: f32,
    frame: usize, // Usize to index arrays
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y: y as f32,
            velocity: 0.0,
            frame: 0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_fancy(
            PointF::new(0.0, self.y),
            1,
            Degrees::new(0.0),
            PointF::new(2.0, 2.0),
            WHITE,
            NAVY,
            DRAGON_FRAMES[self.frame],
        );
        ctx.set_active_console(0);
    }

    fn gravity_and_mode(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.1;
        }
        self.y += self.velocity;
        if self.y < 0.0 {
            self.y = 0.0;
        }

        self.x += 1;
        self.frame += 1;
        self.frame = self.frame % 6; // % is modulus - remainder
    }

    fn flap(&mut self) {
        self.velocity = -1.0; // a negative number, so it moves upward, because 0 is the top of the screen
    }
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
}

impl Obstacle {
    fn new(x: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(5, 45),
            size: i32::max(10, 40),
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        // The ground
        for x in 0..SCREEN_WIDTH {
            ctx.set(x, SCREEN_HEIGHT - 1, WHITE, WHITE, to_cp437('#'));
        }

        let screen_x = self.x - player_x;
        let half_size = self.size / 2;

        // Top wall
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, WHITE, NAVY, 179);
        }

        // Bottom wall - now leaving room for the ground
        for y in self.gap_y + half_size..SCREEN_HEIGHT - 1 {
            ctx.set(screen_x, y, WHITE, NAVY, 179);
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x == self.x;
        let player_above_gap = (player.y as i32) < self.gap_y - half_size;
        let player_below_gap = (player.y as i32) > self.gap_y + half_size;
        does_x_match && (player_above_gap || player_below_gap)
    }
}

enum GameMode {
    Menu,
    Playing,
    Pause,
    End,
}

struct State {
    player: Player,
    frame_time: f32,
    obstacles: Vec<Obstacle>,
    mode: GameMode,
    score: i32,
}

impl State {
    fn new() -> Self {
        State {
            player: Player::new(5, 25),
            frame_time: 0.0,
            obstacles: vec![Obstacle::new(SCREEN_WIDTH)],
            mode: GameMode::Menu,
            score: 0,
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_mode();
        }

        match ctx.key {
            Some(VirtualKeyCode::Space) => {
                self.player.flap();
            }
            Some(VirtualKeyCode::Escape) => {
                // TODO implement pause
                self.mode = GameMode::Pause;
            }
            _ => {}
        }

        self.player.render(ctx);
        ctx.print(0, 0, "Press SPACE to flap.");
        ctx.print(0, 1, "Press ESC to pause.");
        ctx.print(0, 2, &format!("Score {}", self.score));

        // add new obstacles with a certain percentage
        let mut random = RandomNumberGenerator::new();
        if self.frame_time as i32 % 50 == 0 && random.range(1, 40) % 10 == 0 {
            self.obstacles
                .push(Obstacle::new(self.player.x + SCREEN_WIDTH));
        }

        // render obstacles
        for obstacle in &mut self.obstacles {
            obstacle.render(ctx, self.player.x);
        }

        // add obstacles to remove in a vec
        let mut pop_obstacle: bool = false;
        for obstacle in &mut self.obstacles {
            if self.player.x > obstacle.x {
                self.score += 1;
                pop_obstacle = true;
            }
        }

        for obstacle in &mut self.obstacles {
            if self.player.y as i32 > SCREEN_HEIGHT || obstacle.hit_obstacle(&self.player) {
                self.mode = GameMode::End;
            }
        }

        if pop_obstacle {
            // remove first element
            self.obstacles.remove(0);
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(5, 25);
        self.frame_time = 0.0;
        self.obstacles = vec![Obstacle::new(SCREEN_WIDTH)];
        self.mode = GameMode::Playing;
        self.score = 0;
    }

    fn continue_game(&mut self) {
        self.mode = GameMode::Playing;
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn pause_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Pause!");
        ctx.print_centered(8, "(ESC) Continue Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Escape => self.continue_game(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::Pause => self.pause_menu(ctx),
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::new()
        .with_font("../resources/flappy32.png", 32, 32)
        .with_simple_console(SCREEN_WIDTH, SCREEN_HEIGHT, "../resources/flappy32.png")
        .with_fancy_console(SCREEN_WIDTH, SCREEN_HEIGHT, "../resources/flappy32.png")
        .with_title("Flappy dragon")
        .with_tile_dimensions(16, 16)
        .build()?;

    main_loop(context, State::new())
}
