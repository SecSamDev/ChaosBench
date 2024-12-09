use serde::{Deserialize, Serialize};

pub mod user_actions;
pub mod agent;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Log {
    pub agent : String,
    pub msg : String
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestingReport {
    pub id : String,
    pub date : i64,
    pub report : String
}

impl TestingReport {
    pub fn add_h1(&mut self, header : &str) {
        self.report.push_str("# ");
        self.report.push_str(header);
        self.report.push('\n');
    }
    pub fn add_h2(&mut self, header : &str) {
        self.report.push_str("## ");
        self.report.push_str(header);
        self.report.push('\n');
    }
    pub fn add_h3(&mut self, header : &str) {
        self.report.push_str("### ");
        self.report.push_str(header);
        self.report.push('\n');
    }
    pub fn add_content(&mut self, txt : &str) {
        self.report.push_str(txt);
        self.report.push('\n');
    }
    pub fn add_table_header(&mut self, list : &[&str]) {
        if list.is_empty() {
            return
        }
        self.report.push('|');
        for header in list {
            self.report.push_str(header);
            self.report.push('|');
        }
        self.report.push('\n');
        self.report.push('|');
        for _ in list {
            self.report.push_str("-----|");
        }
        self.report.push('\n');
    }
    pub fn add_table_row(&mut self, list : &[&str]) {
        if list.is_empty() {
            return
        }
        self.report.push('|');
        for header in list {
            self.report.push_str(header);
            self.report.push('|');
        }
        self.report.push('\n');
    }
}