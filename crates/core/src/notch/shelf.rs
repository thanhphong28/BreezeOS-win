#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub path: String,
}

impl Item {
    pub fn name(&self) -> &str {
        let p = self.path.trim_end_matches(['\\', '/']);
        p.rsplit(['\\', '/']).next().unwrap_or(p)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shelf {
    items: Vec<Item>,
    max: usize,
}

impl Shelf {
    pub fn new(max: usize) -> Shelf {
        Shelf { items: Vec::new(), max: max.max(1) }
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn position(&self, path: &str) -> Option<usize> {
        self.items.iter().position(|i| i.path.eq_ignore_ascii_case(path))
    }

    pub fn add(&mut self, path: &str) -> bool {
        let path = path.trim();
        if path.is_empty() {
            return false;
        }
        if let Some(i) = self.position(path) {
            self.items.remove(i);
        }
        self.items.insert(0, Item { path: path.to_string() });
        self.items.truncate(self.max);
        true
    }

    pub fn remove(&mut self, path: &str) -> bool {
        match self.position(path) {
            Some(i) => {
                self.items.remove(i);
                true
            }
            None => false,
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn serialize(&self) -> String {
        self.items.iter().map(|i| i.path.as_str()).collect::<Vec<_>>().join("\n")
    }

    pub fn parse(src: &str, max: usize) -> Shelf {
        let mut s = Shelf::new(max);
        for line in src.lines().rev() {
            s.add(line);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_first_without_duplicates_and_capped() {
        let mut s = Shelf::new(3);
        assert!(s.add(r"C:\a\Laporan.pdf"));
        s.add(r"C:\b\foto.png");
        s.add(r"c:\A\laporan.PDF");
        assert_eq!(s.len(), 2);
        assert_eq!(s.items()[0].name(), "laporan.PDF");
        s.add(r"C:\c\1.txt");
        s.add(r"C:\c\2.txt");
        assert_eq!(s.len(), 3, "dibatasi");
        assert_eq!(s.items()[2].name(), "laporan.PDF");
        assert!(!s.add("   "));
        assert!(s.remove(r"C:\C\2.TXT"));
        assert!(!s.remove(r"C:\none"));
    }

    #[test]
    fn names_and_round_trip() {
        assert_eq!(Item { path: r"D:\Folder\Sub\".into() }.name(), "Sub");
        assert_eq!(Item { path: "C:/x/y.zip".into() }.name(), "y.zip");
        let mut s = Shelf::new(10);
        for p in [r"C:\1", r"C:\2", r"C:\3"] {
            s.add(p);
        }
        let back = Shelf::parse(&s.serialize(), 10);
        assert_eq!(back, s);
    }
}
