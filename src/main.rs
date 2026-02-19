use macroquad::{prelude::{rand, *}};

struct Enemy {
    position: Vec2
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Bouncy Ball".to_owned(),
        window_resizable: false,
        ..Default::default()
    }
}

const ENEMY_SIZE: f32 = 64.;
const PLAYER_SIZE: f32 = 64.;
const HITBOX_ADJUSTMENT: f32 = 8.;
const GENERATION_BUFFER_Y: i32 = 300;
const FALL_SPEED: f32 = 0.3;
const IMPACT_FORCE: Vec2 = vec2(19., 36.);
const SLOWDOWN_SPEED: f32 = 0.9;


#[macroquad::main(window_conf)]
async fn main() {
    let ball_texture = Texture2D::from_image(&Image::from_file_with_format(include_bytes!("assets/ball.png"), None).unwrap());
    ball_texture.set_filter(FilterMode::Nearest);
    let enemy_texture = Texture2D::from_image(&Image::from_file_with_format(include_bytes!("assets/enemy.png"), None).unwrap());
    enemy_texture.set_filter(FilterMode::Nearest);


    let mut ball_coords = vec2(300., 0.);
    let mut ball_velocity = vec2(0., 0.);

    let (mut enemies, mut highest_y) = create_enemies(vec![], 300., 30).await;

    let mut mouse_locked = false;

    let mut camera = Camera2D {
        zoom: vec2(2. / screen_width(), 2. / screen_height()),
        ..Default::default()
    };


    loop {
        set_camera(&camera);
        clear_background(BLACK);

        let (mouse_x, mouse_y) = mouse_position();
        let relative_mouse = vec2(mouse_x, camera.target.y + (mouse_y - screen_height() / 2.));

        ball_coords.y -= ball_velocity.y * FALL_SPEED;
        ball_coords.x += ball_velocity.x;

        draw_text(&(-ball_coords.y).round().max(0.).to_string(), 90., camera.target.y, 64., WHITE);

        draw_texture_ex(&ball_texture, ball_coords.x, ball_coords.y, WHITE, DrawTextureParams { dest_size: Some(vec2(PLAYER_SIZE, PLAYER_SIZE)), ..Default::default() });

        let hit_strength = (relative_mouse.distance(ball_coords)/screen_width()).max(0.);
        
        draw_line(
            ball_coords.x + PLAYER_SIZE / 2.0,
            ball_coords.y + PLAYER_SIZE / 2.0,
            relative_mouse.x, 
            relative_mouse.y, 
            hit_strength*4., 
            Color { r: 0., g: 0.48, b: 1., a: hit_strength}
        );
        
        if !is_mouse_button_down(MouseButton::Left) && mouse_locked {
            mouse_locked = false;
        }

        if is_mouse_button_down(MouseButton::Left) && !mouse_locked {
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

        for enemy in &enemies {
            if enemy.position.y > ball_coords.y + screen_height() || enemy.position.y < ball_coords.y - screen_height() {
                continue;
            }

            let left_side = (enemy.position.x * screen_width()) + HITBOX_ADJUSTMENT;
            let right_side = (enemy.position.x * screen_width()) + ENEMY_SIZE - HITBOX_ADJUSTMENT;
            let top = enemy.position.y + HITBOX_ADJUSTMENT;
            let bottom = enemy.position.y + ENEMY_SIZE - HITBOX_ADJUSTMENT;

            let vertical_check = ball_top < bottom && ball_bottom > top;
            let horizontal_check = ball_right > left_side && ball_left < right_side;

            if vertical_check && horizontal_check {
                clear_background(RED);
            }
            
            draw_texture_ex(
                &enemy_texture,
                enemy.position.x * screen_width(),
                enemy.position.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(ENEMY_SIZE, ENEMY_SIZE)), ..Default::default() 
            });
            draw_text(&format!("{}, {}", vertical_check, horizontal_check), 0., ball_coords.y + screen_height()/2. - 90., 16., WHITE);

            draw_rectangle(ball_left, ball_top, 4., 4., RED);
            draw_rectangle(ball_right, ball_top, 4., 4., RED);
            draw_rectangle(ball_left, ball_bottom, 4., 4., RED);
            draw_rectangle(ball_right, ball_bottom, 4., 4., RED);

            draw_rectangle(left_side, top, 4., 4., BLUE);
            draw_rectangle(left_side, bottom, 4., 4., BLUE);
            draw_rectangle(right_side, top, 4., 4., BLUE);
            draw_rectangle(right_side, bottom, 4., 4., BLUE);
        }

        draw_text(&format!("{} fps", get_fps()), 0., ball_coords.y - screen_height()/2. + 16., 16., WHITE);

        if ball_coords.y < highest_y + GENERATION_BUFFER_Y as f32 {
            (enemies, highest_y) = create_enemies(enemies, highest_y, 30).await;
        }

        ball_velocity.y -= 1.;
        ball_velocity.x *= SLOWDOWN_SPEED;
        
        if ball_coords.y > 500. {
            ball_velocity.y = 0.;
            ball_coords.x = 300.;
            ball_coords.y = 0.;
        }

                
        if ball_coords.x < -HITBOX_ADJUSTMENT || ball_coords.x + PLAYER_SIZE + HITBOX_ADJUSTMENT > screen_width() {
            ball_velocity.x *= -1.2;
        }

        camera.target = vec2(screen_width()/2., ball_coords.y);
        

        next_frame().await;

    }
}

async fn create_enemies(current: Vec<Enemy>, from_y: f32, amount: u32) -> (Vec<Enemy>, f32) {
    println!("Creating new enemies");
    let mut enemies: Vec<Enemy> = current;

    let mut last_y = from_y;

    for _i in 0..amount {
        let generated_y = rand::gen_range(last_y - ENEMY_SIZE - 200., last_y - ENEMY_SIZE);

        enemies.push({Enemy { 
                position: vec2(
                    rand::gen_range(0., 1.),
                    generated_y
                )
            }});

        last_y = generated_y;
    }

    return (enemies, last_y)
}