use std::collections::HashMap;
use strfmt::strfmt;

#[derive(Debug, PartialEq)]
pub struct Collection {
    curr: usize,
    end: usize,
    pub head: String,
    pub tail: String,
    pub padding: i32,
    pub indexes: Vec<i32>,
}

impl Collection {
    pub fn new(head: String, tail: String, padding: i32, indexes: Vec<i32>) -> Collection {
        Collection {
            curr: 0,
            end: indexes.len(),
            head: head,
            tail: tail,
            padding: padding,
            indexes: indexes,
        }
    }

    // Return formatted string represented collection.
    pub fn format(self: &Self, fmt: Option<&str>) -> String {
        let mut vars = HashMap::new();
        let padding = self.padding.to_string();
        let start = self.indexes[0].to_string();
        let end = self.indexes.last().unwrap().to_string();

        vars.insert("head".to_string(), self.head.as_str());
        vars.insert("padding".to_string(), padding.as_str());
        vars.insert("tail".to_string(), self.tail.as_str());
        vars.insert("start".to_string(), start.as_str());
        vars.insert("end".to_string(), end.as_str());

        let fmt = fmt.unwrap_or("{head}%0{padding}d{tail} [{start}-{end}]");
        match strfmt(&fmt, &vars) {
            Ok(string) => string,
            Err(_) => "".to_string(),
        }
    }

    pub fn contains(self: &Self, item: String) -> bool {
        for i in self.into_iter() {
            if i == item {
                return true;
            }
        }
        false
    }

    // Return whether entire collection is contiguous.
    pub fn is_contiguous(self) -> bool {
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
    pub fn holes(self) -> Collection {
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

        return Collection::new(self.head, self.tail, self.padding, missing);
    }
}

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

// ---------------------------------
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

    // // just return the str reference
    // fn next(&mut self) -> Option<Self::Item> {
    //         self.iter.next()
    // }
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
            c.format(Some("{head}%0{padding}d{tail} [{start}-{end}]")),
            "head.%04d.tail [1001-1005]"
        );
        assert_eq!(c.format(Some("{head}%0{padding}d{tail}")), "head.%04d.tail");
        assert_eq!(c.format(Some("{head}%0{padding}d{tail} {FOO}")), "");
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
    }
}
