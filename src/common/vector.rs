use rand::Rng;

pub fn shuffle_vec<T>(vec: &mut Vec<T>) {
    let mut rng = rand::thread_rng();
    for i in 0..vec.len() {
        let j = rng.gen_range(0..vec.len());
        vec.swap(i, j);
    }
}

pub fn pop_where<T>(vec: &mut Vec<T>, predicate: impl Fn(&T) -> bool) -> Option<T> {
    for i in 0..vec.len() {
        if predicate(&vec[i]) {
            return Some(vec.remove(i));
        }
    }
    None
}
