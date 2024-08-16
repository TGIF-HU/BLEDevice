use serde::Serialize;
use std::collections::VecDeque;

/// 固定長の配列
/// 配列の要素数が最大値を超えると、最も古い要素を削除する
#[derive(Debug, Serialize)]
pub struct FixedQueue<T> {
    queue: VecDeque<T>,
    max_len: usize,
}

impl<T> FixedQueue<T> {
    // コンストラクタ
    pub fn new(max_len: usize) -> Self {
        FixedQueue {
            queue: VecDeque::with_capacity(max_len),
            max_len,
        }
    }

    // 要素を追加するメソッド
    pub fn push(&mut self, item: T) {
        if self.queue.len() == self.max_len {
            self.queue.pop_front(); // 最も古い要素を削除
        }
        self.queue.push_back(item); // 新しい要素を追加
    }

    pub fn get_queue(&self) -> &VecDeque<T> {
        &self.queue
    }
}
