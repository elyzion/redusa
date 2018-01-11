
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::collections::{BTreeSet,BTreeMap};
use std::sync::{Once, ONCE_INIT};

use score::Score;
use config::HIGHSCORE_LIST_SIZE;

pub struct LevelRepository {
    levels:     BTreeMap<u64, Level>,
    lock:       Mutex<usize>
}

impl LevelRepository {

    pub fn new() -> LevelRepository {
        LevelRepository {
            levels: BTreeMap::new(),
            lock:   Mutex::new(1)
        }
    }

    pub fn get_level_high_scores(&self, levelId: u64) -> Option<BTreeSet<Score>> {
        match self.levels.get(&levelId) {
            Some(level) => Some(level.get_high_scores()),
            None => None
        }
    }

    pub fn add_score(&mut self, levelId: u64, userId: String, points: u64) -> bool {
        if self.levels.contains_key(&levelId) {
            match self.levels.get_mut(&levelId) {
                Some(ref mut level) => level.add_score(userId, points),
                None => true
            }
        } else {
            self.lock.lock();
            let ret = if !self.levels.contains_key(&levelId) {
                let mut level = Level::new();
                level.add_score(userId, points);
                match self.levels.insert(levelId, level) {
                    Some(b) => false,
                    None => true
                }
            } else {
                self.levels.get_mut(&levelId).unwrap().add_score(userId, points)
            };
            ret
        }
    }

    pub fn reset(&mut self) {
        self.levels.clear();
    }

}

/// Reddit <3 TODO: This should be mutable.
pub mod repository {
    use super::LevelRepository;
    use std::collections::BTreeMap;
    use std::sync::{Mutex, Once, ONCE_INIT};
    use std::mem;

    static mut REPOSITORY: *const LevelRepository = 0 as *const LevelRepository;

    pub struct Inited { x: () }
    pub fn init() -> Inited {
        unsafe {
            static mut ONCE: Once = ONCE_INIT;
            ONCE.call_once(|| {
                REPOSITORY = mem::transmute(Box::new(LevelRepository::new()));
            });
        }
        Inited { x: () }
    }

    pub fn get(evidence: Inited) -> &'static mut LevelRepository {
        unsafe {  mem::transmute(REPOSITORY) }
    }
}


pub struct Level {
    scores:     BTreeSet<Score>,
    highScores: BTreeSet<Score>,
    lock:       Mutex<usize>,
    counter:    AtomicUsize,
}

impl Level {
    pub fn new() -> Level {
        Level { 
            scores: BTreeSet::new(), 
                        highScores: BTreeSet::new(), 
                        lock: Mutex::new(0),
                        counter: AtomicUsize::new(0)
        }
    }

    pub fn get_high_scores(&self) -> BTreeSet<Score> {
        // Clone so that we don't have to worry about overwrites
        self.highScores.clone()
    }

    pub fn add_score(&mut self, user_id: String, points: u64) -> bool {
        let score = Score::new(user_id.clone(), points.clone());

        let min = match self.scores.iter().last() {
            Some(m) => Some(m.clone()),
            None    => None
        };

        if min.is_some() && min.as_ref().unwrap() > &score && self.counter.load(SeqCst) == HIGHSCORE_LIST_SIZE {
            return false
        } else if self.scores.contains(&score) {
            return false
        }

        self.lock.lock();
        let userScore = match self.scores.iter().find(|x| x.user_id == score.user_id) {
            Some(u) => Some(u.clone()),
            None    => None
        };

        if userScore.is_some() && userScore.as_ref().unwrap() > &score {
            return false
        }

        if self.scores.insert(score) {
            if userScore.is_some() {
                self.scores.remove(userScore.as_ref().unwrap());
            } else if self.counter.load(SeqCst) < HIGHSCORE_LIST_SIZE {
                self.counter.fetch_add(1, SeqCst);
            } else {
                let remove = match self.scores.iter().last() {
                    Some(l) => Some(l.clone()),
                    None => None
                };
                self.scores.remove(&remove.unwrap());
            }
            self.update_high_scores();
            return true
        } else {
            self.update_high_scores();
            return false
        }
    }

    fn update_high_scores(&mut self) {
        self.highScores = self.scores.clone();
    }

    pub fn get_user_score<'s>(&'s self, user_id: String) -> Option<&'s Score> {
        for s in self.scores.iter() {
            if s.user_id == user_id {
                return Some(s)
            }
        }
        None
    }

    pub fn get_score(&self, score: u64) -> Option<BTreeSet<Score>> {
        let scores: BTreeSet<Score> = 
            self.scores
            .iter()
            .filter(|ref x| x.score == score) 
            .map(|x| x.clone())
            .collect();
        if !scores.is_empty() {
            Some(scores)
        } else {
            None
        }
    }
}



#[cfg(test)]
mod test_level_repository {
    use level::repository;
    use level::LevelRepository;

    /// TODO: Singleton ownership is broken.
    #[test]
    fn test_construct_level_repository() {
        let evi = repository::init();
        repository::get(evi);
    }

    #[test]
    fn test_add_score_repository() {
        let mut repo = LevelRepository::new();
        assert!(repo.add_score(1, "1".to_string(), 1));
        assert!(repo.get_level_high_scores(1).unwrap().len() == 1);
    }
}

#[cfg(test)]
mod test_level {
    use level::Level;
    use config::HIGHSCORE_LIST_SIZE;

    #[test]
    fn test_get_user_id() {
        let mut level = Level::new();
        level.add_score("1".to_string(), 2);
        level.add_score("2".to_string(), 3);
        assert_eq!(level.get_user_score("3".to_string()), None);
        assert!(level.get_user_score("1".to_string()).is_some());
        assert_eq!(level.get_user_score("2".to_string()).unwrap().score, 3);
    }

    #[test]
    fn test_get_score() {
        let mut level = Level::new();
        level.add_score("1".to_string(), 2);
        level.add_score("2".to_string(), 3);
        assert_eq!(level.get_score(4), None);
        assert!(level.get_score(2).is_some());
        assert_eq!(level.get_score(2).unwrap().len(), 1);
        assert_eq!(level.get_score(2).unwrap().iter().last().unwrap().user_id, "1".to_string());
    }

    #[test]
    fn test_size_limit() {
        let mut level = Level::new();
        for i in 0..HIGHSCORE_LIST_SIZE {
            level.add_score(i.to_string(), i as u64);
        }

        assert_eq!(level.scores.len(), 15);
        level.add_score("200".to_string(), 200);
        assert_eq!(level.scores.len(), 15);
    }

    //TODO
    fn test_high_score_write_concurrency() {}

    //TODO
    fn test_high_score_read_write_concurrency() {}

}
