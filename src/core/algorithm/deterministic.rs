pub fn stable_order(node_count: usize) -> Vec<usize> {
    (0..node_count).collect()
}

pub fn tie_break_community(c1: usize, c2: usize) -> usize {
    std::cmp::min(c1, c2)
}

pub fn tie_break_gain(g1: f64, c1: usize, g2: f64, c2: usize) -> (f64, usize) {
    if (g1 - g2).abs() < 1e-9 {
        if c1 < c2 {
            (g1, c1)
        } else {
            (g2, c2)
        }
    } else if g1 > g2 {
        (g1, c1)
    } else {
        (g2, c2)
    }
}
