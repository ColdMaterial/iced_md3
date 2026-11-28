/// Returns a score for a case-insensitive substring or subsequence match.
pub fn score(query: &str, candidate: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }

    let query: Vec<char> = query.chars().flat_map(char::to_lowercase).collect();
    let candidate: Vec<char> = candidate.chars().flat_map(char::to_lowercase).collect();

    if query.len() > candidate.len() {
        return None;
    }

    if let Some(position) = find_subslice(&candidate, &query) {
        let mut score = 1_000;

        if position == 0 {
            score += 300;
        } else if !candidate[position - 1].is_alphanumeric() {
            score += 150;
        }

        score -= position as i32;
        score -= (candidate.len() - query.len()).min(100) as i32;

        return Some(score);
    }

    let mut query_index = 0;
    let mut score = 0;
    let mut previous = None;

    for (index, &character) in candidate.iter().enumerate() {
        if character != query[query_index] {
            continue;
        }

        score += 10;

        if index == 0 || !candidate[index - 1].is_alphanumeric() {
            score += 20;
        }

        match previous {
            Some(previous) if index == previous + 1 => score += 15,
            Some(_) => score -= 5,
            None => score -= index.min(20) as i32,
        }

        previous = Some(index);
        query_index += 1;

        if query_index == query.len() {
            break;
        }
    }

    if query_index == query.len() {
        score -= (candidate.len().min(40) / 4) as i32;
        Some(score)
    } else {
        None
    }
}

fn find_subslice(haystack: &[char], needle: &[char]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::score;

    #[test]
    fn substring_matches_before_subsequences() {
        let scores = [
            ("one item", score("item", "one item").unwrap()),
            ("I t e m", score("item", "I t e m").unwrap()),
            ("item", score("item", "item").unwrap()),
        ];
        assert!(scores[2].1 > scores[0].1);
        assert!(scores[0].1 > scores[1].1);
    }

    #[test]
    fn prefix_and_earlier_matches_rank_higher() {
        assert!(score("op", "opaque") > score("op", "top"));
        assert!(score("op", "top") > score("op", "stop"));
    }

    #[test]
    fn rejects_non_matches_and_handles_empty_query() {
        assert_eq!(score("zzz", "Option 01"), None);
        assert_eq!(score("", "anything"), Some(0));
    }

    #[test]
    fn matches_subsequences_case_insensitively() {
        assert!(score("o1", "Option 01").is_some());
        assert!(score("x1", "Option 01").is_none());
    }
}
