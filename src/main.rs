use core::f64::consts::PI;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fs;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct MetresPerSecond(f64);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Radians(f64);

impl Radians {
    fn sin(&self) -> f64 {
        self.0.sin()
    }
    fn cos(&self) -> f64 {
        self.0.cos()
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Fitness(f64); // Bigger is better

#[derive(Debug, PartialEq, PartialOrd)]
struct Seconds(f64);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Metres(f64);

impl Metres {
    fn is_positive(&self) -> bool {
        self.0 > 0.0
    }
}

#[derive(Debug)]
struct Coordinates {
    x: Metres,
    y: Metres,
}

#[derive(Debug)]
struct Trajectory(Vec<Coordinates>);

impl Trajectory {
    fn distance(&self) -> Metres {
        self.0
            .last()
            .map(|coord| coord.x.clone())
            .unwrap_or(Metres(0.0))
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
fn simulate(p: &FiringPlan, params: &Params) -> Trajectory {
    const G: f64 = 9.81; // gravity on Earth
    let cos_theta = p.angle.cos();
    let sin_theta = p.angle.sin();

    // Calculate co-ordinates of cannon-ball at time t:
    let position = |t: &Seconds| {
        let vt = p.velocity.0 * t.0;
        let x = Metres(vt * cos_theta);
        let y = Metres(vt * sin_theta - (0.5 * G * t.0 * t.0));
        Coordinates { x, y }
    };

    // What's the cannon-ball height at the point of the wall?
    // i.e., did we clear the wall?
    let t_at_wall = Seconds(params.wall_distance.0 / (p.velocity.0 * cos_theta));
    let coords_at_wall = position(&t_at_wall);
    let did_hit_wall = coords_at_wall.y.is_positive() && coords_at_wall.y < params.wall_height;

    // Build up cannon-ball trajectory:
    let mut path = Vec::new();
    let mut t = Seconds(0.0);
    let mut y = Metres(0.0);
    while t.0 == 0.0 || (did_hit_wall && t < t_at_wall) || (!did_hit_wall && y.is_positive()) {
        t = Seconds(t.0 + params.simulation_step_size.0);
        let coords = position(&t);
        y = coords.y.clone();
        path.push(coords);
    }
    Trajectory(path)
}

// We maximize how far the cannon ball has travelled horizontally.
fn evaluate(p: &FiringPlan, params: &Params) -> Fitness {
    let traj = simulate(&p, &params);
    Fitness(traj.distance().0)
}

#[derive(Debug)]
struct Individual {
    plan: FiringPlan,
    fitness: Fitness,
}

struct Params {
    wall_height: Metres,
    wall_distance: Metres,
    simulation_step_size: Seconds,
    seed: u64,
    pop_size: usize,
    num_evaluations: usize,
}

fn main() {
    let params = Params {
        wall_height: Metres(25.0),
        wall_distance: Metres(10.0),
        simulation_step_size: Seconds(0.01),
        seed: 1,
        pop_size: 25,
        num_evaluations: 3000,
    };

    let mut rng: StdRng = SeedableRng::seed_from_u64(params.seed);

    fn mutate<R: Rng>(plan: &FiringPlan, rng: &mut R) -> FiringPlan {
        FiringPlan::new(
            MetresPerSecond((plan.velocity.0 + rng.gen::<f64>() - 0.5).max(0.1)),
            Radians(
                (plan.angle.0 + rng.gen::<f64>() - 0.5)
                    .max(0.1)
                    .min(PI / 2.0),
            ),
        )
    }

    let mut best_fitness_to_date = Fitness(0.0);

    let mut ps = FiringPlan::randoms(&mut rng, params.pop_size);

    for r in 0..params.num_evaluations {
        let mut pop: Vec<Individual> = ps
            .iter()
            .map(|plan| {
                let fitness = evaluate(&plan, &params);
                Individual {
                    plan: plan.clone(),
                    fitness,
                }
            })
            .collect();

        pop.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

        if let Some(best) = pop.first() {
            if best.fitness > best_fitness_to_date {
                best_fitness_to_date = best.fitness.clone();
                println!("Epoc {} Fitness {}", r, best.fitness.0);
                let traj = simulate(&best.plan, &params);
                traj.save(&format!("traj/{}.dat", r));
            }
        }

        for i in 0..params.pop_size {
            ps[i] = if i <= 1 {
                pop[i].plan.clone()
            } else {
                mutate(&(pop[i].plan), &mut rng)
            }
        }
    }
}
