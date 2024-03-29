use macroquad::prelude::*;
use serde::{Serialize, Deserialize};
use std::f32::consts::{PI, SQRT_2};

use crate::color::Color;
use crate::connection::Connection;
use crate::particle::ParticleList;
use crate::particle::fire::Fire;
use crate::tile::BorderState;
use crate::utils::direction_midpoint;
use crate::sprites::GameSprites;
use crate::sprites::SoundType;

// used for storing a train in a Tracktile
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Train {
    color: Color,
    source: u8,
    destination: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracktile {
    active_connection: Option<Connection>,
    passive_connection: Option<Connection>,
    trains: Vec<Train>,
    #[serde(skip)]
    pub rect: Option<Rect>,
    scale: f32,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConnectionType {
    None,
    S,
    B,
    H,
    Z,
    M,
    J,
}
impl ConnectionType {
    pub fn get_char(&self) -> char {
        match self {
            ConnectionType::None => '_',
            ConnectionType::S => 'S',
            ConnectionType::B => 'B',
            ConnectionType::H => 'H',
            ConnectionType::Z => 'Z',
            ConnectionType::M => 'M',
            ConnectionType::J => 'J',
        }
    }
}

impl Tracktile {
    pub fn new(
        active_connection: Option<Connection>,
        passive_connection: Option<Connection>,
    ) -> Tracktile {
        Tracktile {
            active_connection,
            passive_connection,
            trains: Vec::new(),
            rect: None,
            scale: 1.,
        }
    }

    fn has_any_connection(&self, dir: u8) -> bool {
        if let Some(connection) = self.active_connection {
            if connection.contains(dir) {
                return true;
            }
        }
        if let Some(connection) = self.passive_connection {
            if connection.contains(dir) {
                return true;
            }
        }
        false
    }
    pub fn switch_active_passive(&mut self, gs: &mut GameSprites) {
        // this function is called whenever an odd number of trains rolls through a tracktile
        // or when a user double clicks a tile when drawing
        // if there is no passive connection, then we do nothing
        let c = self.connection_type();
        if c == ConnectionType::M || c == ConnectionType::J {
            let temp = self.passive_connection;
            self.passive_connection = self.active_connection;
            self.active_connection = temp;
            gs.add_sound(SoundType::SwitchTrack);
        }
    }

    fn has_connections(&self, c1: Connection, c2: Connection) -> bool {
        // returns true iff self has both an active and passive connection,
        // and the connections match c1 and c2 (regardless of active/passive)
        if let Some(my_c1) = self.active_connection {
            if let Some(my_c2) = self.passive_connection {
                return (my_c1 == c1 && my_c2 == c2) || (my_c1 == c2 && my_c2 == c1);
            }
        }
        false
    }

    pub fn has_connection_up_to_rot(&self, c: Connection) -> i8 {
        // returns -1 if there is no connection, otherwise returns the rotation amount
        if let Some(my_c) = self.active_connection {
            for rot_amt in 0..4 {
                if c.rot(rot_amt) == my_c {
                    return rot_amt as i8;
                }
            }
        }
        if let Some(my_c) = self.passive_connection {
            for rot_amt in 0..4 {
                if c.rot(rot_amt) == my_c {
                    return rot_amt as i8;
                }
            }
        }
        -1
    }

    pub fn has_active_connection_up_to_rot(&self, c: Connection) -> i8 {
        // returns -1 if there is no connection, otherwise returns the rotation amount
        if let Some(my_c) = self.active_connection {
            for rot_amt in 0..4 {
                if c.rot(rot_amt) == my_c {
                    return rot_amt as i8;
                }
            }
        }
        -1
    }

    pub fn has_connections_up_to_rot(&self, c1: Connection, c2: Connection) -> i8 {
        // returns true iff self has both an active and passive connection,
        // and the connections match c1 and c2 (regardless of active/passive)
        // after being rotated a fixed amount
        if let Some(my_c1) = self.active_connection {
            if let Some(my_c2) = self.passive_connection {
                for rot_amt in 0..4 {
                    let rot_c1 = c1.rot(rot_amt);
                    let rot_c2 = c2.rot(rot_amt);
                    if (my_c1 == rot_c1 && my_c2 == rot_c2) || (my_c1 == rot_c2 && my_c2 == rot_c1)
                    {
                        return rot_amt as i8;
                    }
                }
            }
        }
        -1
    }
    pub fn has_active_passive_connections_up_to_rot(
        &self,
        active: Connection,
        passive: Connection,
    ) -> i8 {
        if let Some(my_active) = self.active_connection {
            if let Some(my_passive) = self.passive_connection {
                for rot_amt in 0..4 {
                    let rot_active = active.rot(rot_amt);
                    let rot_passive = passive.rot(rot_amt);
                    if my_active == rot_active && my_passive == rot_passive {
                        return rot_amt as i8;
                    }
                }
            }
        }
        -1
    }

    fn indices_of_trains_along(&self, c: Connection, i1: &mut usize, i2: &mut usize) -> bool {
        // returns true iff there is a train colliding along the connection c.
        for index1 in 0..self.trains.len() {
            for index2 in 0..self.trains.len() {
                if self.trains[index1].source == c.dir1
                    && self.trains[index1].destination == c.dir2
                    && self.trains[index2].source == c.dir2
                    && self.trains[index2].destination == c.dir1
                {
                    *i1 = index1;
                    *i2 = index2;
                    return true;
                }
            }
        }
        false
    }

    pub fn connection_type(&self) -> ConnectionType {
        if self.active_connection.is_none() {
            return ConnectionType::None;
        }
        if self.passive_connection.is_none() {
            if self.has_connection_up_to_rot(Connection { dir1: 0, dir2: 2 }) != -1 {
                return ConnectionType::S;
            }
            return ConnectionType::B;
        }
        // now we can assume that there is both an active and passive connection
        if self.has_connections(
            Connection { dir1: 0, dir2: 2 },
            Connection { dir1: 1, dir2: 3 },
        ) {
            return ConnectionType::H;
        }
        if self.has_connections_up_to_rot(
            Connection { dir1: 0, dir2: 1 },
            Connection { dir1: 2, dir2: 3 },
        ) != -1
        {
            return ConnectionType::Z;
        }

        if self.has_connections_up_to_rot(
            Connection { dir1: 0, dir2: 1 },
            Connection { dir1: 1, dir2: 2 },
        ) != -1
        {
            return ConnectionType::M;
        }

        if self.has_connections_up_to_rot(
            Connection { dir1: 0, dir2: 1 },
            Connection { dir1: 0, dir2: 2 },
        ) != -1
            || self.has_connections_up_to_rot(
                Connection { dir1: 0, dir2: 3 },
                Connection { dir1: 0, dir2: 2 },
            ) != -1
        {
            return ConnectionType::J;
        }

        unreachable!()
    }

    pub fn process_tick(&mut self, gs: &mut GameSprites, p: &mut ParticleList) {
        // This function mixes any train colors (happens when trains are halfway through the tile)
        let scale = self.scale;
        let my_type = self.connection_type();
        if self.trains.len() >= 2 {
            if my_type == ConnectionType::H
                || my_type == ConnectionType::S
                || my_type == ConnectionType::B
            {
                // simply mix all the trains in these connection types
                let new_color =
                    Color::mix_many(self.trains.iter().map(|train| train.color).collect());
                for i in 0..self.trains.len() {
                    self.trains[i].color = new_color;
                }
                gs.play_train_sound(new_color);
                let (x, y) = get_midpoint_of_conn(self.active_connection.unwrap(), self.rect.unwrap());
                p.push(Box::new(Fire::new(
                    x, y, new_color, scale
                )));
                return;
            }

            if my_type == ConnectionType::Z {
                let mut i1: usize = 0;
                let mut i2: usize = 0;
                // first do mixing on Active Connection
                if self.indices_of_trains_along(self.active_connection.unwrap(), &mut i1, &mut i2) {
                    let new_color =
                        Color::mix_many(vec![self.trains[i1].color, self.trains[i2].color]);
                    self.trains[i1].color = new_color;
                    self.trains[i2].color = new_color;
                    gs.play_train_sound(new_color);
                    let (x, y) = get_midpoint_of_conn(self.active_connection.unwrap(), self.rect.unwrap());
                    p.push(Box::new(Fire::new(
                        x, y, new_color, scale
                    )));

                }
                // then do mixing on Passive Connection
                if self.indices_of_trains_along(self.passive_connection.unwrap(), &mut i1, &mut i2)
                {
                    let new_color =
                        Color::mix_many(vec![self.trains[i1].color, self.trains[i2].color]);
                    self.trains[i1].color = new_color;
                    self.trains[i2].color = new_color;
                    gs.play_train_sound(new_color);
                    let (x, y) = get_midpoint_of_conn(self.passive_connection.unwrap(), self.rect.unwrap());
                    p.push(Box::new(Fire::new(
                        x, y, new_color, scale
                    )));
                }
                return;
            }

            if my_type == ConnectionType::J || my_type == ConnectionType::M {
                let mut i1: usize = 0;
                let mut i2: usize = 0;
                if self.indices_of_trains_along(self.active_connection.unwrap(), &mut i1, &mut i2) {
                    let new_color = Color::mix_many(vec![self.trains[i1].color, self.trains[i2].color]);
                    self.trains[i1].color = new_color;
                    self.trains[i2].color = new_color;
                    gs.play_train_sound(new_color);
                    let (x, y) = get_midpoint_of_conn(self.active_connection.unwrap(), self.rect.unwrap());
                    p.push(Box::new(Fire::new(
                        x, y, new_color, scale
                    )));
                }
            }
        }
    }

    pub fn interact_trains(&mut self, gs:&mut GameSprites, p: &mut ParticleList) {
        // This function merges trains (happens at the moment trains are exiting the tile)
        let my_type = self.connection_type();

        let need_to_switch_active_passive = self.trains.len() % 2 == 1;

        if self.trains.len() >= 2 {
            // we don't need to worry about these connection types because we have already handled mixing in the process_tick function
            if my_type == ConnectionType::H
                || my_type == ConnectionType::S
                || my_type == ConnectionType::B
                || my_type == ConnectionType::Z
            {
                return;
            }
            // At this point, we know we either have a J or M type connection.

            // We simply merge trains (if there are any going to the same destination)

            'outer: for i1 in 0..self.trains.len() {
                for i2 in 0..self.trains.len() {
                    if i1 == i2 {
                        continue;
                    }

                    if self.trains[i1].destination == self.trains[i2].destination {
                        let new_color =
                            Color::mix_many(vec![self.trains[i1].color, self.trains[i2].color]);
                        self.trains[i1].color = new_color;
                        self.trains.remove(i2);
                        gs.play_train_sound(new_color);

                        let dir = self.trains[i1].destination;
                        let (x, y) = direction_midpoint(self.rect.unwrap(), dir);
                        p.push(Box::new(Fire::new(
                            x, y, new_color, self.scale
                        )));

                        break 'outer;
                    }
                }
            }
        }

        if need_to_switch_active_passive {
            self.switch_active_passive(gs);
        }
    }

    pub fn accept_trains(&mut self, colors: BorderState) -> BorderState {
        // return None if no trains on edge, and return Some(_) on edge if train crashed.
        
        let mut crashed = false;
        let mut border_state = [None, None, None, None];
        for (dir, train) in colors.iter().enumerate() {
            if let Some(color) = *train {
                if !self.has_any_connection(dir as u8) {
                    crashed = true;
                    border_state[dir] = Some(color);
                }
            }
        }
        if crashed {
            return border_state;
        }
        for (dir, train) in colors.iter().enumerate() {
            let dir = dir as u8;

            if let Some(color) = *train {
                //now we have to determine the train's destination based on it's source direction (dir).
                if let Some(active_conn) = self.active_connection {
                    if active_conn.contains(dir) {
                        let other_dir = if active_conn.dir1 == dir {
                            active_conn.dir2
                        } else {
                            active_conn.dir1
                        };
                        self.trains.push(Train {
                            color,
                            source: dir,
                            destination: other_dir,
                        });
                        continue;
                    }
                }
                if let Some(passive_conn) = self.passive_connection {
                    if passive_conn.contains(dir) {
                        let other_dir = if passive_conn.dir1 == dir {
                            passive_conn.dir2
                        } else {
                            passive_conn.dir1
                        };
                        self.trains.push(Train {
                            color,
                            source: dir,
                            destination: other_dir,
                        });
                        continue;
                    }
                }
                unreachable!()
            }
        }

        [None, None, None, None]
    }

    pub fn dispatch_trains(&mut self) -> BorderState {
        // we panic if two trains have the same destination, since we should have dealt with that already,
        let mut res = [None, None, None, None];
        while let Some(train) = self.trains.pop() {
            let dest = train.destination;
            if res[dest as usize].is_some() {
                panic!("Two trains had the same direction when exiting a tracktile! This should have been handled by the function interact_trains()");
            }
            res[dest as usize] = Some(train.color);
        }
        res
    }

    pub fn add_connection(&mut self, conn: Connection, gs: &mut GameSprites) {
        self.passive_connection = self.active_connection;
        self.active_connection = Some(conn);
        if self.active_connection == self.passive_connection {
            self.passive_connection = None;
        }
        gs.add_sound(SoundType::DrawTrack);
    }

    pub fn clear_trains(&mut self) {
        self.trains = vec![];
    }

    pub fn clear_connections(&mut self) {
        self.active_connection = None;
        self.passive_connection = None;
    }

    pub fn set_rect(&mut self, rect: Rect, gs: &GameSprites) {
        self.rect = Some(rect);
        self.scale = rect.w / gs.tracktile_blank.width();
    }


    pub fn render_trains(&self, gs: &GameSprites, progress: f32) {
        let train_width = gs.train.width() * self.scale;
        let train_height = gs.train.height() * self.scale;
        let rect = self.rect.unwrap();

        for train in &self.trains {
            let mut train_center_x = rect.x;
            let mut train_center_y = rect.y;
            let mut rot = 0.0;
            if train.source == 0 && train.destination == 2 {
                train_center_x = rect.x + (rect.w/2.);
                train_center_y = rect.y + (rect.h * progress/2.);
                rot = PI;
            } else if train.source == 2 && train.destination == 0 {
                train_center_x = rect.x + (rect.w/2.);
                train_center_y = rect.y + (rect.h * (1.0 - progress/2.0));
                rot = 0.0;
            } else if train.source == 3 && train.destination == 1 {
                train_center_x = rect.x + (rect.w * progress/2.0);
                train_center_y = rect.y + (rect.h/2.0);
                rot = PI/2.;
            } else if train.source == 1 && train.destination == 3 {
                train_center_x = rect.x + (rect.w * (1.0 - progress/2.0));
                train_center_y = rect.y + (rect.h/2.);
                rot = 3.*PI/2.;
            } else if train.source == 0 && train.destination == 1 {
                train_center_x = rect.x + rect.w + 
                    ((rect.w/2.) * -f32::cos(progress/4.0*PI));
                train_center_y = rect.y +
                    ((rect.h/2.) * f32::sin(progress/4.0*PI));
                rot = PI - PI/2. * progress/2.0;
            } else if train.source == 1 && train.destination == 2 {
                train_center_x = rect.x + rect.w + 
                    ((rect.w/2.) * -f32::sin(progress/4.0*PI));
                train_center_y = rect.y + rect.h + 
                    ((rect.h/2.) * -f32::cos(progress/4.0*PI));
                rot = 3.*PI/2. - PI/2. * progress/2.0;
            } else if train.source == 2 && train.destination == 3 {
                train_center_x = rect.x +
                    ((rect.w/2.) * f32::cos(progress/4.0*PI));
                train_center_y = rect.y + rect.h + 
                    ((rect.h/2.) * -f32::sin(progress/4.0*PI));
                rot = 2.*PI - PI/2. * progress/2.0;
            } else if train.source == 3 && train.destination == 0 {
                train_center_x = rect.x +
                    ((rect.w/2.) * f32::sin(progress/4.0*PI));
                train_center_y = rect.y +
                    ((rect.h/2.) * f32::cos(progress/4.0*PI));
                rot = PI/2. - PI/2. *progress/2.0;
            } else if train.source == 0 && train.destination == 3 {
                train_center_x = rect.x +
                    ((rect.w/2.) * f32::cos(progress/4.0*PI));
                train_center_y = rect.y +
                    ((rect.h/2.) * f32::sin(progress/4.0*PI));
                rot = PI + PI/2. *progress/2.0;
            } else if train.source == 3 && train.destination == 2 {
                train_center_x = rect.x +
                    ((rect.w/2.) * f32::sin(progress/4.0*PI));
                train_center_y = rect.y + rect.h + 
                    ((rect.h/2.) * -f32::cos(progress/4.0*PI));
                rot = PI/2.  + PI/2. *progress/2.0;
            }  else if train.source == 2 && train.destination == 1 {
                train_center_x = rect.x + rect.w + 
                    ((rect.w/2.) * -f32::cos(progress/4.0*PI));
                train_center_y = rect.y + rect.h + 
                    ((rect.h/2.) * -f32::sin(progress/4.0*PI));
                rot = PI/2. *progress/2.0;
            } else if train.source == 1 && train.destination == 0 {
                train_center_x = rect.x + rect.w + 
                    ((rect.w/2.) * -f32::sin(progress/4.0*PI));
                train_center_y = rect.y + 
                    ((rect.h/2.) * f32::cos(progress/4.0*PI));
                rot = 3. * PI/2. + PI/2. *progress/2.0;
            }

            draw_texture_ex(
                gs.train,
                train_center_x - (train_width/2.),
                train_center_y - (train_height/2.),
                train.color.get_color(),
                DrawTextureParams {
                    dest_size: Some(Vec2::new(train_width, train_height)),
                    source: None,
                    rotation: rot,
                    flip_x: false,
                    flip_y: false,
                    pivot: None
                }
             );
        }
    }
}


pub fn get_midpoint_of_conn(conn: Connection, rect: Rect) -> (f32, f32) {
    let (x, y, w, h) = (rect.x, rect.y, rect.w, rect.h);
    let small_w = SQRT_2/4.0 * w;
    let small_h = SQRT_2/4.0 * h;
    let big_w = w - small_w;
    let big_h = h - small_h;


    if conn == (Connection {dir1: 0, dir2: 2}) || conn == (Connection {dir1: 1, dir2: 3}) {
        return (x + w/2., y + h/2.);
    }
    if conn == (Connection {dir1: 3, dir2: 0}) {
        return (x + small_w, y + small_h);
    }
    if conn == (Connection {dir1: 0, dir2: 1}) {
        return (x + big_w, y + small_h);
    }
    if conn == (Connection {dir1: 1, dir2: 2}) {
        return (x + big_w, y + big_h);
    }
    if conn == (Connection {dir1: 2, dir2: 3}) {
        return (x + small_w, y + big_h);
    }

    unreachable!()
}