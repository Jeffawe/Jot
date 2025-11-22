#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    Knowledge,   // User wants general help/command info
    Retrieval,   // User wants to search their history
}

pub fn classify_intent(query: &str) -> Intent {
    let q = query.to_lowercase();
    
    // Strong knowledge indicators
    if q.starts_with("how to") 
        || q.starts_with("how do i")
        || q.starts_with("command to")
        || q.starts_with("command for")
        || q.starts_with("what is the command") {
        return Intent::Knowledge;
    }
    
    // Has temporal markers? Definitely retrieval
    if q.contains("yesterday")
        || q.contains("last week")
        || q.contains("last month")
        || q.contains("today")
        || q.contains("ago")
        || q.contains("i used")
        || q.contains("i ran")
        || q.contains("i did") {
        return Intent::Retrieval;
    }
    
    // Default to retrieval (main use case)
    Intent::Retrieval
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_knowledge_intent() {
        assert_eq!(classify_intent("command to checkout git branch"), Intent::Knowledge);
        assert_eq!(classify_intent("how to merge branches"), Intent::Knowledge);
    }
    
    #[test]
    fn test_retrieval_intent() {
        assert_eq!(classify_intent("ssh i used yesterday"), Intent::Retrieval);
        assert_eq!(classify_intent("show me build commands"), Intent::Retrieval);
    }
}