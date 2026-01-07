pub const B: usize = 6;

pub struct BtreeNode<K: Ord, V: usize> {
    n: u64, // number of keys currently stored in node x
    // leaf is a boolean value which is TRUE if the x is a leaf node and FALSE if x is a internal
    // node
    // leaf node -> no childrens
    // intenal node -> at least one childrens
    leaf: bool,
    keys: Vec<K>,
    // children/edges of internal node
    children: Vec<Box<BtreeNode<K, V>>; 2 * B>,
    vals: Vec<V>,
}

impl<K: Ord, V: usize> BtreeNode<K, V> {
    pub fn search(&mut self, key: &K) -> Option<usize> {
        let mut i = 0;
        while i <= self.n && key > self.keys[i] {
            i += 1;
        }
        if i < self.n && key == self.keys[i] {
            return i;
        }
        if self.leaf() {
            return None;
        } else {
            self.children[i].search(key);
        }
    }
}
