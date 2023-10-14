use crate::core::generator::Generator;
use rand::Rng;

#[derive(Debug, Default)]
pub struct RandomGenerator;

impl Generator for RandomGenerator {
    fn generate(&self) -> String {
        let mut rng = rand::thread_rng();
        let n = rng.gen_range(0..1000000);
        format!("{:06}", n)
    }
}
