#[allow(dead_code)]
#[derive(Debug)]
pub struct ClipboardEntry {
    pub timestamp: u64,
    pub context: SimplifiedWindowInfo,
    pub content: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SimpleProcessInfo {
    pub process_id: u32,
    pub path: String,
    pub name: String,
    pub exec_name: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SimplifiedWindowInfo {
    pub id: u32, // Assuming u32 based on the input
    pub os: String,
    pub title: String,
    pub info: SimpleProcessInfo,
}