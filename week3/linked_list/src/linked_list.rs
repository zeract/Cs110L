use std::fmt;
use std::option::Option;

pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    size: usize,
}

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(value: T, next: Option<Box<Node<T>>>) -> Node<T> {
        Node {value: value, next: next}
    }
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        LinkedList {head: None, size: 0}
    }
    
    pub fn get_size(&self) -> usize {
        self.size
    }
    
    pub fn is_empty(&self) -> bool {
        self.get_size() == 0
    }
    
    pub fn push_front(&mut self, value: T) {
        let new_node: Box<Node<T>> = Box::new(Node::new(value, self.head.take()));
        self.head = Some(new_node);
        self.size += 1;
    }
    
    pub fn pop_front(&mut self) -> Option<T> {
        let node: Box<Node<T>> = self.head.take()?;
        self.head = node.next;
        self.size -= 1;
        Some(node.value)
    }
}


impl <T:fmt::Display> fmt::Display for LinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current: &Option<Box<Node<T>>> = &self.head;
        let mut result = String::new();
        loop {
            match current {
                Some(node) => {
                    result = format!("{} {}", result, node.value);
                    current = &node.next;
                },
                None => break,
            }
        }
        write!(f, "{}", result)
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(mut node) = current {
            current = node.next.take();
        }
    }
}

impl<T:Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        let mut result = LinkedList::new();
        let length = self.size;
        let mut vec :Vec<T>= Vec::new();
        let mut head = &self.head;
        for i in 1..=length{
            let value = &head.as_ref().unwrap().value;
            vec.push(value.clone());
            head = &head.as_ref().unwrap().next;
        }
        for i in 0..length{
            result.push_front(vec[length-i-1].clone());
        }
        result
    }

}

impl<T:PartialEq> PartialEq for LinkedList<T>{
    fn eq(&self, other: &Self) -> bool {
        let len1 = self.get_size();
        let len2 = other.get_size();
        if len1 != len2{
            return false;
        }
        let mut first_node = &self.head;
        let mut second_node = &other.head;
        while first_node.is_some() && second_node.is_some() {
            let value1 = &first_node.as_ref().unwrap().value;
            let value2 = &second_node.as_ref().unwrap().value;
            if value1 != value2{
                return false;
            }
            first_node = &first_node.as_ref().unwrap().next;
            second_node = &second_node.as_ref().unwrap().next;
        }
        true
    }
}


