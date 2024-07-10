use std::collections::{HashMap, HashSet};

use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha512};

#[derive(Debug)]
pub struct Block<T>
where
    T: Serialize + DeserializeOwned,
{
    data: T,                  // Data
    root_hashs: String,       // root hash
    other_hashs: Vec<String>, // other hashs
    trust: u32,
}

impl<T> Block<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn get_self_hash(&self) -> String {
        let mut hasher = Sha512::new();
        hasher.update(format!(
            "{}{}",
            self.trust,
            serde_json::to_string(&self.data).unwrap()
        ));
        format!("{:x}", hasher.finalize())
    }
    pub fn get_root_hashs(&mut self) {
        let mut hasher = Sha512::new();
        hasher.update(&self.get_self_hash());
        self.other_hashs.iter().for_each(|other| {
            hasher.update(other);
        });
        self.root_hashs = format!("{:x}", hasher.finalize());
    }
    pub fn check_request(&mut self, req: String) -> bool {
        req == self.root_hashs
    }
    pub fn update_others(&mut self, mut req: HashMap<String, Vec<String>>) {
        // 몇몇 접속되어있는 유저에게 랜덤하게 요청한 후, 각 값중 같은 값이 많은 쪽의 데이터를 others에 넣는다.
        // 접속이 끊어져 있는 동안, 다른 블록들에 대한 변경사항은 한번에 받기 위함
        let root = &self.root_hashs;
        let mut result = HashMap::new();
        result.insert(root.clone(), 1);
        req.iter().for_each(|(s, _)| {
            if let Some(v) = result.get_mut(s) {
                *v += 1;
            } else {
                result.insert(s.to_owned(), 1);
            }
        });

        let mut max_count = 0;
        let mut max_hash = "".to_string();

        result.iter().for_each(|(k, v)| {
            if max_count < *v {
                max_count = *v;
                max_hash = k.to_owned();
            }
        });

        self.other_hashs = req.remove(&max_hash).unwrap();
        self.get_root_hashs();
    }
    pub fn change_set(&mut self, change: HashMap<String, String>) {
        // 정상적인 요청으로 인해, 어떤 블록들의 해시값이 변경 되었을 때, 이를 적용시킴
        change.iter().for_each(|(k, v)| {
            if let Some(index) = self.other_hashs.iter().position(|p| *p == *k) {
                self.other_hashs.swap_remove(index);
                self.other_hashs.push(v.to_string());
            }
        });
    }
}
