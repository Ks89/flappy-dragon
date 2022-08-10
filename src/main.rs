use bracket_lib::prelude::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 40.0;

// |    0,0  1,0  2,0  3,0  4,0
// |    0,1  1,1  2,1  3,1  4,1
// |    0,2  1,2  2,2  3,2  4,2
// y __________________________> x

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

        let mut random = RandomNumberGenerator::new();

        // add new obstacles with a certain percentage
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
            if self.player.y > SCREEN_HEIGHT || obstacle.hit_obstacle(&self.player) {
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

struct Player {
    x: i32,
    y: i32,
    velocity: f32,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(5, self.y, YELLOW, BLACK, to_cp437('@'));
    }

    fn gravity_and_mode(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }
        self.y += self.velocity as i32;
        self.x += 1;
        if self.y < 0 {
            self.y = 0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.0; // a negative number, so it moves upward, because 0 is the top of the screeen
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
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;

        // draw the top half of the obstacle
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, WHITE, NAVY, 179);
        }

        // draw the bottom half of the obstacle
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x, y, WHITE, NAVY, 179);
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x == self.x;
        let player_above_gap = player.y < self.gap_y - half_size;
        let player_below_gap = player.y > self.gap_y + half_size;
        does_x_match && (player_above_gap || player_below_gap)
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
