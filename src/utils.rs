use std::{collections::HashSet, ops::{Add, Mul, Sub}};

use rand::{seq::SliceRandom, thread_rng, Rng};



pub fn ease_in_out_sine(t: f32) -> f32 {
    0.5 * (1.0 - (std::f32::consts::PI * t).cos())
}

pub fn lerp<T, F>(from: T, to: T, t: F) -> T 
where 
    T : Add<Output = T> + Mul<F, Output = T> + Sub<Output = T> + Copy,
    F : Into<f32>
{
    from + (to - from) * t
}

#[inline]
pub fn format_time(seconds: f32) -> String {
    let mins = (seconds / 60.0).floor();
    let secs = seconds % 60.0;
    format!("{:.0}:{:02.0}", mins, secs)
}


pub trait RandomChooser<T> {
    fn choose_random(&self) -> Option<T>;
    fn choose_random_n(&self, n: usize) -> Vec<T>;
}
impl<T: Clone> RandomChooser<T> for Vec<T> {
    fn choose_random(&self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        
        let mut rng = thread_rng();
        let index = rng.gen_range(0..self.len());
        Some(self[index].clone()) 
    }

    fn choose_random_n(&self, n: usize) -> Vec<T> {
        let mut rng = thread_rng();
        let mut indices: Vec<usize> = (0..self.len()).collect();
        indices.shuffle(&mut rng);
        
        let count = n.min(self.len());
        indices.truncate(count);
        
        indices.iter()
               .map(|&i| self[i].clone())
               .collect()
    }
}

pub fn create_range(n: f32, m: usize) -> Vec<f32> {
    // calc the step size for the range
    let step = n / (m as f32 - 1.0);
    // gen elements by stepping from 0 up to n
    (0..m).map(|i| i as f32 * step).collect()
}

pub fn has_common_elements<T: Eq + std::hash::Hash>(vec1: &[T], vec2: &[T]) -> bool {
    let set: HashSet<_> = vec1.iter().collect();
    vec2.iter().any(|item| set.contains(item))
}