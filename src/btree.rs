// Btree here
#[derive(Debug)]
pub struct BtreeNode<T>{
    n: u64, // number of keys
    keys: Vec<T>,
    is_leaf: bool,
    children: Vec<Box<BtreeNode<T>>>,
}

#[derive(Debug)]
pub struct Btree {
    t: u64, // minimum degree
    root: BtreeNode<T>
}

impl<T> BtreeNode<T> {
    pub fn new(is_leaf: bool) -> Self{
        n: keys.len(),
        keys: Vec::new(),
        children: Vec::new(),
        is_leaf
    }
    pub is_full(&self, t: usize) -> usize {
        self.n = 2 * t - 1
    }
    pub fn split_child(&mut self, i: usize, t: usize) {
        let mut z = BtreeNode::new(self.children[i].is_leaf);
        let y = &mut self.children[i];
        z.keys = y.keys.split_off(t);
        if !y.is_leaf() {
            z.children = y.children.split_off(t);
        }
        let middle = y.keys.remove(t - 1);
        self.children.insert(i + 1, Box::new(z));
        self.keys.insert(i, middle);
    }
    pub fn search(&self, k: &T) -> Option<(&'a Self, usize)>{
        let mut i = 0;
        while i < self.n && key > self.keys[i] {
            i += 1;
        }
        if i < self.n && k == self.keys[i] {
            Some((self, i))
        }
        if self.is_leaf {
            None
        } else{
            self.children[i].search(k)
        }
    }
    pub fn insert_non_full(&self, k: &T, t: usize) {
        let mut i = self.n;
        if self.is_leaf {
            while i > 1 && k < self.keys(i){
                self.keys(i) = self.keys(i+1);
                let i -= 1;
            }
            self.keys.insert(i, k);
        } else {
            while i > 1 && k < self.keys(i){
                let i -= 1;
            }
            if self.children[i].is_full() {
                self.split_child(i,t);
                if k > self.key(i) {
                    let i += 1;
                }
            }
            self.children[i].insert_non_full(k, t);
        }
    }
    pub fn fill(&mut self, idx: usize, t: usize) {
        if idx != 0 && self.children[idx - 1].keys.len() >= t {
            let mut child = &mut self.children[idx];
            let mut sib = &mut self.children[idx - 1];
            let sep = self.keys.remove(idx - 1);
            self.keys.insert(idx - 1, sib.keys.pop().unwrap());
            child.keys.insert(0, sep);
            if !child.is_leaf {
                let moved_child = sib.keys.pop().unwrap();
                child.children.insert(0, moved_child);
            }
        } else if idx < self.keys.len() && self.children[idx + 1].keys.len() >= t {
            let mut child = &mut self.children[idx];
            let mut sib = &mut self.children[idx + 1];
            let sep = self.keys.remove(idx);
            self.keys.insert(idx - 1, sib.keys.remove(0));
            child.keys.push(sep);
            if !child.is_leaf {
                let moved_child = sib.keys.remove(0);
                child.children.push(moved_child);
            }
        } else {
            let merge_idx = if idx != self.keys.len() { idx } else { idx - 1 };
            let mut sib = self.children.remove(merge_idx + 1);
            let sep = self.keys.remove(merge_idx);
            let child = &mut self.remove[merge_idx];
            child.keys.push(sep);
            child.keys.append(&mut sib.keys);
            if !child.is_leaf {
                child.children.append(&mut sib.children);
            }
        }
    }
    pub fn remove(&mut self, k: &T, t: usize) -> bool {
        let mut idx = 0;
        while idx < self.keys.len() && k > &self.keys[idx] {
            idx += 1;
        }

        if idx < self.keys.len() && k == &self.keys[idx] {
            // Key found
            if self.is_leaf {
                self.keys.remove(idx);
            } else {
                // Internal node
                if self.children[idx].keys.len() >= t {
                    let pred = self.children[idx].get_max().clone();
                    self.keys[idx] = pred.clone();
                    self.children[idx].remove(&pred, t);
                } else if self.children[idx + 1].keys.len() >= t {
                    let succ = self.children[idx + 1].get_min().clone();
                    self.keys[idx] = succ.clone();
                    self.children[idx + 1].remove(&succ, t);
                } else {
                    self.fill(idx, t);
                    self.children[idx].remove(k, t);
                }
            }
            true
        } else {
            if self.is_leaf {
                return false; // Not found
            }
            // Key not found, recurse to child
            let flag = idx == self.keys.len();
            if self.children[idx].keys.len() < t {
                self.fill(idx, t);
            }
            let child_idx = if flag && idx > self.keys.len() { idx - 1 } else { idx };
            self.children[child_idx].remove(k, t)
        }
    }
    fn get_min(&self) -> &T {
        if self.is_leaf {
            &self.keys[0]
        } else {
            self.children[0].get_min()
        }
    }

    fn get_max(&self) -> &T {
        if self.is_leaf {
            &self.keys[self.keys.len() - 1]
        } else {
            self.children[self.children.len() - 1].get_max()
        }
    }
}

impl<T> Btree<T> {
    pub fn create(t: usize) -> Self{
        let x = BtreeNode::new(true);
        Self{
            root: Box::new(x),
            t
        }
    }
    pub fn search(&self, k: &T) -> Option<(&'a Self, usize){
        self.root.search(k)
    }
    pub fn insert(&mut self, k) {
        let r = &mut self.root;
        if r.is_full(self.t) {
            let mut s = BtreeNode::New(false);
            let mut old_root = Box::new(BtreeNode::new(true));
            std::mem::swap(&mut old_root, &mut self.root);
            s.children.push(old_root);
            s.split_child(0, self.t);
            s.insert_non_full(k, self.t);
            self.root = Box::new(s);
        } else {
            r.insert_non_full(k, self.t);
        }
    }
}
