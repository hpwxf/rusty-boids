use std::f32::consts::PI;

use cgmath::{Point2, Vector2, InnerSpace};
use cgmath::{Basis2, Rad, Rotation, Rotation2};
use rand::distributions::{IndependentSample, Range};
use rand::ThreadRng;
use rand;

//TODO: Have some sort of control for these
//Could have a config file, with a flag to reload on change
const MAX_SPEED: f32 = 2.0;
const MAX_FORCE: f32 = 0.1;
const SEP_WEIGHT: f32 = 1.5;
const ALI_WEIGHT: f32 = 1.0;
const COH_WEIGHT: f32 = 1.0;
const SEP_RADIUS: f32 = 25.0;
const ALI_RADIUS: f32 = 50.0;
const COH_RADIUS: f32 = 50.0;

// Maintain squared versions to speed up calculation
const SEP_RADIUS_2: f32 = SEP_RADIUS * SEP_RADIUS;
const ALI_RADIUS_2: f32 = ALI_RADIUS * ALI_RADIUS;
const COH_RADIUS_2: f32 = COH_RADIUS * COH_RADIUS;

const TWO_PI: f32 = 2. * PI;

type Position = Point2<f32>;
type Velocity = Vector2<f32>;
type Force = Vector2<f32>;

#[repr(C)]
struct Boid {
    position: Position,
    velocity: Velocity,
}

impl Boid {
    fn apply_force(&mut self, force: Force) {
        self.velocity += force;
        self.velocity = limit(self.velocity, MAX_SPEED);
        self.position += self.velocity;
    }

    fn wrap_to(&mut self, width: f32, height: f32) {
        if self.position.x < 0. { self.position.x = width };
        if self.position.y < 0. { self.position.y = height };
        if self.position.x > width { self.position.x = 0. };
        if self.position.y > height { self.position.y = 0. };
    }
}

pub struct FlockingSystem {
    boids: Vec<Boid>,
    width: f32,
    height: f32,
    rng: ThreadRng,
}

impl FlockingSystem {
    pub fn new(size: (f32, f32)) -> FlockingSystem {
        FlockingSystem {
            boids: vec![],
            width: size.0,
            height: size.1,
            rng: rand::thread_rng(),
        }
    }

    pub fn add_boids(&mut self, count: usize) {
        for _ in 0..count {
            let pos = self.random_position();
            let vel = self.random_velocity();
            self.boids.push(Boid{
                position: pos,
                velocity: vel,
            });
        }

    }

    pub fn resize(&mut self, size: (f32, f32)) {
        self.width = size.0;
        self.height = size.1;
    }


    pub fn randomise(&mut self) {
        for i in 0..self.boids.len() {
           self.boids[i].position = self.random_position();
           self.boids[i].velocity = self.random_velocity();
        }
    }

    pub fn centralise(&mut self) {
        let center = Point2::new(self.width/2., self.height/2.);
        for i in 0..self.boids.len() {
           self.boids[i].position = center;
           self.boids[i].velocity = self.random_velocity();
        }
    }

    pub fn zeroise(&mut self) {
        for i in 0..self.boids.len() {
           self.boids[i].position = Point2::new(0., 0.);
           self.boids[i].velocity = self.random_velocity();
        }
    }

    //TODO: Introduce dt to smooth the simulation
    pub fn update(&mut self) {
        for i in 0..self.boids.len() {
            let force = self.react_to_neighbours(i);
            self.apply_force(i, force);
        }
    }

    fn apply_force(&mut self, id: usize, force: Force) {
        let boid = &mut self.boids[id];
        boid.apply_force(force);
        boid.wrap_to(self.width, self.height);
    }

    //TODO: At some point, use spacial data structure
    //TODO: Break this up a bit
    fn react_to_neighbours(&self, i: usize) -> Force {
        let boid = &self.boids[i];
        let mut dodge = Vector2::new(0., 0.);
        let mut ali_vel_acc = Vector2::new(0., 0.);
        let mut ali_vel_count = 0;
        let mut coh_pos_acc = Vector2::new(0., 0.);
        let mut coh_pos_count = 0;
        for j in 0..self.boids.len() {
            if i != j {
                let other = &self.boids[j];
                let from_neighbour = boid.position - other.position;
                let dist_squared = from_neighbour.magnitude2();
                if dist_squared > 0. {
                    if dist_squared < SEP_RADIUS_2 {
                        let repulse = 1./dist_squared.sqrt();
                        dodge += from_neighbour.normalize_to(repulse);
                    }
                    if dist_squared < ALI_RADIUS_2 {
                        ali_vel_acc += other.velocity;
                        ali_vel_count += 1;
                    }
                    if dist_squared < COH_RADIUS_2 {
                        coh_pos_acc.x += other.position.x;
                        coh_pos_acc.y += other.position.y;
                        coh_pos_count += 1;
                    }
                }
            }
        }
        let mut force = Vector2::new(0., 0.);
        if dodge.magnitude2() > 0. {
            let d_steer = steer(boid, dodge.normalize_to(MAX_SPEED));
            force += SEP_WEIGHT * d_steer;
        }
        if ali_vel_count > 0 {
            let align = ali_vel_acc / ali_vel_count as f32;
            let a_steer = steer(boid, align.normalize_to(MAX_SPEED));
            force += ALI_WEIGHT * a_steer;
        }
        if coh_pos_count > 0 {
            let avg_pos = coh_pos_acc / coh_pos_count as f32;
            let boid_pos = Vector2::new(boid.position.x, boid.position.y);
            let cohesion = avg_pos - boid_pos;
            let c_steer = steer(boid, cohesion.normalize_to(MAX_SPEED));
            force += COH_WEIGHT * c_steer;
        }
        force
    }

    //TODO: Instead do this with zero copy somehow?
    // Maybe just make renderer accept boids...
    // use two vertex atribs for vel and pos
    // do something pretty with vel...?
    pub fn positions(&self) -> Vec<Position> {
        self.boids.iter()
            .map(|b| b.position)
            .collect()
    }

    fn random_position(&mut self) -> Position {
        let sim_space_x = Range::new(0., self.width);
        let sim_space_y = Range::new(0., self.height);
        let x = sim_space_x.ind_sample(&mut self.rng);
        let y = sim_space_y.ind_sample(&mut self.rng);
        Point2::new(x, y)
    }

    fn random_velocity(&mut self) -> Velocity {
        let vel_space = Range::new(0., MAX_SPEED);
        let ang_space = Range::new(0., TWO_PI);
        let s = vel_space.ind_sample(&mut self.rng);
        let a = ang_space.ind_sample(&mut self.rng);
        Basis2::from_angle(Rad(a))
            .rotate_vector(Vector2::new(0., s))
    }

}


fn steer(boid: &Boid, target_vel: Velocity) -> Force {
    let force = target_vel - boid.velocity;
    limit(force, MAX_FORCE)
}

fn limit(force: Force, max: f32) -> Force {
    if force.magnitude2() > max*max {
        force.normalize_to(max)
    } else {
        force
    }
}

