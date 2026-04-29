use bracket_lib::prelude::*;

use vim_rogue::game;
use vim_rogue::renderer;
use vim_rogue::types::App;

embedded_resource!(UNICODE_FONT, "../resources/Kjammer_16x16.png");

struct AppWrapper {
    app: App,
}

impl GameState for AppWrapper {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.app.refresh_time();

        if let Some(key) = ctx.key {
            game::handle_key(&mut self.app, key, ctx.shift);
        }

        if self.app.session.game_state == vim_rogue::types::GameState::Quit {
            ctx.quit();
            return;
        }

        game::tick(&mut self.app, f64::from(ctx.frame_time_ms));

        self.app.update_visibility();

        renderer::render(ctx, &self.app);
    }
}

fn main() -> BError {
    link_resource!(UNICODE_FONT, "resources/Kjammer_16x16.png");

    let context = BTermBuilder::new()
        .with_font("Kjammer_16x16.png", 16, 16)
        .with_simple_console(80u32, 50u32, "Kjammer_16x16.png")
        .with_tile_dimensions(16u32, 16u32)
        .with_title("vim-rogue")
        .build()?;
    main_loop(context, AppWrapper { app: App::new() })
}
