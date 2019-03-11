use core::f64::consts::PI;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fs;

#[derive(Debug)]
struct FiringPlan {
    velocity: MetresPerSecond,
    angle: Radians,
}

impl FiringPlan {
    fn new(velocity: MetresPerSecond, angle: Radians) -> FiringPlan {
        // For cannon-ball firing the following two constraints are needed:
        // 1. So we actually move...
        assert!(velocity > MetresPerSecond(0.0));
        // 2. So we fire up, not into the ground:
        assert!(angle > Radians(0.0));
        assert!(angle < Radians(PI));

        FiringPlan { velocity, angle }
    }
    fn random<R: Rng>(rng: &mut R) -> FiringPlan {
        FiringPlan::new(
            MetresPerSecond(1.0 + rng.gen::<f64>()),
            Radians(rng.gen::<f64>() * PI), // NB: Pi radians is 1/2 a circle
        )
    }
    fn randoms<R: Rng>(rng: &mut R, n: usize) -> Vec<FiringPlan> {
        let mut vec = Vec::with_capacity(n);
        (0..n).for_each(|_| vec.push(FiringPlan::random(rng)));
        vec
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
struct MetresPerSecond(f64);

#[derive(Debug, PartialEq, PartialOrd)]
struct Radians(f64);

impl Radians {
    fn sin(&self) -> f64 {
        self.0.sin()
    }
    fn cos(&self) -> f64 {
        self.0.cos()
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
struct Fitness(f64); // Bigger is better

#[derive(Debug, PartialEq, PartialOrd)]
struct Metres(f64);

#[derive(Debug)]
struct Coordinates {
    x: Metres,
    y: Metres,
}

#[derive(Debug)]
struct Trajectory(Vec<Coordinates>);

impl Trajectory {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn save(&self, filename: &str) {
        let data: Vec<String> = self
            .0
            .iter()
            .map(|c| format!("{} {}", c.x.0, c.y.0))
            .collect();
        fs::write(filename, data.join("\n")).expect("Save failed");
    }
}

// We fire from (0, 0) and the wall is at (+/-distance, 0) up to (+/-distance, height)
fn simulate(p: &FiringPlan, height: &Metres, distance: &Metres) -> Trajectory {
    const G: f64 = 9.81; // gravity on Earth
    const STEP_SIZE: f64 = 0.1; // seconds
    let cos_theta = p.angle.cos();
    let sin_theta = p.angle.sin();

    // Calculate co-ordinates of cannon-ball at time t:
    let position = |t: &f64| {
        let vt = p.velocity.0 * t;
        let x = Metres(vt * cos_theta);
        let y = Metres(vt * sin_theta - (0.5 * G * t * t));
        Coordinates { x, y }
    };

    // What's the cannon-ball height at the point of the wall?
    // i.e., did we clear the wall?
    let t_at_wall = distance.0 / (p.velocity.0 * cos_theta);
    let coords_at_wall = position(&t_at_wall);
    let did_hit_wall = coords_at_wall.y >= Metres(0.0) && coords_at_wall.y < *height;

    // Build up cannon-ball trajectory:
    let mut path = Vec::new();
    let mut t = 0.0;
    let mut y = Metres(0.0);
    while t == 0.0 || (did_hit_wall && t < t_at_wall) || y >= Metres(0.0) {
        t = t + STEP_SIZE;
        let coords = position(&t);
        y = Metres(coords.y.0);
        path.push(coords);
    }
    Trajectory(path)
}

// We maximize how far the cannon bal has travelled horizontally.
fn evaluate(p: &FiringPlan, height: &Metres, distance: &Metres) -> Fitness {
    let traj = simulate(&p, &height, &distance);
    Fitness(traj.len() as f64)
}

fn main() {
    let wall_height = Metres(10.0);
    let wall_distance = Metres(30.0);

    let seed: u64 = 1;
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let pop_size = 4;

    let ps = FiringPlan::randoms(&mut rng, pop_size);

    for (i, p) in ps.iter().enumerate() {
        let fitness = evaluate(&p, &wall_height, &wall_distance);
        println!("\nIndividual {}: {:?},\n{:?}", i, p, fitness);
    }
    /*
        let p = FiringPlan {
            velocity: MetresPerSecond(10.0),
            angle: Radians(0.8),
        };

        let traj = simulate(&p, &wall_height, &wall_distance);
        println!("\nSimulated: {:?}", traj);

        // Write for gnuplot
        traj.save("traj.dat");
    */
}
