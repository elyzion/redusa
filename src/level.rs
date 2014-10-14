
use std::sync::Semaphore;
use std::sync::atomics::{AtomicInt, SeqCst};
use std::collections::treemap::{TreeSet, TreeMap};

use score::Score;
use config::HIGHSCORE_LIST_SIZE;

pub struct LevelRepository {
    levels:     TreeMap<uint, Level>,
    lock:       Semaphore
}

impl LevelRepository {

    pub fn new() -> LevelRepository {
        LevelRepository {
            levels: TreeMap::new(),
            lock:   Semaphore::new(1)
        }
    }

    pub fn get_level_high_scores(&self, levelId: uint) -> Option<TreeSet<Score>> {
        match self.levels.find(&levelId) {
            Some(level) => Some(level.get_high_scores()),
            None => None
        }
    }

    pub fn add_score(&mut self, levelId: uint, userId: String, points: uint) -> bool {
        if self.levels.contains_key(&levelId) {
            match self.levels.find_mut(&levelId) {
                Some(ref mut level) => level.add_score(userId, points),
                None => true
            }
        } else {
            self.lock.acquire();
            let ret = if !self.levels.contains_key(&levelId) {
                let mut level = Level::new();
                level.add_score(userId, points);
                self.levels.insert(levelId, level)
            } else {
                self.levels.find_mut(&levelId).unwrap().add_score(userId, points)
            };
            self.lock.release();
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
    use std::collections::TreeMap;
    use std::sync::{Semaphore, Once, ONCE_INIT};
    use std::mem;

    static mut REPOSITORY: *const LevelRepository = 0 as *const LevelRepository;

    pub struct Inited { x: () }
    pub fn init() -> Inited {
        unsafe {
            static mut ONCE: Once = ONCE_INIT;
            ONCE.doit(|| {
                REPOSITORY = mem::transmute(box LevelRepository::new());
            });
        }
        Inited { x: () }
    }

    pub fn get(evidence: Inited) -> &'static mut LevelRepository {
        unsafe {  mem::transmute(REPOSITORY) }
    }
}


pub struct Level {
    scores:     TreeSet<Score>,
    highScores: TreeSet<Score>,
    lock:       Semaphore,
    counter:    AtomicInt,
}

impl Level {
    pub fn new() -> Level {
        Level { 
            scores: TreeSet::new(), 
                        highScores: TreeSet::new(), 
                        lock: Semaphore::new(1),
                        counter: AtomicInt::new(0)
        }
    }

    pub fn get_high_scores(&self) -> TreeSet<Score> {
        // Clone so that we don't have to worry about overwrites
        self.highScores.clone()
    }

    pub fn add_score(&mut self, user_id: String, points: uint) -> bool {
        let score = Score::new(user_id.clone(), points.clone());

        let min = match self.scores.iter().last() {
            Some(m) => Some(m.clone()),
            None    => None
        };

        if min.is_some() && min.get_ref() > &score && self.counter.load(SeqCst) == HIGHSCORE_LIST_SIZE {
            return false
        } else if self.scores.contains(&score) {
            return false
        }

        self.lock.acquire();
        let userScore = match self.scores.iter().find(|x| x.user_id == score.user_id) {
            Some(u) => Some(u.clone()),
            None    => None
        };

        if userScore.is_some() && userScore.get_ref() > &score {
            return false
        }

        if self.scores.insert(score) {
            if userScore.is_some() {
                self.scores.remove(userScore.get_ref());
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
            self.lock.release();
            return true
        } else {
            self.update_high_scores();
            self.lock.release();
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

    pub fn get_score(&self, score: uint) -> Option<TreeSet<Score>> {
        let scores: TreeSet<Score> = 
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
        for i in range(0, HIGHSCORE_LIST_SIZE) {
            level.add_score(i.to_string(), i.to_uint().unwrap());
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
