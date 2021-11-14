use alloc::vec::Vec;

use hashbrown::HashMap;

/// Helper function to calculate statistics of a provided dataset
pub fn calculate_stats(vec: &Vec<u64>) -> Option<Stats> {
  let mean;
  let median;
  let mode;
  let p_75;
  let p_25;
  let min;
  let max;
  let var;
  let std_dev;

  if vec.is_empty() {
    return None;
  }

  let len = vec.len();

  { // calculate average
    let sum: u64 = vec.iter().sum();
    mean = sum as f64 / len as f64;
  }

  { // calculate median
    let mut vec2 = vec.clone();
    vec2.sort();
    let mid = len / 2;
    let i_75 = len * 3 / 4;
    let i_25 = len * 1 / 4;

    median = vec2[mid];
    p_25 = vec2[i_25];
    p_75 = vec2[i_75];
    min = vec2[0];
    max = vec2[len - 1];
  }

  { // calculate sample variance
    let mut diff_sum: f64 = 0.0;
    for &val in vec {
      let x = val as f64;
      if x > mean {
        diff_sum = diff_sum + ((x - mean) * (x - mean));
      } else {
        diff_sum = diff_sum + ((mean - x) * (mean - x));
      }
    }

    var = (diff_sum) / (len as f64);
    std_dev = libm::sqrt(var);
  }

  { // calculate mode
    let mut values: HashMap<u64, usize> = HashMap::with_capacity(len);
    for val in vec {
      values.entry(*val).and_modify(|v| { *v += 1 }).or_insert(1);
    }
    mode = *values.iter().max_by(|(_k1, v1), (_k2, v2)| v1.cmp(v2)).unwrap().0; // safe to call unwrap since we've already checked if the vector is empty
  }

  Some(Stats { min, p_25, median, p_75, max, mode, mean, std_dev })
}

pub struct Stats {
  pub min: u64,
  pub p_25: u64,
  pub median: u64,
  pub p_75: u64,
  pub max: u64,
  pub mode: u64,
  pub mean: f64,
  pub std_dev: f64,
}

impl core::fmt::Debug for Stats {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "stats \n
        min:     {} \n
        p_25:    {} \n
        median:  {} \n
        p_75:    {} \n
        max:     {} \n
        mode:    {} \n
        mean:    {} \n
        std_dev: {} \n",
           self.min, self.p_25, self.median, self.p_75, self.max, self.mode, self.mean, self.std_dev)
  }
}