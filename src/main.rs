use std::{f32::consts::PI, mem::take, ops::Index, sync::{Arc, Mutex, MutexGuard}};

use macroquad::{audio::{load_sound_from_bytes, play_sound}, prelude::{rand, *}, rand::{ChooseRandom, RandomRange, gen_range, rand}};

#[derive(PartialEq, Clone, Copy)]
enum EnemyType {
    Normal,
    Seeking,
}

impl RandomRange for EnemyType {
    fn gen_range(low: Self, high: Self) -> Self {
        let enemy_type = rand::gen_range(low as usize, high as usize + 5);
        
        if enemy_type == 0 {
            return Self::Seeking
        } else {
            return Self::Normal
        }
    }

    fn gen_range_with_state(_state: &rand::RandGenerator, low: Self, high: Self) -> Self {
        EnemyType::gen_range(low, high)
    }
}

#[derive(Clone, Copy)]
struct Enemy {
    scale: u8,
    _type: EnemyType,
    position: Vec2,    
}


const ENEMY_SIZE: f32 = 64.;
const PLAYER_SIZE: f32 = 64.;
const HITBOX_ADJUSTMENT: f32 = 10.;
const GENERATION_BUFFER_Y: i32 = 300;
const FALL_SPEED: f32 = 0.3;
const IMPACT_FORCE: Vec2 = vec2(24., 36.);
const SLOWDOWN_SPEED: f32 = 0.9;
const INITIAL_HEIGHT: i32 = 800;

fn window_conf() -> Conf {
    Conf {
        window_title: "Bouncy Ball".to_owned(),
        window_resizable: true,
        window_height: INITIAL_HEIGHT,
        window_width: 600,
        ..Default::default()
    }
}

fn handle_texture(bytes: &'static [u8]) -> Texture2D {
    let texture = Texture2D::from_image(&Image::from_file_with_format(bytes, None).unwrap());
    texture.set_filter(FilterMode::Nearest);

    return texture;
}


struct SkyObject {
    rotation: f32,
    texture_index: i32
}
struct SkyTile {
    stars: SkyObject,
    objects: SkyObject,
    y: f32
}

fn randomize_rotation() -> f32 {
    return 2.*PI/4. * rand::gen_range(0, 4) as f32;
}

fn randomize_texture_index() -> i32 {
    return rand::gen_range(0, 2);
}

fn create_sky_object() -> SkyObject {
    return SkyObject { rotation: randomize_rotation(), texture_index:  randomize_texture_index()}
}

fn create_sky_tiles() -> Vec<SkyTile> {
    let mut tiles: Vec<SkyTile> = vec![];

    for i in 0..6 {
        tiles.push(SkyTile { stars: create_sky_object(), objects: create_sky_object(), y: -(i as f32*screen_width()) });
    }

    return tiles;
}

#[macroquad::main(window_conf)]
async fn main() {
    let (screen_w, mut screen_h) = (screen_width().min(600.), screen_height());

    let ball_texture = handle_texture(include_bytes!("assets/ball.png"));
    let enemy_normal_texture = handle_texture(include_bytes!("assets/enemy.png"));
    let enemy_seeking_texture = handle_texture(include_bytes!("assets/enemy_seeking.png"));

    let star_textures = vec![handle_texture(include_bytes!("assets/background/star_pattern_1.png")), handle_texture(include_bytes!("assets/background/star_pattern_2.png"))];
    let sky_object_textures = vec![handle_texture(include_bytes!("assets/background/sky_object_1.png")), handle_texture(include_bytes!("assets/background/sky_object_2.png"))];
    let sky_texture = handle_texture(include_bytes!("assets/background/static_bg.png"));

    let music = load_sound_from_bytes(include_bytes!("assets/sick_ass_beat.wav")).await.unwrap();
    let hit_wall = load_sound_from_bytes(include_bytes!("assets/swoosh.wav")).await.unwrap();

    let mut ball_coords = vec2(300., 0.);
    let mut ball_velocity = vec2(0., 0.);

    let (mut enemies, mut last_ball_spawn_y) = create_enemies(vec![], -300., 30).await;

    let mut camera = Camera2D {
        zoom: vec2(2. / screen_w, 2. / screen_h),
        viewport: Some(((screen_width() - screen_w) as i32 / 2, 0, screen_w as i32, screen_h as i32)),
        ..Default::default()
    };

    let mut camera_applied_dimensions = (screen_w, screen_h);

    let mut animation_tick = 0;
    let mut animation_reversing: bool = false;

    play_sound(&music, macroquad::audio::PlaySoundParams { looped: true, volume: 0.4 });

    let mut score: i64 = 0;
    let mut highest_y: f64 = 0.;

    let mut sky_tiles: Vec<SkyTile> = create_sky_tiles();
    let mut highest_sky_tile = -3.*screen_w;
    let mut lowest_sky_tile = 0.;

    let mut dead = false;
    let mut death_opacity = 0.;

    loop {
        set_camera(&camera);
        clear_background(BLACK);


        let screen_x_offset = (screen_width()-screen_w) as i32 / 2;

        if camera_applied_dimensions != (screen_width(), screen_height()) {
            screen_h = screen_height();
            camera = Camera2D {
                zoom: vec2(2. / screen_w, 2. / screen_h),
                viewport: Some((screen_x_offset, 0, screen_w as i32, screen_h as i32)),
                ..Default::default()
            };

            camera_applied_dimensions = (screen_width(), screen_height());
        }

        let (mouse_x, mouse_y) = (
            (mouse_position().0 - screen_x_offset as f32).max(0.).min(screen_w),
             mouse_position().1
        );
        let relative_mouse = vec2(mouse_x, camera.target.y + (mouse_y - screen_h / 2.));


        let sky_dimensions = vec2(screen_w, screen_w);

        for tile in &mut sky_tiles {
            println!("{} | {}", tile.y, highest_sky_tile);
            if lowest_sky_tile < camera.target.y - screen_height()/2. && tile.y == highest_sky_tile {
                tile.y = lowest_sky_tile+sky_dimensions.y;
                lowest_sky_tile = tile.y;
                highest_sky_tile += sky_dimensions.y;
            }
            
            if tile.y > camera.target.y + screen_height()/2. + sky_dimensions.y {
                tile.y = highest_sky_tile - sky_dimensions.y;

                tile.stars.rotation = randomize_rotation();
                tile.stars.texture_index = randomize_texture_index();

                tile.objects.rotation = randomize_rotation();
                tile.objects.texture_index = randomize_texture_index();

                highest_sky_tile = tile.y;
            }


            let object_offset = (ball_coords.y - tile.y)/10.;
            let star_offset = (ball_coords.y - tile.y)/8.;


            draw_texture_ex(&sky_texture, 0., tile.y, WHITE, DrawTextureParams { dest_size: Some(sky_dimensions),..Default::default() });

            draw_texture_ex(&sky_object_textures[tile.objects.texture_index as usize], 0., tile.y - object_offset, WHITE, DrawTextureParams { dest_size: Some(sky_dimensions),  rotation: tile.objects.rotation, ..Default::default() });
            draw_texture_ex(&star_textures[tile.stars.texture_index as usize], 0., tile.y - star_offset, WHITE, DrawTextureParams { dest_size: Some(sky_dimensions),  rotation: tile.stars.rotation, ..Default::default() });
        }

        println!();


        draw_line(0., camera.target.y-screen_h, 0., camera.target.y+screen_h, 4., BLUE);
        draw_line(screen_w, camera.target.y-screen_h, screen_w, camera.target.y+screen_h, 4., BLUE);


        ball_coords.y -= ball_velocity.y * FALL_SPEED;
        ball_coords.x += ball_velocity.x;

        draw_text(&(-score).to_string(), 90., (highest_y as f32 - 32.).clamp(ball_coords.y - (screen_h/2.), f32::MAX) + 64., 64., WHITE);

        draw_texture_ex(&ball_texture, ball_coords.x, ball_coords.y, WHITE, DrawTextureParams { dest_size: Some(vec2(PLAYER_SIZE, PLAYER_SIZE)), ..Default::default() });

        let hit_strength = (relative_mouse.distance(ball_coords)/screen_w).max(0.);
        
        draw_line(
            ball_coords.x + PLAYER_SIZE / 2.0,
            ball_coords.y + PLAYER_SIZE / 2.0,
            relative_mouse.x, 
            relative_mouse.y, 
            hit_strength*4., 
            Color { r: 0., g: 0.48, b: 1., a: hit_strength}
        );
        

        if is_mouse_button_pressed(MouseButton::Left) && !dead {
            let dx = ball_coords.x - mouse_x;
            let dy = (ball_coords.y-camera.target.y) - mouse_y + screen_h/2. + PLAYER_SIZE/2.;

            let len: f32 = (dx*dx + dy*dy).sqrt();

            let ball_vel_x = dx / len * IMPACT_FORCE.x;
            let ball_vel_y = dy / len * IMPACT_FORCE.y;

            ball_velocity = vec2(ball_vel_x, -ball_vel_y);
        } 


        let ball_left = ball_coords.x + HITBOX_ADJUSTMENT;
        let ball_right = ball_coords.x + PLAYER_SIZE - HITBOX_ADJUSTMENT;
        let ball_top = ball_coords.y + HITBOX_ADJUSTMENT;
        let ball_bottom = ball_coords.y + PLAYER_SIZE - HITBOX_ADJUSTMENT;
    
        for enemy in &mut enemies {
            if enemy.position.y > ball_coords.y + screen_h || enemy.position.y < ball_coords.y - screen_h {
                continue;
            }

            let scale_float = enemy.scale as f32;
            
            match &enemy._type {
                EnemyType::Normal => {
                    if animation_reversing {
                        enemy.position.x -= 0.1 * get_frame_time();
                    } else {
                        enemy.position.x += 0.1 * get_frame_time();
                    }


                    draw_texture_ex(
                        &enemy_normal_texture,
                        enemy.position.x * screen_w,
                        enemy.position.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(ENEMY_SIZE*scale_float, ENEMY_SIZE*scale_float)), ..Default::default() 
                    });
                }
                EnemyType::Seeking => {
                    if enemy.position.x * screen_w > ball_coords.x {
                        enemy.position.x -= 0.1 * get_frame_time();
                    } else {
                        enemy.position.x += 0.1 * get_frame_time();
                    }

                    if enemy.position.y < ball_coords.y {
                        enemy.position.y += 50. * get_frame_time();
                    } else {
                        enemy.position.y -= 50. * get_frame_time();
                    }
                    
                    draw_texture_ex(
                        &enemy_seeking_texture,
                        enemy.position.x * screen_w,
                        enemy.position.y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(ENEMY_SIZE*scale_float, ENEMY_SIZE*scale_float)), ..Default::default() 
                    });
                }
                _ => {}
            }

            let adjusted_enemy_x = enemy.position.x * screen_w;

            let left_side = adjusted_enemy_x + HITBOX_ADJUSTMENT*scale_float;
            let right_side = adjusted_enemy_x + ENEMY_SIZE*scale_float - HITBOX_ADJUSTMENT*scale_float;
            let top = enemy.position.y + HITBOX_ADJUSTMENT*scale_float;
            let bottom = enemy.position.y + ENEMY_SIZE*scale_float - HITBOX_ADJUSTMENT*scale_float;

            let vertical_check = ball_top < bottom && ball_bottom > top;
            let horizontal_check = ball_right > left_side && ball_left < right_side;

            if vertical_check && horizontal_check {
                dead = true;
            }

            draw_rectangle(0., highest_y as f32, screen_w, 4., Color 
                { r: 1., g: 1., b: 1., a: ((((ball_coords.y as f64 - highest_y) as f32) / 10000.).max(0.)).min(0.4) }
            );


            draw_text(&format!("{}, {}", vertical_check, horizontal_check), 0., ball_coords.y + screen_h/2. - 90., 16., WHITE);

            draw_rectangle(ball_left, ball_top, 4., 4., RED);
            draw_rectangle(ball_right, ball_top, 4., 4., RED);
            draw_rectangle(ball_left, ball_bottom, 4., 4., RED);
            draw_rectangle(ball_right, ball_bottom, 4., 4., RED);

            draw_rectangle(left_side, top, 4., 4., BLUE);
            draw_rectangle(left_side, bottom, 4., 4., BLUE);
            draw_rectangle(right_side, top, 4., 4., BLUE);
            draw_rectangle(right_side, bottom, 4., 4., BLUE);
        }


        draw_text(&format!("{} fps {} ms", get_fps(), get_frame_time()), 0., ball_coords.y - screen_h/2. + 32., 16., WHITE);
        draw_text(&format!("#{} animation tick, direction: {}", animation_tick, animation_reversing as u8), 0., ball_coords.y - screen_h/2. + 48., 16., WHITE);
        draw_text(&format!("{},{} | {}, {}", mouse_x, mouse_y, camera.target.x, camera.target.y), 0., ball_coords.y - screen_h/2. + 80., 16., WHITE);

        
        if ball_coords.y < last_ball_spawn_y + GENERATION_BUFFER_Y as f32 {
            (enemies, last_ball_spawn_y) = create_enemies(enemies.clone(), last_ball_spawn_y, 90).await;
        }

        ball_velocity.y -= 75. * get_frame_time();
        ball_velocity.x *= SLOWDOWN_SPEED;

                
        if ball_coords.x < -HITBOX_ADJUSTMENT || ball_coords.x + PLAYER_SIZE + HITBOX_ADJUSTMENT > screen_w {
            play_sound(&hit_wall, macroquad::audio::PlaySoundParams { looped: false, volume: 1.0 });
            ball_velocity.x *= -2.;
            score /= 2;
            score += 100;
        }

        if dead {
            draw_rectangle(0., camera.target.y - screen_h/2., screen_w, screen_h, Color { r: 0., g: 0., b: 0., a: death_opacity });
            draw_text("You died!!", 16., camera.target.y, 128., WHITE);
            draw_text(&format!("Final score: {}", -score), 16., camera.target.y+64., 38., WHITE);

            if death_opacity < 1. {
                death_opacity += 0.5 * get_frame_time();
            }
        }

        camera.target = vec2(screen_w/2., ball_coords.y);
        
        if animation_reversing {
            animation_tick -= 1;
        } else {
            animation_tick += 1;
        }

        if animation_tick >= 255 {
            animation_reversing = true;
        }
        if animation_tick <= 0 {
            animation_reversing = false;
        }

        if (ball_coords.y as f64) < highest_y {
            score += (ball_coords.y as f64 - highest_y) as i64;
            highest_y = ball_coords.y as f64;
        }

        next_frame().await;
    }
}

async fn create_enemies(mut current: Vec<Enemy>, from_y: f32, amount: u32) -> (Vec<Enemy>, f32) {
    println!("Creating new enemies");

    let mut last_y = from_y;
    for _i in 0..amount {
        let generated_y = rand::gen_range(last_y - ENEMY_SIZE - 150., last_y - ENEMY_SIZE);

        current.push({Enemy { 
                position: vec2(
                    rand::gen_range(0.1, 0.9),
                    generated_y,
                ),
                _type: rand::gen_range(EnemyType::Normal, EnemyType::Seeking),
                scale: rand::gen_range(1, 3)
            }});

        last_y = generated_y;
    }

    return (current, last_y)
}