extern crate time;
extern crate serialize;

use time::precise_time_ns;
use serialize::Encodable;

/// Represents a simple score object.
/// Scores are comprised of a user_id, a score and a registration time offset relative to some future point.
#[deriving(Hash, Eq, Show, Clone, Send, Encodable)]
pub struct Score {
    pub user_id:    String,
    pub score:      uint,
    timestamp:  u64,
}

impl Score {
    /// Creates a new `Score` instance, `timestamp` is usually not specified explicitly.
    pub fn new(user_id: String, score: uint) -> Score {
        Score { user_id: user_id, score: score, timestamp: time::precise_time_ns() }
    }

    /// Creates a new `Score` instance with an explicit `timestamp`.
    /// Note: I would prefer to use another new.. but Rust doesn't allow duplicate function names.
    fn with_timestamp(user_id: String, score: uint, timestamp: u64) -> Score {
        Score { user_id: user_id, score: score, timestamp: timestamp }
    }
}

impl PartialEq for Score {
    fn eq(&self, other: &Score) -> bool {
        self.user_id == other.user_id && self.score == other.score && self.timestamp == other.timestamp
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Score) -> Ordering {
        match self.score.cmp(&other.score) {
            Equal => match self.timestamp.cmp(&other.timestamp) {
                Equal => self.user_id.cmp(&other.user_id),
                Less => Greater,
                Greater => Less,
            },
            ordering => ordering
        } 
    }
}

impl PartialOrd for Score {

    fn partial_cmp(&self, other: &Score) -> Option<Ordering> {
        match (!self.lt(other), !other.lt(self)) {
           (false, false) => None,
           (false, true) => Some(Less),
           (true, false) => Some(Greater),
           (true, true) => Some(Equal),
        }
    }
    
    fn lt(&self, other: &Score) -> bool {  
        match self.cmp(other) {
            Less => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test_score {
    use score::Score;
    /// Test score sort orders
    /// Scores are sorted in the following order:
    ///     score (DESC)
    ///     timestamp(ASC)
    ///     user_id(DESC)
    #[test]
    fn test_score_sort_order() {
        let mut highScore = Score::new(String::from_str("1"), 2);
        let mut lowScore = Score::new(String::from_str("1"), 2);
        // highScore was registered before lowscore.
        assert!(highScore > lowScore);
        
        // highScore has a higher score than lowScore.
        lowScore = Score::new(String::from_str("1"), 1);
        highScore = Score::new(String::from_str("1"), 2);
        assert!(highScore > lowScore);
    
       // highScore user_id has a higher ordering than lowScore user_id.
        lowScore = Score::with_timestamp(String::from_str("1"), 1, 1);
        highScore = Score::with_timestamp(String::from_str("2"), 1, 1);
        assert!(highScore > lowScore);
    }

    /// A score object is equal to itself,
    /// or to another score object with exactly the same parameters.
    #[test]
    fn test_score_equivalence() {
       let mut score = Score::new(String::from_str("1"), 2);
    
        // Self equivalence.
        assert!(score == score);

        score = Score::with_timestamp(String::from_str("1"), 1, 1);
        let otherScore = Score::with_timestamp(String::from_str("1"), 1, 1);
    
        // Field equivalence.
        assert!(score == otherScore);
    }
}
