//! 向量搜索模块
//!
//! 基于 ONNX Runtime 的本地 embedding（当前为随机向量占位）。
//! 后续集成 sqlite-vec 后替换为持久化向量索引。

use serde::{Deserialize, Serialize};

/// 向量维度（与 all-MiniLM-L6-v2 对齐）
pub const EMBEDDING_DIM: usize = 384;

/// Embedding 服务（当前为随机向量占位）
pub struct EmbeddingService;

impl EmbeddingService {
    pub fn new() -> Self {
        Self
    }

    /// 生成文本的向量表示
    ///
    /// 当前使用简单哈希占位，后续替换为 ONNX Runtime 模型推理。
    pub fn embed(&self, text: &str) -> Vec<f32> {
        // 简单哈希占位：基于文本内容生成确定性伪向量
        let mut vec = Vec::with_capacity(EMBEDDING_DIM);
        let bytes = text.as_bytes();
        for i in 0..EMBEDDING_DIM {
            let b = bytes.get(i % bytes.len()).copied().unwrap_or(0);
            // 归一化到 [-1, 1] 范围
            let val = ((b as f32) / 255.0) * 2.0 - 1.0;
            vec.push(val);
        }
        // 归一化
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in vec.iter_mut() {
                *v /= norm;
            }
        }
        vec
    }
}

/// 计算余弦相似度
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|v| v * v).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

/// 向量索引条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: i64,
    pub embedding: Vec<f32>,
}

/// 内存向量搜索器（sqlite-vec 占位）
pub struct EmbeddingSearcher {
    entries: Vec<VectorEntry>,
    service: EmbeddingService,
}

impl EmbeddingSearcher {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            service: EmbeddingService::new(),
        }
    }

    /// 索引一条记忆
    pub fn index_drawer(&mut self, id: i64, content: &str) {
        let embedding = self.service.embed(content);
        // 去重：如果 id 已存在则更新
        if let Some(existing) = self.entries.iter_mut().find(|e| e.id == id) {
            existing.embedding = embedding;
        } else {
            self.entries.push(VectorEntry { id, embedding });
        }
    }

    /// 搜索最相似的条目
    pub fn search(&self, query: &str, limit: usize) -> Vec<(i64, f32)> {
        let query_vec = self.service.embed(query);
        let mut scored: Vec<(i64, f32)> = self
            .entries
            .iter()
            .map(|e| (e.id, cosine_similarity(&query_vec, &e.embedding)))
            .filter(|(_, score)| *score > 0.0)
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        scored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_returns_correct_dimension() {
        let svc = EmbeddingService::new();
        let vec = svc.embed("hello");
        assert_eq!(vec.len(), EMBEDDING_DIM);
    }

    #[test]
    fn embed_is_deterministic() {
        let svc = EmbeddingService::new();
        let a = svc.embed("test input");
        let b = svc.embed("test input");
        assert_eq!(a, b);
    }

    #[test]
    fn cosine_similarity_identical() {
        let v = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b)).abs() < 1e-6);
    }

    #[test]
    fn searcher_index_and_search() {
        let mut searcher = EmbeddingSearcher::new();
        searcher.index_drawer(1, "今天天气很好");
        searcher.index_drawer(2, "我是一只猫");
        let results = searcher.search("天气", 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, 1);
    }

    #[test]
    fn searcher_dedup() {
        let mut searcher = EmbeddingSearcher::new();
        searcher.index_drawer(1, "原始内容");
        searcher.index_drawer(1, "更新内容");
        assert_eq!(searcher.entries.len(), 1);
    }
}
