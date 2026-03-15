#[derive(Debug, Clone)]
pub struct Node<V: Clone> {
    leaf: bool,
    keys: Vec<i64>,
    values: Vec<V>,
    children: Vec<Box<Node<V>>>
}

impl<V: Clone> Node<V> {
    pub fn new(leaf: bool) -> Self {
        Self { leaf, keys: Vec::new(), values: Vec::new(), children: Vec::new() }
    }
    pub fn search(&self, key: i64) -> Option<&V>{
        let mut i = 0;
        while i < self.keys.len() && key > self.keys[i] {
            i += 1;
        }
        if i < self.keys.len() && key == self.keys[i] {
            return Some(&self.values[i])
        }
        if self.leaf {
            None
        } else {
            self.children[i].search(key)
        }
    }
    pub fn split_child(&mut self, i: usize, t: usize) {
        let y = &mut self.children[i];
        let mut z = Box::new(Node::new(y.leaf));
        for _ in 0..(t - 1){
            z.keys.insert(0, y.keys.pop().unwrap());
            z.values.insert(0, y.values.pop().unwrap());
        }
        if !y.leaf {
            for _ in 0..t {
                z.children.insert(0, y.children.pop().unwrap())
            }
        }
        let mid_key = y.keys.pop().unwrap();
        let mid_val = y.values.pop().unwrap();
        self.keys.insert(i, mid_key);
        self.values.insert(i, mid_val);
        self.children.insert(i+1, z);
    }
    pub fn insert_not_full(&mut self, k: i64, value: V, t: usize) {
        let mut i = self.keys.len();
        if self.leaf {
            while i >= 1 && k < self.keys[i - 1] {
                i -= 1;
            }
            if i > 0 && self.keys[i-1] == k {
                self.values[i-1] =  value;
                return;
            }
            self.keys.insert(i, k);
            self.values.insert(i, value);
        } else {
            while i > 0 && k < self.keys[i - 1] {
                i -= 1;
            }
            if self.children[i].keys.len() == 2 * t - 1 {
                self.split_child(i, t);
                if k > self.keys[i] {
                    i += 1;
                } else if k == self.keys[i] {
                    self.values[i] = value;
                    return;
                }
            }
            self.children[i].insert_not_full(k, value, t);
        }
    }
    pub fn keys_in_order(&self, out: &mut Vec<i64>) {
        if self.leaf {
            out.extend(self.keys.iter().cloned());
        } else {
            for i in 0..self.keys.len() {
                self.children[i].keys_in_order(out);
                out.push(self.keys[i]);
            }
            if let Some(last) = self.children.last() {
                last.keys_in_order(out);
            }
        }
    }
}


#[derive(Debug, Clone)]
pub struct Btree<T: Clone> {
    t: usize,
    root: Box<Node<T>>
}

impl<V: Clone> Btree<V> {
    pub fn new(t: usize) -> Self {
        Self { t, root: Box::new(Node::new(true)) }
    }
    pub fn search(&self, key: i64) -> Option<&V> {
        self.root.search(key)
    }
    /// Returns all (key, value) pairs in ascending key order.
    pub fn in_order(&self) -> Vec<i64> {
        let mut out = Vec::new();
        self.root.keys_in_order(&mut out);
        out
    }
    pub fn insert(&mut self, key: i64, value: V) {
        if self.root.keys.len() == 2 * self.t - 1 {
            let mut s = Box::new(Node::new(false));
            s.children.push(self.root.clone());
            s.split_child(0, self.t);
            s.insert_not_full(key, value, self.t);
            self.root = s;
        } else {
            self.root.insert_not_full(key, value, self.t);
        }
    }
    pub fn keys_in_order(&self) -> Vec<i64> {
        let mut out = Vec::new();
        self.root.keys_in_order(&mut out);
        out
    }
}


