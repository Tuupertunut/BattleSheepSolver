use battle_sheep_solver::board::{add_offset, Board, Player, Tile, TileType, DIRECTION_OFFSETS};
use eframe::{
    egui::{self, CentralPanel, Painter, Sense},
    emath::Align2,
    epaint::{pos2, vec2, Color32, FontId, Pos2, Rect, Shape, Stroke},
};
use egui_extras::RetainedImage;

fn main() {
    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(vec2(1200.0, 800.0));
    eframe::run_native(
        "Battle Sheep UI",
        options,
        Box::new(|_cc| Box::new(BattleSheepApp::new())),
    )
    .unwrap();
}

struct HoverStack {
    stack: Tile,
    origin: Option<(isize, isize)>,
}

struct BattleSheepApp {
    board: Board,
    hover_stack: Option<HoverStack>,
    home_stacks: [Option<Tile>; Player::PLAYER_COUNT as usize],
    red_image: RetainedImage,
    blue_image: RetainedImage,
}

impl BattleSheepApp {
    fn new() -> Self {
        return Self {
            board: Board {
                tiles: vec![Tile::EMPTY],
                row_length: 1,
            },
            hover_stack: None,
            home_stacks: Player::iter()
                .map(|player| Some(Tile::stack(player, 16)))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            red_image: RetainedImage::from_image_bytes(
                "redsheep.png",
                include_bytes!("redsheep.png"),
            )
            .unwrap(),
            blue_image: RetainedImage::from_image_bytes(
                "bluesheep.png",
                include_bytes!("bluesheep.png"),
            )
            .unwrap(),
        };
    }

    const TILE_COLOR: Color32 = Color32::GREEN;
    const HIGHLIGHT_COLOR: Color32 = Color32::from_rgb(0, 255, 180);
    const PATH_HIGHLIGHT_COLOR: Color32 = Color32::from_rgb(140, 220, 0);

    fn draw_empty_tile(&self, painter: &Painter, middle_point: Pos2, height: f32, color: Color32) {
        let quarter_height = height / 4.0;
        let half_width = f32::sqrt(3.0) * quarter_height;
        painter.add(Shape::convex_polygon(
            vec![
                middle_point + vec2(0.0, -2.0 * quarter_height),
                middle_point + vec2(half_width, -quarter_height),
                middle_point + vec2(half_width, quarter_height),
                middle_point + vec2(0.0, 2.0 * quarter_height),
                middle_point + vec2(-half_width, quarter_height),
                middle_point + vec2(-half_width, -quarter_height),
            ],
            color,
            Stroke::new(height * 0.08, Color32::DARK_GREEN),
        ));
    }

    fn draw_stack(
        &self,
        ctx: &egui::Context,
        painter: &Painter,
        middle_point: Pos2,
        height: f32,
        player: Player,
        stack_size: u8,
    ) {
        let image = match player {
            Player(0) => &self.red_image,
            Player(1) => &self.blue_image,
            _ => unreachable!(),
        };
        painter.image(
            image.texture_id(ctx),
            Rect::from_center_size(middle_point, vec2(height * 0.65, height * 0.65)),
            Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        painter.text(
            middle_point,
            Align2::CENTER_CENTER,
            format!("{}", stack_size),
            FontId::proportional(height * 0.5),
            Color32::WHITE,
        );
    }
}

impl eframe::App for BattleSheepApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.label("text");
            let (canvas, painter) =
                ui.allocate_painter(ui.available_size() - vec2(0.0, 20.0), Sense::drag());

            let board_rows = self.board.num_rows();
            let mut first_half_column = 0;
            let mut last_half_column = 0;
            for ((r, q), tile) in self.board.iter_row_major() {
                if tile.is_board_tile() {
                    first_half_column = isize::min(first_half_column, q * 2 - r - 1);
                    last_half_column = isize::max(last_half_column, q * 2 - r + 1)
                }
            }
            let board_half_columns = last_half_column - first_half_column;
            let board_size_heights = vec2(
                board_half_columns as f32 * f32::sqrt(3.0) / 4.0,
                (board_rows as f32 * 3.0 + 1.0) / 4.0,
            );

            let ideal_by_x = canvas.rect.width() / (board_size_heights.x + 2.0);
            let ideal_by_y = canvas.rect.height() / (board_size_heights.y + 3.0);

            let height = f32::min(ideal_by_x, ideal_by_y);
            let grid_start = canvas.rect.min
                + vec2(
                    height * (1.0 - first_half_column as f32 * f32::sqrt(3.0) / 4.0),
                    height * 1.5,
                );

            for (hex_coords, tile) in self.board.iter_row_major() {
                if tile.is_board_tile() {
                    let middle_point = hex_to_middle_point(hex_coords, grid_start, height);

                    self.draw_empty_tile(&painter, middle_point, height, Self::TILE_COLOR);

                    if tile.is_stack() {
                        self.draw_stack(
                            ctx,
                            &painter,
                            middle_point,
                            height,
                            tile.player(),
                            tile.stack_size(),
                        );
                    }
                }
            }

            for player in Player::iter() {
                let player_id = player.id() as usize;
                let home_stack = self.home_stacks[player_id];

                let home = canvas.rect.center_bottom()
                    + vec2(
                        ((Player::PLAYER_COUNT - 1) as f32 * -0.5 + player_id as f32) * height,
                        -0.5 * height,
                    );
                if let Some(home_stack) = home_stack {
                    self.draw_stack(
                        ctx,
                        &painter,
                        home,
                        height,
                        home_stack.player(),
                        home_stack.stack_size(),
                    );
                }

                if let Some(pointer_pos) = canvas.hover_pos() {
                    /* Did click end on this frame? drag_released() is much like clicked() but without
                     * time or movement limit. */
                    if canvas.drag_released() {
                        if Rect::from_center_size(home, vec2(height, height)).contains(pointer_pos)
                        {
                            match home_stack {
                                Some(home_stack) => {
                                    if let None = self.hover_stack {
                                        self.hover_stack = Some(HoverStack {
                                            stack: home_stack,
                                            origin: None,
                                        });
                                        self.home_stacks[player_id] = None;
                                    }
                                }
                                None => {
                                    if let Some(HoverStack {
                                        stack: hover_stack,
                                        origin: hover_origin,
                                    }) = self.hover_stack
                                    {
                                        if hover_origin == None {
                                            self.home_stacks[player_id] = Some(hover_stack);
                                            self.hover_stack = None;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(pointer_pos) = canvas.hover_pos() {
                let pointer_coords = point_to_hex(pointer_pos, grid_start, height);
                ui.label(format!("{:?}", pointer_coords));

                /* Did click end on this frame? drag_released() is much like clicked() but without
                 * time or movement limit. */
                if canvas.drag_released() {
                    let mut clicked_coords = pointer_coords;
                    let clicked_tile = self.board[clicked_coords];
                    match clicked_tile.tile_type() {
                        TileType::NoTile => {
                            if self
                                .board
                                .iter_row_major()
                                .all(|(_, tile)| !tile.is_stack())
                                && self
                                    .board
                                    .iter_neighbors(clicked_coords)
                                    .any(|(_, tile)| tile.is_board_tile())
                            {
                                /* Extend board to contain the clicked coordinates. If the board is
                                 * extended on the left or top side, all coordinates are shifted by
                                 * an offset. The resulting offset is returned and must be applied
                                 * to all stored coordinates. */
                                let resulting_offset = self.board.extend_to_contain(clicked_coords);

                                clicked_coords = add_offset(clicked_coords, resulting_offset);
                                if let Some(HoverStack {
                                    origin: Some(hover_origin),
                                    ..
                                }) = &mut self.hover_stack
                                {
                                    *hover_origin = add_offset(*hover_origin, resulting_offset);
                                }

                                self.board[clicked_coords] = Tile::EMPTY;
                            }
                        }
                        TileType::Empty => {
                            if let Some(HoverStack {
                                stack: hover_stack,
                                origin: hover_origin,
                            }) = self.hover_stack
                            {
                                match hover_origin {
                                    Some(hover_origin) => {
                                        if self
                                            .board
                                            .iter_empty_straight_line_ends(hover_origin)
                                            .any(|coords| coords == clicked_coords)
                                        {
                                            self.board[clicked_coords] = hover_stack;
                                            self.hover_stack = None;
                                        }
                                    }
                                    None => {
                                        if self
                                            .board
                                            .iter_empty_outer_edge()
                                            .any(|coords| coords == clicked_coords)
                                        {
                                            self.board[clicked_coords] = hover_stack;
                                            self.hover_stack = None;
                                        }
                                    }
                                }
                            }
                        }
                        TileType::Stack => {
                            let stack_size = clicked_tile.stack_size();
                            match self.hover_stack {
                                None => {
                                    if stack_size > 1 {
                                        let half_size = stack_size / 2;
                                        self.hover_stack = Some(HoverStack {
                                            stack: Tile::stack(clicked_tile.player(), half_size),
                                            origin: Some(clicked_coords),
                                        });
                                        self.board[clicked_coords] = Tile::stack(
                                            clicked_tile.player(),
                                            stack_size - half_size,
                                        );
                                    }
                                }
                                Some(HoverStack {
                                    stack: hover_stack,
                                    origin: hover_origin,
                                }) => {
                                    if hover_origin == Some(clicked_coords) {
                                        self.board[clicked_coords] = Tile::stack(
                                            clicked_tile.player(),
                                            stack_size + hover_stack.stack_size(),
                                        );
                                        self.hover_stack = None;
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(HoverStack {
                    stack: hover_stack,
                    origin: hover_origin,
                }) = self.hover_stack
                {
                    let scroll_delta = ui.input(|i| i.scroll_delta);
                    if scroll_delta.y != 0.0 {
                        match hover_origin {
                            Some(hover_origin) => {
                                let hover_origin_stack = self.board[hover_origin];
                                let (new_hover_size, new_origin_size) = if scroll_delta.y > 0.0 {
                                    (
                                        hover_stack.stack_size() + 1,
                                        hover_origin_stack.stack_size() - 1,
                                    )
                                } else {
                                    (
                                        hover_stack.stack_size() - 1,
                                        hover_origin_stack.stack_size() + 1,
                                    )
                                };
                                if new_hover_size >= 1 && new_origin_size >= 1 {
                                    self.hover_stack.as_mut().unwrap().stack =
                                        Tile::stack(hover_stack.player(), new_hover_size);
                                    self.board[hover_origin] =
                                        Tile::stack(hover_origin_stack.player(), new_origin_size);
                                }
                            }
                            None => {
                                let new_hover_size = if scroll_delta.y > 0.0 {
                                    hover_stack.stack_size() + 1
                                } else {
                                    hover_stack.stack_size() - 1
                                };
                                if new_hover_size >= 1 && new_hover_size <= Tile::MAX_STACK_SIZE {
                                    self.hover_stack.as_mut().unwrap().stack =
                                        Tile::stack(hover_stack.player(), new_hover_size);
                                }
                            }
                        }
                    }

                    match hover_origin {
                        Some(hover_origin) => {
                            for &dir in DIRECTION_OFFSETS.iter() {
                                for coords in self.board.iter_empty_straight_line(hover_origin, dir)
                                {
                                    self.draw_empty_tile(
                                        &painter,
                                        hex_to_middle_point(coords, grid_start, height),
                                        height,
                                        Self::PATH_HIGHLIGHT_COLOR,
                                    );
                                }
                            }
                            for coords in self.board.iter_empty_straight_line_ends(hover_origin) {
                                self.draw_empty_tile(
                                    &painter,
                                    hex_to_middle_point(coords, grid_start, height),
                                    height,
                                    Self::HIGHLIGHT_COLOR,
                                );
                            }
                        }
                        None => {
                            for coords in self.board.iter_empty_outer_edge() {
                                self.draw_empty_tile(
                                    &painter,
                                    hex_to_middle_point(coords, grid_start, height),
                                    height,
                                    Self::HIGHLIGHT_COLOR,
                                );
                            }
                        }
                    }

                    self.draw_stack(
                        ctx,
                        &painter,
                        pointer_pos,
                        height,
                        hover_stack.player(),
                        hover_stack.stack_size(),
                    )
                }
            }
        });
    }
}

fn hex_to_middle_point((r, q): (isize, isize), grid_start: Pos2, height: f32) -> Pos2 {
    let quarter_height = height / 4.0;
    let half_width = f32::sqrt(3.0) * quarter_height;
    return grid_start
        + vec2(
            2.0 * half_width * q as f32 - half_width * r as f32,
            3.0 * quarter_height * r as f32,
        );
}

fn point_to_hex(point: Pos2, grid_start: Pos2, height: f32) -> (isize, isize) {
    let quarter_height = height / 4.0;
    let half_width = f32::sqrt(3.0) * quarter_height;

    let point_relative = point - grid_start;

    /* Point coordinates in a rectangular grid of half-columns and rows. */
    let pos_in_grid = vec2(
        point_relative.x / half_width,
        point_relative.y / (3.0 * quarter_height),
    );
    let cell = pos_in_grid.floor();
    let pos_in_cell = pos_in_grid - cell;

    /* Each cell contains either a downward or an upward slope, alternating in a
     * checkerboard pattern. */
    let (slope, intercept) = if (cell.x + cell.y) % 2.0 == 0.0 {
        (-1.0 / 3.0, 2.0 / 3.0) /* Upward slope */
    } else {
        (1.0 / 3.0, 1.0 / 3.0) /* Downward slope */
    };

    /* Is point below slope? */
    let hex_r = if pos_in_cell.y > slope * pos_in_cell.x + intercept {
        cell.y + 1.0
    } else {
        cell.y
    };

    /* Rows are offset by one half-column per row. */
    let hex_q = ((cell.x + hex_r) / 2.0).ceil();

    return (hex_r as isize, hex_q as isize);
}
