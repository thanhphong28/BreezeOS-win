#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Device,
    Update,
    Notice,
    Reminder,
    Info,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pill {
    pub id: u64,
    pub kind: Kind,
    pub key: String,
    pub title: String,
    pub sub: String,
    pub until_ms: Option<i64>,
    pub at_ms: i64,
}

#[derive(Default, Debug)]
pub struct Queue {
    items: Vec<Pill>,
    next: u64,
}

pub const DEVICE_MS: i64 = 4_000;
pub const NOTICE_MS: i64 = 6_000;
pub const UPDATE_MS: i64 = 9_000;

impl Queue {
    pub fn push(&mut self, kind: Kind, key: &str, title: &str, sub: &str, now_ms: i64, linger_ms: Option<i64>) -> u64 {
        self.items.retain(|p| p.key != key);
        self.next += 1;
        let id = self.next;
        self.items.push(Pill { id, kind, key: key.to_string(), title: title.to_string(), sub: sub.to_string(), until_ms: linger_ms.map(|l| now_ms + l), at_ms: now_ms });
        if self.items.len() > 12 {
            let oldest = self.items.iter().enumerate().filter(|(_, p)| p.kind != Kind::Reminder).min_by_key(|(_, p)| p.at_ms).map(|(i, _)| i);
            if let Some(i) = oldest {
                self.items.remove(i);
            }
        }
        id
    }

    pub fn prune(&mut self, now_ms: i64) -> bool {
        let before = self.items.len();
        self.items.retain(|p| p.until_ms.is_none_or(|u| u > now_ms));
        before != self.items.len()
    }

    pub fn current(&self, now_ms: i64) -> Option<&Pill> {
        self.items
            .iter()
            .filter(|p| p.until_ms.is_none_or(|u| u > now_ms))
            .max_by(|a, b| a.kind.cmp(&b.kind).then(a.at_ms.cmp(&b.at_ms)).then(a.id.cmp(&b.id)))
    }

    pub fn dismiss(&mut self, id: u64) -> Option<Pill> {
        let i = self.items.iter().position(|p| p.id == id)?;
        Some(self.items.remove(i))
    }

    pub fn dismiss_key(&mut self, key: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|p| p.key != key);
        before != self.items.len()
    }

    pub fn next_expiry(&self) -> Option<i64> {
        self.items.iter().filter_map(|p| p.until_ms).min()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn higher_kind_wins_and_newest_breaks_ties() {
        let mut q = Queue::default();
        q.push(Kind::Device, "bt:a", "Earbuds", "Tersambung", 0, Some(DEVICE_MS));
        q.push(Kind::Reminder, "todo:1", "Rapat", "15:30", 10, None);
        q.push(Kind::Device, "bt:b", "Mouse", "Tersambung", 20, Some(DEVICE_MS));
        assert_eq!(q.current(30).unwrap().key, "todo:1");
        let id = q.current(30).unwrap().id;
        q.dismiss(id);
        assert_eq!(q.current(30).unwrap().key, "bt:b");
    }

    #[test]
    fn expiry_and_prune() {
        let mut q = Queue::default();
        q.push(Kind::Notice, "n", "WhatsApp", "Halo", 0, Some(1000));
        q.push(Kind::Update, "u", "Pembaruan", "0.5.1", 0, Some(3000));
        assert_eq!(q.next_expiry(), Some(1000));
        assert_eq!(q.current(500).unwrap().key, "n");
        assert_eq!(q.current(1500).unwrap().key, "u");
        assert!(q.prune(1500));
        assert_eq!(q.len(), 1);
        assert!(!q.prune(1500));
        assert!(q.current(4000).is_none());
    }

    #[test]
    fn same_key_replaces() {
        let mut q = Queue::default();
        q.push(Kind::Device, "drive:E", "Flashdisk", "Tersambung", 0, Some(DEVICE_MS));
        q.push(Kind::Device, "drive:E", "Flashdisk", "Dilepas", 10, Some(DEVICE_MS));
        assert_eq!(q.len(), 1);
        assert_eq!(q.current(20).unwrap().sub, "Dilepas");
        assert!(q.dismiss_key("drive:E"));
        assert!(q.is_empty());
    }

    #[test]
    fn queue_is_bounded_but_keeps_reminders() {
        let mut q = Queue::default();
        q.push(Kind::Reminder, "todo:x", "Ingat", "", 0, None);
        for i in 0..20 {
            q.push(Kind::Notice, &format!("n{i}"), "x", "", 1 + i, Some(NOTICE_MS));
        }
        assert!(q.len() <= 12);
        assert_eq!(q.current(30).unwrap().key, "todo:x");
    }
}
