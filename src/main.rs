use macroquad::{audio::{load_sound_from_bytes, play_sound}, prelude::{rand, *}};

struct Enemy {
    position: Vec2,
    scale: u8,
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


#[macroquad::main(window_conf)]
async fn main() {
    let (screen_w, mut screen_h) = (screen_width().min(600.), screen_height());

    let ball_texture = Texture2D::from_image(&Image::from_file_with_format(include_bytes!("assets/ball.png"), None).unwrap());
    let enemy_texture = Texture2D::from_image(&Image::from_file_with_format(include_bytes!("assets/enemy.png"), None).unwrap());
    ball_texture.set_filter(FilterMode::Nearest);
    enemy_texture.set_filter(FilterMode::Nearest);

    let music = load_sound_from_bytes(include_bytes!("assets/sick_ass_beat.wav")).await.unwrap();
    let hit_wall = load_sound_from_bytes(include_bytes!("assets/swoosh.wav")).await.unwrap();

    let mut ball_coords = vec2(300., 0.);
    let mut ball_velocity = vec2(0., 0.);

    let (mut enemies, mut highest_y) = create_enemies(vec![], 300., 30).await;

    let mut camera = Camera2D {
        zoom: vec2(2. / screen_w, 2. / screen_h),
        viewport: Some(((screen_width() - screen_w) as i32 / 2, 0, screen_w as i32, screen_h as i32)),
        ..Default::default()
    };

    let mut camera_applied_dimensions = (screen_w, screen_h);

    let mut animation_tick = 0;
    let mut animation_reversing: bool = false;

    play_sound(&music, macroquad::audio::PlaySoundParams { looped: true, volume: 0.4 });

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

        ball_coords.y -= ball_velocity.y * FALL_SPEED;
        ball_coords.x += ball_velocity.x;

        draw_text(&(-ball_coords.y).round().max(0.).to_string(), 90., camera.target.y, 64., WHITE);

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
        

        if is_mouse_button_pressed(MouseButton::Left) {
            let dx = ball_coords.x - mouse_x;
            let dy = (ball_coords.y-camera.target.y) - mouse_y;

            let len: f32 = (dx*dx + dy*dy).sqrt();

            let ball_vel_x = dx / len * IMPACT_FORCE.x;
            let ball_vel_y = dy / len * IMPACT_FORCE.y;

            ball_velocity = vec2(ball_vel_x, -ball_vel_y);
        } 


        let ball_left = ball_coords.x + HITBOX_ADJUSTMENT;
        let ball_right = ball_coords.x + PLAYER_SIZE - HITBOX_ADJUSTMENT;
        let ball_top = ball_coords.y + HITBOX_ADJUSTMENT;
        let ball_bottom = ball_coords.y + PLAYER_SIZE - HITBOX_ADJUSTMENT;
        let mut rendered_enemies = 0;

        for enemy in &enemies {
            if enemy.position.y > ball_coords.y + screen_h || enemy.position.y < ball_coords.y - screen_h {
                continue;
            }

            let scale_float = enemy.scale as f32;
            let enemy_move_speed: f32 = (3-enemy.scale) as f32/3.;
            let enemy_x = enemy.position.x * screen_w + animation_tick as f32 * enemy_move_speed;

            let left_side = enemy_x + HITBOX_ADJUSTMENT*scale_float;
            let right_side = enemy_x + ENEMY_SIZE*scale_float - HITBOX_ADJUSTMENT*scale_float;
            let top = enemy.position.y + HITBOX_ADJUSTMENT*scale_float;
            let bottom = enemy.position.y + ENEMY_SIZE*scale_float - HITBOX_ADJUSTMENT*scale_float;

            let vertical_check = ball_top < bottom && ball_bottom > top;
            let horizontal_check = ball_right > left_side && ball_left < right_side;

            if vertical_check && horizontal_check {
                clear_background(RED);
            }
            
            draw_texture_ex(
                &enemy_texture,
                enemy_x,
                enemy.position.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(ENEMY_SIZE*scale_float, ENEMY_SIZE*scale_float)), ..Default::default() 
            });
            draw_text(&format!("{}, {}", vertical_check, horizontal_check), 0., ball_coords.y + screen_h/2. - 90., 16., WHITE);

            draw_rectangle(ball_left, ball_top, 4., 4., RED);
            draw_rectangle(ball_right, ball_top, 4., 4., RED);
            draw_rectangle(ball_left, ball_bottom, 4., 4., RED);
            draw_rectangle(ball_right, ball_bottom, 4., 4., RED);

            draw_rectangle(left_side, top, 4., 4., BLUE);
            draw_rectangle(left_side, bottom, 4., 4., BLUE);
            draw_rectangle(right_side, top, 4., 4., BLUE);
            draw_rectangle(right_side, bottom, 4., 4., BLUE);
            rendered_enemies += 1;
        }

        draw_text(&format!("{} fps {} ms", get_fps(), get_frame_time()), 0., ball_coords.y - screen_h/2. + 32., 16., WHITE);
        draw_text(&format!("#{} animation tick, direction: {}", animation_tick, animation_reversing as u8), 0., ball_coords.y - screen_h/2. + 48., 16., WHITE);
        draw_text(&format!("{} enemies, {}x rendered, single enemy footprint: {} bytes, total footprint: {} bytes", enemies.len(), rendered_enemies, size_of_val(&enemies.first().unwrap()), size_of_val(&enemies.first().unwrap()) * enemies.len()), 0., ball_coords.y - screen_h/2. + 64., 16., WHITE);
        draw_text(&format!("{},{} | {}, {}", mouse_x, mouse_y, camera.target.x, camera.target.y), 0., ball_coords.y - screen_h/2. + 80., 16., WHITE);


        if ball_coords.y < highest_y + GENERATION_BUFFER_Y as f32 {
            (enemies, highest_y) = create_enemies(enemies, highest_y, 90).await;
        }

        ball_velocity.y -= 1.;
        ball_velocity.x *= SLOWDOWN_SPEED;
        
        if ball_coords.y > 500. {
            ball_velocity.y = 0.;
            ball_coords.x = 300.;
            ball_coords.y = 0.;
        }

                
        if ball_coords.x < -HITBOX_ADJUSTMENT || ball_coords.x + PLAYER_SIZE + HITBOX_ADJUSTMENT > screen_w {
            play_sound(&hit_wall, macroquad::audio::PlaySoundParams { looped: false, volume: 1.0 });
            ball_velocity.x *= -1.2;
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

        next_frame().await;

    }
}

async fn create_enemies(current: Vec<Enemy>, from_y: f32, amount: u32) -> (Vec<Enemy>, f32) {
    println!("Creating new enemies");
    let mut enemies: Vec<Enemy> = current;

    let mut last_y = from_y;

    for _i in 0..amount {
        let generated_y = rand::gen_range(last_y - ENEMY_SIZE - 50., last_y - ENEMY_SIZE);

        enemies.push({Enemy { 
                position: vec2(
                    rand::gen_range(0.1, 0.9),
                    generated_y,
                ),
                scale: rand::gen_range(1, 3)
            }});

        last_y = generated_y;
    }

    return (enemies, last_y)
}