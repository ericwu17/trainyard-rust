use crate::color::Color;

use crate::tile::BorderState;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use crate::sprites::GameSprites;
use crate::particle::ParticleList;
use crate::particle::shrinking_plus::ShrinkingPlus;


#[derive(Debug, Clone)]
pub struct Trainsource {
    pub trains: Vec<Option<Color>>,
    pub dir: u8,
    pub outgoing_train: Option<Color>,
    pub icon_rects: Vec<Rect>,
    pub rect: Option<Rect>,
}

impl Trainsource {
    pub fn new(trains: Vec<Color>, dir: u8) -> Trainsource {
        Trainsource {
            trains: trains.into_iter().map(Some).collect(),
            dir,
            outgoing_train: None,
            rect: None,
            icon_rects: vec![],
        }
    }

    pub fn accept_trains(&self, trains: BorderState) -> BorderState {
        return trains;
    }

    pub fn process_tick(&mut self, p: &mut ParticleList) {
        for (index, train) in self.trains.iter().enumerate() {
            if let Some(color) = *train {
                self.outgoing_train = Some(color);
                self.trains[index] = None;
                p.push(Box::new(ShrinkingPlus::new(
                    self.icon_rects[index], 
                    color,
                )));
                return;
            }
        }
    }

    pub fn dispatch_trains(&mut self) -> BorderState {
        let mut border_state = [None, None, None, None];
        border_state[self.dir as usize] = self.outgoing_train;
        self.outgoing_train = None;
        border_state
    }

    pub fn is_empty(&self) -> bool {
        for train in &self.trains {
            if train.is_some() {
                return false;
            }
        }
        return true;
    }

    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = Some(rect);

        let plus_sign_width = (rect.width() as f64 * (52.0 / 96.0)) as i32;
        let plus_sign_height = (rect.height() as f64 * (52.0 / 96.0)) as i32;
        let num_cols;
        if self.trains.len() <= 1 {
            num_cols = 1;
        } else if self.trains.len() <= 4 {
            num_cols = 2;
        } else if self.trains.len() <= 9 {
            num_cols = 3;
        } else {
            num_cols = 4;
        }
        for i in 0..self.trains.len() {
            let curr_col = i % num_cols;
            let curr_row = i / num_cols;
            let scaled_plus_sign_width = plus_sign_width / num_cols as i32;
            let scaled_plus_sign_height = plus_sign_height / num_cols as i32;
            let x_pos = rect.x()
                + (rect.width() as i32 - plus_sign_width) / 2
                + curr_col as i32 * scaled_plus_sign_width;
            let y_pos = rect.y()
                + (rect.width() as i32 - plus_sign_height) / 2
                + curr_row as i32 * scaled_plus_sign_height;
            self.icon_rects.push(
                Rect::new(
                    x_pos,
                    y_pos,
                    scaled_plus_sign_width as u32,
                    scaled_plus_sign_height as u32,
                )
            );
            
        }
    }


    pub fn render_trains(&self, canvas: &mut WindowCanvas, rect: &Rect, gs: &mut GameSprites, progress: f64) -> Result<(), String> {
        if let Some(color) = self.outgoing_train {
            let train_width = (rect.width() as f64 * (32.0 / 96.0)) as u32;
            let train_height = (rect.height() as f64 * (57.0 / 96.0)) as u32;
            gs.set_color(color);
            let train_center_x;
            let train_center_y;
            let rot;

            if self.dir == 2 {
                train_center_x = rect.x() + (rect.width()/2) as i32;
                train_center_y = rect.y() + (rect.height() as f64 * progress/2.0) as i32;
                rot = 180.0;
            } else if self.dir == 3 {
                train_center_x = rect.x() + (rect.width() as f64 * (1.0 - progress/2.0)) as i32;
                train_center_y = rect.y() + (rect.height()/2) as i32;
                rot = 270.0;
            } else if self.dir == 0 {
                train_center_x = rect.x() + (rect.width()/2) as i32;
                train_center_y = rect.y() + (rect.height() as f64 * (1.0 - progress/2.0)) as i32;
                rot = 0.0;
            } else {
                train_center_x = rect.x() + (rect.width() as f64 * progress/2.0) as i32;
                train_center_y = rect.y() + (rect.height()/2) as i32;
                rot = 90.0;
            }

            let train_rect = Rect::new(train_center_x - (train_width/2) as i32, train_center_y - (train_height/2) as i32, train_width, train_height);
            canvas.copy_ex(&gs.atlas_color, gs.train, train_rect, rot, None, false, false)?;

        }

        Ok(())
    }
}
