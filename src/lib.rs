use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
mod collection;
use collection::Collection;

static DIGITS_PATTERN: &str = "(?P<index>(?P<padding>0*)\\d+)";
static FRAME_PATTERN: &str = "\\.(?P<index>(?P<padding>0*)\\d+)\\.\\D+\\d?$";
static VERSION_PATTERN: &str = "v(?P<index>(?P<padding>0*)\\d+)";


pub fn assemble<T: AsRef<str>>(
    iterable: &Vec<T>,
    patterns: Option<Vec<String>>,
) -> (Vec<Collection>, Vec<String>) {
    let mut compiled_patterns: Vec<Regex> = vec![];
    let mut collection_map: HashMap<(String, String, i32), Vec<i32>> = HashMap::new();
    let mut remainder: Vec<String> = Vec::new();

    match patterns {
        Some(patterns) => {
            for pattern in patterns.iter() {
                compiled_patterns.push(Regex::new(pattern).unwrap());
            }
        }
        None => {
            lazy_static! {
                static ref DIGITS_REGEX: Regex =
                    Regex::new(&*DIGITS_PATTERN).unwrap();
            }
            compiled_patterns.push(DIGITS_REGEX.to_owned());
        }
    }

    for item in iterable.iter() {
        let mut matched = false;

        for pattern in compiled_patterns.iter() {
            for captures in pattern.captures_iter(item.as_ref()) {
                let index_match = captures.name("index").unwrap();
                let head = &item.as_ref()[..index_match.start()];
                let tail = &item.as_ref()[index_match.end()..];

                let padding = match captures.name("padding") {
                    Some(_) => index_match.range().count() as i32,
                    None => 0,
                };

                let key = (head.to_string(), tail.to_string(), padding);
                let index = index_match.as_str().parse::<i32>().unwrap();
                if collection_map.contains_key(&key) {
                    collection_map.get_mut(&key).unwrap().push(index)
                } else {
                    collection_map.insert(key, vec![index]);
                }
                matched = true;
            }
        }
        if !matched {
            println!("not matched: {}", item.as_ref());
            remainder.push(item.as_ref().to_string());
        }
    }

    // sort the indexes in the collection map
    for (_k, v) in collection_map.iter_mut() {
        v.sort();
    }

    // form collections
    let mut collections = Vec::new();
    let mut merge_candidates = Vec::new();
    for ((head, tail, padding), indexes) in collection_map.iter() {
        collections.push(Collection::new(
            head.to_string(),
            tail.to_string(),
            *padding,
            indexes.to_vec(),
        ));

        if *padding == 0 {
            merge_candidates.push(Collection::new(
                head.to_string(),
                tail.to_string(),
                *padding,
                indexes.to_vec(),
            ));
        }
    }

    // Merge together collections that align on padding boundaries. For example,
    // 0998-0999 and 1000-1001 can be merged into 0998-1001. Note that only
    // indexes within the padding width limit are merged. If a collection is
    // entirely merged into another then it will not be included as a separate
    // collection in the results.
    let mut fully_merged = vec![];
    for collection in collections.iter_mut() {
        if collection.padding == 0 {
            continue;
        }

        for candidate in merge_candidates.iter() {
            if candidate.head == collection.head && candidate.tail == collection.tail {
                let mut merged_index_count = 0;
                for index in candidate.indexes.iter() {
                    if index.to_string().len() as i32 == collection.padding {
                        collection.indexes.push(*index);
                        merged_index_count += 1;
                    }
                }

                if merged_index_count == candidate.indexes.len() {
                    fully_merged.push(candidate);
                }
            }
        }
    }

    // filter out fully merged collections.
    let collections: Vec<Collection> = collections
        .into_iter()
        .filter(|x| !fully_merged.contains(&x))
        .collect();

    // Filter out collections that do not have at least as many indexes as
    // minimum_items. In addition, add any members of a filtered collection,
    // which are not members of an unfiltered collection, to the remainder.
    let minimum_items = 2;
    let mut filtered = vec![];
    let mut remainder_candidates = vec![];
    for collection in collections.into_iter() {
        if collection.indexes.len() > minimum_items {
            filtered.push(collection);
        } else {
            for member in collection.into_iter() {
                remainder_candidates.push(member);
            }
        }
    }

    for candidate in remainder_candidates.into_iter() {
        // Check if candidate has already been added to remainder to avoid
        // duplicate entries.
        if remainder.contains(&candidate) {
            continue;
        }

        let mut has_membership = false;

        for collection in filtered.iter() {
            if collection.contains(&candidate) {
                has_membership = true;
                break;
            }
        }

        if !has_membership {
            remainder.push(candidate.to_string());
        }
    }

    // // Set padding for all ambiguous collections according to the
    // // assume_padded_when_ambiguous setting.
    // let assume_padded_when_ambiguous = false;
    // if assume_padded_when_ambiguous {
    //     for collection in filtered.iter_mut() {
    //         if collection.padding == 0 && collection.indexes.len() != 0 {
    //             let start = collection.indexes[0].to_string();
    //             let end = collection.indexes.last().unwrap().to_string();
    //             let first_index_width = start.chars().count();
    //             let last_index_width = end.chars().count();
    //             if first_index_width == last_index_width {
    //                 collection.padding = first_index_width as i32;
    //             }
    //         }
    //     }
    // }

    return (filtered, remainder);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_assemble_single_sequence() {
        let files = vec![
            "shot/task/main_v001/render.1001.exr",
            "shot/task/main_v001/render.1002.exr",
            "shot/task/main_v001/render.1003.exr",
            "shot/task/main_v001/render.1004.exr",
            "shot/task/main_v001/render.1005.exr",
        ];
        let (collections, _remainder) = assemble(&files, None);
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].head, "shot/task/main_v001/render.");
        assert_eq!(collections[0].tail, ".exr");
        assert_eq!(collections[0].padding, 4);
        assert_eq!(collections[0].indexes, vec![1001, 1002, 1003, 1004, 1005]);
    }

    #[test]
    fn test_assemble_multiple_sequence() {
        let files = vec![
            "shot/task/main_v001/render.1001.exr",
            "shot/task/main_v001/render.1002.exr",
            "shot/task/main_v001/render.1003.exr",
            "shot/task/main_v001/render.1004.exr",
            "shot/task/main_v001/render.1005.exr",
            "shot/task/main_v002/render.1001.exr",
            "shot/task/main_v002/render.1002.exr",
            "shot/task/main_v002/render.1003.exr",
            "shot/task/main_v002/render.1004.exr",
            "shot/task/main_v002/render.1005.exr",
        ];
        let (collections, _remainder) = assemble(&files, None);

        assert_eq!(collections.len(), 2);
        let v1 = Collection::new(
            "shot/task/main_v001/render.".to_string(),
            ".exr".to_string(),
            4,
            vec![1001, 1002, 1003, 1004, 1005],
        );
        let v2 = Collection::new(
            "shot/task/main_v002/render.".to_string(),
            ".exr".to_string(),
            4,
            vec![1001, 1002, 1003, 1004, 1005],
        );
        assert_eq!(collections.contains(&v1), true);
        assert_eq!(collections.contains(&v2), true);
    }

    #[test]
    fn test_assemble_broken_sequence() {
        let files = vec![
            "shot/task/main_v001/render.1001.exr",
            "shot/task/main_v001/render.1002.exr",
            "shot/task/main_v001/render.1003.exr",
            "shot/task/main_v001/render.1005.exr",
            "shot/task/main_v002/render.1001.exr",
            "shot/task/main_v002/render.1002.exr",
            "shot/task/main_v002/render.1004.exr",
            "shot/task/main_v002/render.1005.exr",
        ];
        let (collections, _remainder) = assemble(&files, None);

        assert_eq!(collections.len(), 2);
        let v1 = Collection::new(
            "shot/task/main_v001/render.".to_string(),
            ".exr".to_string(),
            4,
            vec![1001, 1002, 1003, 1005],
        );
        let v2 = Collection::new(
            "shot/task/main_v002/render.".to_string(),
            ".exr".to_string(),
            4,
            vec![1001, 1002, 1004, 1005],
        );
        assert_eq!(collections.contains(&v1), true);
        assert_eq!(collections.contains(&v2), true);
    }

    #[test]
    fn test_large_set() {
        let mut indexes = vec![];
        let mut files = vec![];
        for i in 1..9999 {
            files.push(format!("shot/task/main_v001/render.{:04}.exr", i));
            files.push(format!("shot/task/main_v002/render.{:04}.exr", i));
            files.push(format!("shot/task/main_v003/render.{:04}.exr", i));
            files.push(format!("shot/task/main_v004/render.{:04}.exr", i));
            files.push(format!("shot/task/main_v005/render.{:04}.exr", i));
            indexes.push(i);
        }
        let (collections, _remainder) = assemble(&files, None);

        let v1 = Collection::new(
            "shot/task/main_v001/render.".to_string(),
            ".exr".to_string(),
            4,
            indexes.to_owned(),
        );
        let v2 = Collection::new(
            "shot/task/main_v002/render.".to_string(),
            ".exr".to_string(),
            4,
            indexes,
        );
        assert_eq!(collections.contains(&v1), true);
        assert_eq!(collections.contains(&v2), true);
    }

    #[test]
    fn test_assemble_remainder() {
        let files = vec![
            "shot/task/main_v001/render.1001.exr",
            "shot/task/main_v001/render.1002.exr",
            "shot/task/main_v001/render.1003.exr",
            "shot/task/main_v001/render.1004.exr",
            "shot/task/main_v001/render.1005.exr",
            "shot/task/main_v002/render.1005.exr",
            "foo",
        ];
        let (collections, remainder) = assemble(&files, None);
        assert_eq!(collections.len(), 1);
        assert_eq!(remainder.len(), 2);
        assert_eq!(
            remainder.contains(&String::from("shot/task/main_v002/render.1005.exr")),
            true
        );
        assert_eq!(remainder.contains(&String::from("foo")), true);
    }


    #[test]
    fn test_assemble_patterns() {
        let files = vec![
            "shot/task/main_v001/render.1001.exr",
            "shot/task/main_v002/render.1001.exr",
            "shot/task/main_v003/render.1001.exr",
            "shot/task/main_v001/render.1002.exr",
            "shot/task/main_v002/render.1002.exr",
            "shot/task/main_v003/render.1002.exr",
            "shot/task/main_v001/render.1003.exr",
            "shot/task/main_v002/render.1003.exr",
            "shot/task/main_v003/render.1003.exr",
        ];
        let (collections, _remainder) = assemble(&files, Some(vec![VERSION_PATTERN.to_string()]));
        assert_eq!(collections.len(), 3);

        let c1001 = Collection::new(
            "shot/task/main_v".to_string(),
            "/render.1001.exr".to_string(),
            3,
            vec![1,2,3],
        );
        let c1002 = Collection::new(
            "shot/task/main_v".to_string(),
            "/render.1002.exr".to_string(),
            3,
            vec![1,2,3],
        );
        let c1003 = Collection::new(
            "shot/task/main_v".to_string(),
            "/render.1003.exr".to_string(),
            3,
            vec![1,2,3],
        );

        assert_eq!(collections.contains(&c1001), true);
        assert_eq!(collections.contains(&c1002), true);
        assert_eq!(collections.contains(&c1003), true);
    }
}
