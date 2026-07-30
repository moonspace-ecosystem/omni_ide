use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Chunk {
    pub source_id: String,
    pub text: String,
    pub index: usize,
}

pub struct RAGIndex {
    chunks: Vec<Chunk>,
    // Simple TF-IDF / BM25 term frequency index
    // Word -> Vec<(chunk_index, frequency)>
    tf_index: HashMap<String, Vec<(usize, f64)>>,
}

impl RAGIndex {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            tf_index: HashMap::new(),
        }
    }

    pub fn index_document(&mut self, source_id: &str, text: &str) {
        // Chunk by paragraphs or lines (say 200 words per chunk)
        let words: Vec<&str> = text.split_whitespace().collect();
        let chunk_size = 200;
        let mut i = 0;
        let mut chunk_idx = self.chunks.len();
        
        while i < words.len() {
            let end = std::cmp::min(i + chunk_size, words.len());
            let chunk_text = words[i..end].join(" ");
            
            let chunk = Chunk {
                source_id: source_id.to_string(),
                text: chunk_text.clone(),
                index: chunk_idx,
            };
            self.chunks.push(chunk);
            
            // Index the words for search
            let mut word_counts = HashMap::new();
            for word in chunk_text.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
                if word.len() > 2 {
                    *word_counts.entry(word.to_string()).or_insert(0.0) += 1.0;
                }
            }
            
            for (word, count) in word_counts {
                self.tf_index.entry(word).or_default().push((chunk_idx, count));
            }
            
            chunk_idx += 1;
            i += chunk_size - 50; // overlap of 50 words
        }
    }

    pub fn query(&self, question: &str, top_k: usize) -> Vec<Chunk> {
        let mut scores = HashMap::new();
        let query_words: Vec<String> = question
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .map(|w| w.to_string())
            .collect();
            
        for word in query_words {
            if let Some(postings) = self.tf_index.get(&word) {
                // IDF score
                let idf = ((self.chunks.len() as f64) / (postings.len() as f64)).ln();
                for &(chunk_idx, tf) in postings {
                    *scores.entry(chunk_idx).or_insert(0.0) += tf * idf;
                }
            }
        }
        
        let mut scored_chunks: Vec<(usize, f64)> = scores.into_iter().collect();
        scored_chunks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        scored_chunks
            .into_iter()
            .take(top_k)
            .map(|(idx, _)| self.chunks[idx].clone())
            .collect()
    }
}
