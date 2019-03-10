use core::f64::consts::PI;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[derive(Debug)]
struct Trajectory {
    velocity: MetresPerSecond,
    angle: Radians,
}

impl Trajectory {
    fn new(velocity: MetresPerSecond, angle: Radians) -> Trajectory {
        // For cannon-ball firing the following two constraints are needed:
        // 1. So we actually move...
        assert!(velocity > MetresPerSecond(0.0));
        // 2. So we fire up, not into the ground:
        assert!(angle > Radians(0.0));
        assert!(angle < Radians(PI));

        Trajectory { velocity, angle }
    }
    fn random<R: Rng>(rng: &mut R) -> Trajectory {
        Trajectory::new(
            MetresPerSecond(1.0 + rng.gen::<f64>()),
            Radians(rng.gen::<f64>() * PI), // NB: Pi radians is 1/2 a circle
        )
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
struct MetresPerSecond(f64);

#[derive(Debug, PartialEq, PartialOrd)]
struct Radians(f64);

fn main() {
    let seed: u64 = 1;
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let t = Trajectory::random(&mut rng);
    println!("t is {:?}", t);

}
