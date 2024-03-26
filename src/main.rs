use macroquad::audio::*;
/**
 * Notes for what macroquad needs that raylib provides:
 * - Pitch shifting for sounds
 * - Collision helpers
 * - Crisp font like raylib
 * - More predictable text placement? (Edit: looks like `measure_text` is meant to make this easy)
 * - Circles need more facets
 *
 * Pros:
 * - Draw lines has a thickness field
 */
use macroquad::prelude::*;
use macroquad::rand::{rand, srand};

// Don't use this! It's broken for ints
// https://github.com/not-fl3/quad-rand/issues/12
// use macroquad::rand::RandomRange;

const REGULAR_FONT_SIZE: f32 = 40.;

#[derive(Copy, Clone)]
struct State {
    score: u8,
}

fn save_state(state: &State) {
    let storage = &mut quad_storage::STORAGE.lock().unwrap();
    storage.set("score", &state.score.to_string());
}

fn load_state() -> State {
    let storage = &mut quad_storage::STORAGE.lock().unwrap();
    match storage.get("score") {
        Some(score_str) => match score_str.parse::<u8>() {
            Ok(score) => State { score },
            _ => State { score: 0 },
        },
        None => State { score: 0 },
    }
}

fn vec2_from_tuple(xy: (f32, f32)) -> Vec2 {
    Vec2::new(xy.0, xy.1)
}

fn check_collision_point_rect(point: Vec2, rect: Rect) -> bool {
    rect.x <= point.x
        && point.x <= rect.x + rect.w
        && rect.y <= point.y
        && point.y <= rect.y + rect.h
}

fn check_collision_point_tri(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    // Get the barycentric coordinates and check that they're within the triangle
    // TODO: Maybe this could be faster if the coordinates weren't normalized
    // to the size of the triangle?
    let inv_triangle_area = ((b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y)).recip();

    let bary_a_area = (b.y - c.y) * (p.x - c.x) + (c.x - b.x) * (p.y - c.y);
    let bary_b_area = (c.y - a.y) * (p.x - c.x) + (a.x - c.x) * (p.y - c.y);

    let bary_a = bary_a_area * inv_triangle_area;
    let bary_b = bary_b_area * inv_triangle_area;
    let bary_c = 1. - bary_a - bary_b;

    (bary_a > 0.) && (bary_b > 0.) && (bary_c > 0.)
}

fn draw_text_centered(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    let ufont_size: u16 = font_size.round() as u16;
    let size = measure_text(text, None, ufont_size, 1.0);
    draw_text(
        text,
        x - size.width / 2.,
        y - size.height / 2.,
        font_size,
        color,
    );
}

#[derive(Copy, Clone, PartialEq)]
enum Scene {
    ClickToStart,
    MainMenu,
    Game,
    Credits,
    Score,
}

#[macroquad::main("Rust Simon")]
async fn main() {
    let screen_width: f32 = 800.;
    let screen_height: f32 = 450.;
    let clear_color = Color::from_rgba(245, 245, 245, 255);
    request_new_screen_size(screen_width, screen_height);

    let sound0 = load_sound("assets/drop_003_p0.ogg").await.unwrap();
    let sound1 = load_sound("assets/drop_003_p1.ogg").await.unwrap();
    let sound2 = load_sound("assets/drop_003_p2.ogg").await.unwrap();
    let sound3 = load_sound("assets/drop_003_p3.ogg").await.unwrap();

    srand(macroquad::miniquad::date::now() as u64);

    let mut game_state = load_state();

    let mut scene = Scene::ClickToStart;

    let mut order = [0u8; 256];
    let mut order_max_idx: usize = 0;
    let mut user_idx: usize = 0;

    let mut scene_first_frame = true;

    let mut game_anim_start_time: f64 = 0.0;

    let mut most_recent_score: u8 = 0;

    let mut score_scene_is_new_high_score = false;
    let mut score_scene_old_high_score: u8 = 0;

    // Mutable version of the last_frame_time that shouldn't be read by scenes
    let mut _scene_ignore_last_frame_time = get_time();

    loop {
        let last_frame_time = _scene_ignore_last_frame_time;
        let current_frame_time = get_time();

        let frame_start_scene = scene;
        let mouse_pos = vec2_from_tuple(mouse_position());

        clear_background(clear_color);

        match scene {
            Scene::ClickToStart => {
                draw_text_centered(
                    "Click anywhere to begin",
                    screen_width / 2.,
                    screen_height / 2.,
                    REGULAR_FONT_SIZE,
                    BLACK,
                );
                if is_mouse_button_released(MouseButton::Left) {
                    scene = Scene::MainMenu;
                }
            }
            Scene::MainMenu => {
                let ufont_size: u16 = REGULAR_FONT_SIZE.round() as u16;

                let start_text = "Start Game";
                let start_text_width = measure_text(start_text, None, ufont_size, 1.0).width;
                let start_rect = Rect {
                    x: (screen_width - start_text_width) / 2. - 5.,
                    y: 150.,
                    w: start_text_width + 10.,
                    h: REGULAR_FONT_SIZE + 10.,
                };

                if check_collision_point_rect(mouse_pos, start_rect) {
                    draw_rectangle(
                        start_rect.x - 5.,
                        start_rect.y - 5.,
                        start_rect.w + 10.,
                        start_rect.h + 10.,
                        DARKGREEN,
                    );
                    if is_mouse_button_released(MouseButton::Left) {
                        scene = Scene::Game;
                    }
                }
                draw_rectangle(
                    start_rect.x,
                    start_rect.y,
                    start_rect.w,
                    start_rect.h,
                    GREEN,
                );
                draw_text(
                    start_text,
                    start_rect.x + 5.,
                    start_rect.y + start_rect.h - 14.,
                    REGULAR_FONT_SIZE,
                    BLACK,
                );

                let credits_text = "Credits";
                let credits_text_width = measure_text(credits_text, None, ufont_size, 1.0).width;
                let credit_rect = Rect {
                    x: (screen_width - credits_text_width) / 2. - 5.,
                    y: 150. + REGULAR_FONT_SIZE * 1.5,
                    w: credits_text_width + 10.,
                    h: REGULAR_FONT_SIZE + 10.,
                };

                if check_collision_point_rect(mouse_pos, credit_rect) {
                    draw_rectangle(
                        credit_rect.x - 5.,
                        credit_rect.y - 5.,
                        credit_rect.w + 10.,
                        credit_rect.h + 10.,
                        DARKBLUE,
                    );
                    if is_mouse_button_released(MouseButton::Left) {
                        scene = Scene::Credits;
                    }
                }
                draw_rectangle(
                    credit_rect.x,
                    credit_rect.y,
                    credit_rect.w,
                    credit_rect.h,
                    BLUE,
                );
                draw_text(
                    credits_text,
                    credit_rect.x + 5.,
                    credit_rect.y + credit_rect.h - 14.,
                    REGULAR_FONT_SIZE,
                    BLACK,
                );

                let score_text = format!("High Score: {}", game_state.score);
                draw_text(
                    &score_text,
                    5.,
                    screen_height - 10.,
                    REGULAR_FONT_SIZE,
                    BLACK,
                );
            }
            Scene::Game => {
                if scene_first_frame {
                    user_idx = 0;
                    order_max_idx = 1;
                    for value in order.iter_mut() {
                        *value = (rand() >> 30) as u8;
                    }

                    game_anim_start_time = current_frame_time;
                }

                let red_gray = Color::from_rgba(229, 193, 197, 255);
                let green_gray = Color::from_rgba(192, 226, 199, 255);
                let blue_gray = Color::from_rgba(203, 222, 239, 255);
                let yellow_gray = Color::from_rgba(252, 251, 214, 255);

                let center = Vec2::new(screen_width / 2., screen_height / 2.);
                let tl = Vec2::new(0., 0.);
                let tr = Vec2::new(screen_width, 0.);
                let br = Vec2::new(screen_width, screen_height);
                let bl = Vec2::new(0., screen_height);
                draw_triangle(center, tr, tl, red_gray);
                draw_triangle(center, br, tr, green_gray);
                draw_triangle(center, bl, br, blue_gray);
                draw_triangle(center, tl, bl, yellow_gray);

                draw_circle(
                    screen_width / 2.,
                    80.,
                    55.,
                    Color::from_rgba(190, 22, 35, 255),
                );
                draw_circle(screen_width - 80., screen_height / 2., 55., DARKGREEN);
                draw_circle(screen_width / 2., screen_height - 80., 55., DARKBLUE);
                draw_circle(
                    80.,
                    screen_height / 2.,
                    55.,
                    Color::from_rgba(191, 187, 2, 255),
                );

                let animation_length = order_max_idx as f64 + 0.5;
                let show_pattern = current_frame_time < game_anim_start_time + animation_length;

                if show_pattern {
                    draw_text_centered(
                        "Memorize",
                        screen_width / 2.,
                        screen_height / 2.,
                        40.,
                        BLACK,
                    );

                    let last_t = last_frame_time - game_anim_start_time;
                    let t = current_frame_time - game_anim_start_time;

                    let last_anim_idx = last_t.floor() - 1.;
                    let anim_idx = t.floor() - 1.;
                    if anim_idx >= 0. {
                        let idx = anim_idx as usize;

                        // We have passed from one second to the next, so we play a new sound
                        if last_anim_idx != anim_idx {
                            match order[idx] {
                                0 => play_sound_once(&sound0),
                                1 => play_sound_once(&sound1),
                                2 => play_sound_once(&sound2),
                                _ => play_sound_once(&sound3),
                            }
                        }

                        match order[idx] {
                            0 => draw_circle(screen_width / 2., 80., 50., RED),
                            1 => draw_circle(screen_width - 80., screen_height / 2., 50., GREEN),
                            2 => draw_circle(screen_width / 2., screen_height - 80., 50., BLUE),
                            _ => draw_circle(80., screen_height / 2., 50., YELLOW),
                        }
                    }
                } else {
                    let mut hover_tri: Option<u8> = None;
                    if check_collision_point_tri(mouse_pos, center, tr, tl) {
                        hover_tri = Some(0);
                        draw_triangle_lines(center, tr, tl, 5., BLACK);
                    } else if check_collision_point_tri(mouse_pos, center, br, tr) {
                        hover_tri = Some(1);
                        draw_triangle_lines(center, br, tr, 5., BLACK);
                    } else if check_collision_point_tri(mouse_pos, center, bl, br) {
                        hover_tri = Some(2);
                        draw_triangle_lines(center, bl, br, 5., BLACK);
                    } else if check_collision_point_tri(mouse_pos, center, tl, bl) {
                        hover_tri = Some(3);
                        draw_triangle_lines(center, tl, bl, 5., BLACK);
                    }

                    if let Some(tri) = hover_tri {
                        if is_mouse_button_released(MouseButton::Left) {
                            if tri == order[user_idx] {
                                match tri {
                                    0 => play_sound_once(&sound0),
                                    1 => play_sound_once(&sound1),
                                    2 => play_sound_once(&sound2),
                                    _ => play_sound_once(&sound3),
                                }
                                match tri {
                                    0 => draw_circle(screen_width / 2., 80., 50., RED),
                                    1 => draw_circle(
                                        screen_width - 80.,
                                        screen_height / 2.,
                                        50.,
                                        GREEN,
                                    ),
                                    2 => draw_circle(
                                        screen_width / 2.,
                                        screen_height - 80.,
                                        50.,
                                        BLUE,
                                    ),
                                    _ => draw_circle(80., screen_height / 2., 50., YELLOW),
                                }
                                user_idx += 1;

                                if user_idx == order_max_idx {
                                    user_idx = 0;
                                    order_max_idx += 1;
                                    game_anim_start_time = current_frame_time;
                                }
                            } else {
                                play_sound_once(&sound0);
                                play_sound_once(&sound1);
                                play_sound_once(&sound2);
                                play_sound_once(&sound3);
                                most_recent_score = (order_max_idx - 1) as u8;
                                scene = Scene::Score;
                            }
                        }
                    }
                }
            }
            Scene::Score => {
                if scene_first_frame {
                    score_scene_old_high_score = game_state.score;
                    score_scene_is_new_high_score = false;
                    if game_state.score < most_recent_score {
                        score_scene_is_new_high_score = true;
                        game_state.score = most_recent_score;
                        save_state(&game_state);
                    }
                }

                let score_text = format!("Score: {}", most_recent_score);
                draw_text_centered(
                    &score_text,
                    screen_width / 2.,
                    screen_height / 2.,
                    40.,
                    BLACK,
                );

                let high_score_text = if score_scene_is_new_high_score {
                    let wave_offset = 20.0 * (get_time() * 4.0).sin() as f32;
                    draw_text_centered(
                        "NEW HIGH SCORE!",
                        screen_width / 2.,
                        screen_height / 2. - 100. + wave_offset,
                        50.,
                        RED,
                    );

                    format!("Old High Score: {}", score_scene_old_high_score)
                } else {
                    format!("High Score: {}", score_scene_old_high_score)
                };
                draw_text_centered(
                    &high_score_text,
                    screen_width / 2.,
                    screen_height / 2. + 50.,
                    20.,
                    BLACK,
                );

                draw_text_centered(
                    "Click to Return",
                    screen_width / 2.,
                    screen_height / 2. + 150.,
                    30.,
                    BLACK,
                );

                if is_mouse_button_released(MouseButton::Left) {
                    scene = Scene::MainMenu;
                }
            }
            Scene::Credits => {
                draw_text_centered(
                    "Game by Stephen Molyneaux 2024",
                    screen_width / 2.,
                    screen_height / 2.,
                    40.,
                    BLACK,
                );
                draw_text_centered(
                    "Developed with Macroquad",
                    screen_width / 2.,
                    screen_height / 2. + 40.,
                    20.,
                    BLACK,
                );
                if is_mouse_button_released(MouseButton::Left) {
                    scene = Scene::MainMenu;
                }
            }
        }

        scene_first_frame = frame_start_scene != scene;
        _scene_ignore_last_frame_time = get_time();

        next_frame().await
    }
}
