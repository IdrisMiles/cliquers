use regex::{Captures, Regex};
use std::collections::HashMap;
use strfmt::strfmt;

#[derive(Debug, PartialEq)]
pub struct Collection {
    pub head: String,
    pub tail: String,
    pub padding: i32,
    pub indexes: Vec<i32>,
}

impl Collection {
    pub fn new(head: String, tail: String, padding: i32, indexes: Vec<i32>) -> Collection {
        Collection {
            head: head,
            tail: tail,
            padding: padding,
            indexes: indexes,
        }
    }

    // Return formatted string represented collection.
    pub fn format(self: &Self, fmt: Option<&str>) -> String {
        let padding = format!("%0{}d", self.padding);
        let start = self.indexes[0].to_string();
        let end = self.indexes.last().unwrap().to_string();
        let range = format!("{start}-{end}", start = start.as_str(), end = end.as_str());
        let mut ranges = String::new();
        let separated = self.separate();
        if separated.len() > 1 {
            ranges.clone_from(
                &separated
                    .iter()
                    .map(|x| x.format(Some("{range}")))
                    .collect::<Vec<String>>()
                    .join(", "),
            );
        } else {
            ranges.clone_from(&range);
        }

        match fmt {
            Some(fmt) => {
                let mut vars = HashMap::new();
                let mut holes = String::new();
                if fmt.contains("{holes}") {
                    holes.clone_from(&self.holes().format(Some("{range}")));
                    vars.insert("holes".to_string(), holes.as_str());
                }

                if fmt.contains("{range}") || fmt.contains("{ranges}") {
                    let indexes_count = self.indexes.len();
                    if indexes_count == 0 {
                        vars.insert("range".to_string(), "");
                    } else if indexes_count == 1 {
                        vars.insert("range".to_string(), start.as_str());
                    } else {
                        vars.insert("range".to_string(), range.as_str());
                    }
                }
                if fmt.contains("{ranges}") {
                    vars.insert("ranges".to_string(), ranges.as_str());
                }

                vars.insert("head".to_string(), self.head.as_str());
                vars.insert("padding".to_string(), padding.as_str());
                vars.insert("tail".to_string(), self.tail.as_str());
                vars.insert("start".to_string(), start.as_str());
                vars.insert("end".to_string(), end.as_str());

                match strfmt(&fmt, &vars) {
                    Ok(string) => string,
                    Err(_) => "".to_string(),
                }
            }
            None => format!(
                "{head}{padding}{tail} [{ranges}]",
                head = self.head.as_str(),
                padding = padding.as_str(),
                tail = self.tail.as_str(),
                ranges = ranges.as_str(),
            ),
        }
    }

    pub fn match_item<'t>(self: &Self, item: &'t String) -> Option<Captures<'t>> {
        let regex_str = format!(
            "^{0}(?P<index>(?P<padding>0*)\\d+?){1}$",
            self.head, self.tail
        );
        let compiled_regex: Regex = Regex::new(regex_str.as_str()).unwrap();
        let regex_match = compiled_regex.captures(item);
        match regex_match {
            None => return None,
            Some(capture) => {
                let index = capture.name("index").unwrap().as_str();
                let mut padded = false;
                match capture.name("padding") {
                    Some(_) => padded = true,
                    None => (),
                };

                if self.padding == 0 && padded {
                    return None;
                }
                if index.chars().count() != self.padding as usize {
                    return None;
                }
                Some(capture)
            }
        }
    }

    // Return whether an item exists within the collection
    pub fn contains(self: &Self, item: &String) -> bool {
        for i in self.into_iter() {
            if i == *item {
                return true;
            }
        }
        false
    }

    // Return whether entire collection is contiguous.
    pub fn is_contiguous(self: &Self) -> bool {
        let mut previous = None;
        for index in self.indexes.iter() {
            match previous {
                None => {
                    previous = Some(index);
                    continue;
                }
                Some(v) => {
                    if *index != (v + 1) {
                        return false;
                    }
                }
            }
            previous = Some(index);
        }

        return true;
    }

    // Return holes in collection.
    pub fn holes(self: &Self) -> Collection {
        let mut missing = vec![];
        let mut previous = None;
        for index in self.indexes.iter() {
            match previous {
                None => {
                    previous = Some(index);
                    continue;
                }
                Some(v) => {
                    if *index != (v + 1) {
                        missing.extend(v + 1..*index);
                    }
                }
            }
            previous = Some(index);
        }

        return Collection::new(
            self.head.to_owned(),
            self.tail.to_owned(),
            self.padding,
            missing,
        );
    }

    // Return contiguous parts of collection as separate collections.
    fn separate(self: &Self) -> Vec<Self> {
        let mut collections = vec![];
        let mut start = None;
        let mut end = None;

        for index in self.indexes.iter() {
            if start == None {
                start = Some(*index);
                end = start;
                continue;
            }

            if *index != (end.unwrap() + 1) {
                collections.push(Collection::new(
                    self.head.to_string(),
                    self.tail.to_string(),
                    self.padding,
                    (start.unwrap()..end.unwrap() + 1).collect(),
                ));

                start = Some(*index);
            }

            end = Some(*index);
        }

        if start == None {
            collections.push(Collection::new(
                self.head.to_string(),
                self.tail.to_string(),
                self.padding,
                vec![],
            ))
        } else {
            collections.push(Collection::new(
                self.head.to_string(),
                self.tail.to_string(),
                self.padding,
                (start.unwrap()..end.unwrap() + 1).collect(),
            ))
        }

        return collections;
    }
}

// ------------------------------------------------------------------------------
// Consuming iterator
// implementing into_iter
impl IntoIterator for Collection {
    type Item = String;
    type IntoIter = IntoIteratorHelper;

    fn into_iter(self) -> Self::IntoIter {
        IntoIteratorHelper {
            iter: self.indexes.into_iter(),
            head: self.head,
            tail: self.tail,
            padding: self.padding,
        }
    }
}

pub struct IntoIteratorHelper {
    iter: std::vec::IntoIter<i32>,
    head: String,
    tail: String,
    padding: i32,
}

impl Iterator for IntoIteratorHelper {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(index) => {
                let mut vars = HashMap::new();
                let padding: usize = self.padding as usize;
                let index = format!("{:0>padding$}", index.to_string(), padding = padding);
                println!("{}", index);

                vars.insert("head".to_string(), self.head.as_str());
                vars.insert("tail".to_string(), self.tail.as_str());
                vars.insert("index".to_string(), index.as_str());

                let fmt = "{head}{index}{tail}";
                match strfmt(&fmt, &vars) {
                    Ok(string) => Some(string),
                    Err(_) => None,
                }
            }
            None => None,
        }
    }
}

// ------------------------------------------------------------------------------
// Non consuming iterator
pub struct IterHelper<'a> {
    iter: ::std::slice::Iter<'a, i32>,
    head: String,
    tail: String,
    padding: i32,
}

// implement the IntoIterator trait for a non-consuming iterator. Iteration will
// borrow the Words structure
impl<'a> IntoIterator for &'a Collection {
    type Item = String;
    type IntoIter = IterHelper<'a>;

    // note that into_iter() is consuming self
    fn into_iter(self) -> Self::IntoIter {
        IterHelper {
            iter: self.indexes.iter(),
            head: self.head.to_string(),
            tail: self.tail.to_string(),
            padding: self.padding,
        }
    }
}

// now, implements Iterator trait for the helper struct, to be used by adapters
impl<'a> Iterator for IterHelper<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(index) => {
                let mut vars = HashMap::new();
                let padding: usize = self.padding as usize;
                let index = format!("{:0padding$}", index.to_string(), padding = padding);

                vars.insert("head".to_string(), self.head.as_str());
                vars.insert("tail".to_string(), self.tail.as_str());
                vars.insert("index".to_string(), index.as_str());

                let fmt = "{head}{index}{tail}";
                match strfmt(&fmt, &vars) {
                    Ok(string) => Some(string),
                    Err(_) => None,
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let c = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1003, 1004, 1005],
        );

        assert_eq!(c.format(None), "head.%04d.tail [1001-1005]");
        assert_eq!(
            c.format(Some("{head}{padding}{tail} [{range}]")),
            "head.%04d.tail [1001-1005]"
        );
        assert_eq!(
            c.format(Some("{head}{padding}{tail} [{ranges}]")),
            "head.%04d.tail [1001-1005]"
        );
        assert_eq!(c.format(Some("{head}{padding}{tail}")), "head.%04d.tail");
        assert_eq!(c.format(Some("{head}{padding}{tail} {FOO}")), "");

        let c = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1003, 1005],
        );
        assert_eq!(c.format(Some("{holes}")), "1004");
        assert_eq!(c.format(None), "head.%04d.tail [1001-1003, 1005]");
        assert_eq!(
            c.format(Some("{head}{padding}{tail} [{range}]")),
            "head.%04d.tail [1001-1005]"
        );
        assert_eq!(
            c.format(Some("{head}{padding}{tail} [{ranges}]")),
            "head.%04d.tail [1001-1003, 1005]"
        );
    }

    #[test]
    fn test_match() {
        let c = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1003, 1004, 1005],
        );

        // probably dodgy way of testing...
        match c.match_item(&"head.1010.tail".to_string()) {
            Some(m) => assert_eq!(m.name("index").unwrap().as_str(), "1010"),
            None => assert_eq!(false, true),
        }
        match c.match_item(&"head.10100.tail".to_string()) {
            Some(_) => assert_eq!(false, true),
            None => assert_eq!(true, true),
        }
        match c.match_item(&"foo.1010.tail".to_string()) {
            Some(_) => assert_eq!(false, true),
            None => assert_eq!(true, true),
        }
    }

    #[test]
    fn test_is_contiguous() {
        let c = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1003, 1004, 1005],
        );
        assert_eq!(c.is_contiguous(), true);

        let c = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1003, 1005],
        );
        assert_eq!(c.is_contiguous(), false);
    }

    #[test]
    fn test_holes() {
        let c = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1003, 1004, 1005],
        );
        let expected = Collection::new("head.".to_string(), ".tail".to_string(), 4, vec![]);
        assert_eq!(c.holes(), expected);

        let c = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1003, 1005],
        );
        let expected = Collection::new("head.".to_string(), ".tail".to_string(), 4, vec![1004]);
        assert_eq!(c.holes(), expected);

        let c = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1003, 1004, 1005, 1008, 1010],
        );
        let expected = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1006, 1007, 1009],
        );
        assert_eq!(c.holes(), expected);
    }

    #[test]
    fn test_seperate() {
        let c1 = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1003, 1004, 1005],
        );
        assert_eq!(c1.separate()[0], c1);

        let c1 = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1003, 1005],
        );
        let expected = vec![
            Collection::new(
                "head.".to_string(),
                ".tail".to_string(),
                4,
                vec![1001, 1002, 1003],
            ),
            Collection::new("head.".to_string(), ".tail".to_string(), 4, vec![1005]),
        ];
        assert_eq!(c1.separate(), expected);
    }

    #[test]
    fn test_equals() {
        let c1 = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1003, 1004, 1005],
        );
        let c2 = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1003, 1004, 1005],
        );
        assert_eq!(c1, c2);

        let d1 = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1003, 1004],
        );
        assert_ne!(c1, d1);
    }

    #[test]
    fn test_iterator() {
        let c1 = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1003, 1004, 1005],
        );

        let mut iter = c1.into_iter();
        assert_eq!(iter.next(), Some("head.1001.tail".to_string()));
        assert_eq!(iter.next(), Some("head.1002.tail".to_string()));
        assert_eq!(iter.next(), Some("head.1003.tail".to_string()));
        assert_eq!(iter.next(), Some("head.1004.tail".to_string()));
        assert_eq!(iter.next(), Some("head.1005.tail".to_string()));
        assert_eq!(iter.next(), None);

        let c1 = Collection::new(
            "head.".to_string(),
            ".tail".to_string(),
            4,
            vec![1001, 1002, 1004, 1005],
        );
        let mut iter = c1.into_iter();
        assert_eq!(iter.next(), Some("head.1001.tail".to_string()));
        assert_eq!(iter.next(), Some("head.1002.tail".to_string()));
        assert_eq!(iter.next(), Some("head.1004.tail".to_string()));
        assert_eq!(iter.next(), Some("head.1005.tail".to_string()));
        assert_eq!(iter.next(), None);

        let c1 = Collection::new("head.".to_string(), ".tail".to_string(), 5, vec![23]);
        let mut iter = c1.into_iter();
        assert_eq!(iter.next(), Some("head.00023.tail".to_string()));
        assert_eq!(iter.next(), None);
    }
}
