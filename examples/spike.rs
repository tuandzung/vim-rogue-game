use bracket_lib::prelude::*;

struct State;

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print(1, 1, "Hello bracket-lib!");
        ctx.print_color(1, 2, RGB::named(YELLOW), RGB::named(BLACK), "Colored text");

        if let Some(key) = ctx.key {
            if key == VirtualKeyCode::Escape {
                ctx.quitting = true;
            }
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("vim-quake spike")
        .with_vsync(false)
        .with_fps_cap(30.0)
        .build()?;
    main_loop(context, State)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_console_uses_80_by_50_grid() {
        let console = SimpleConsole::init(80, 50);

        assert_eq!(console.width, 80);
        assert_eq!(console.height, 50);
        assert_eq!(console.tiles.len(), 80 * 50);
    }

    #[test]
    fn print_writes_glyph_at_coordinates() {
        let mut console = SimpleConsole::init(80, 50);
        console.print(3, 4, "A");

        let tile = &console.tiles[(((console.height - 1 - 4) * console.width) + 3) as usize];
        assert_eq!(tile.glyph, to_cp437('A'));
    }

    #[test]
    fn print_color_writes_glyph_and_colors_at_coordinates() {
        let mut console = SimpleConsole::init(80, 50);
        let fg: RGBA = RGB::named(YELLOW).into();
        let bg: RGBA = RGB::named(BLACK).into();

        console.print_color(5, 6, fg, bg, "B");

        let tile = &console.tiles[(((console.height - 1 - 6) * console.width) + 5) as usize];
        assert_eq!(tile.glyph, to_cp437('B'));
        assert_eq!(tile.fg, fg);
        assert_eq!(tile.bg, bg);
    }

    #[test]
    #[ignore = "opens a native window; run locally to smoke-test builder context creation"]
    fn builder_smoke_creates_an_80_by_50_context() {
        let _context = BTermBuilder::simple80x50()
            .with_title("vim-quake spike")
            .with_vsync(false)
            .with_fps_cap(30.0)
            .build()
            .expect("context should build");
    }
}
