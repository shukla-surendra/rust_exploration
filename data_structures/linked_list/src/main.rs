use std::fmt::Display;

// Define the Node structure
#[derive(Debug)]
struct Node<T> {
    data: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    fn new(data: T) -> Self {
        Node {
            data,
            next: None,
        }
    }
}

// Define the LinkedList structure
#[derive(Debug)]
pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    size: usize,
}

impl<T> LinkedList<T> {
    // Create a new empty linked list
    pub fn new() -> Self {
        LinkedList {
            head: None,
            size: 0,
        }
    }

    // Add an element to the front of the list
    pub fn push_front(&mut self, data: T) {
        let new_node = Box::new(Node {
            data,
            next: self.head.take(),
        });
        self.head = Some(new_node);
        self.size += 1;
    }

    // Remove and return the first element
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            self.size -= 1;
            node.data
        })
    }

    // Add an element to the back of the list
    pub fn push_back(&mut self, data: T) {
        let new_node = Box::new(Node::new(data));

        match self.head {
            None => self.head = Some(new_node),
            Some(ref mut head) => {
                let mut current = head;
                while let Some(ref mut next) = current.next {
                    current = next;
                }
                current.next = Some(new_node);
            }
        }
        self.size += 1;
    }

    // Remove and return the last element
    pub fn pop_back(&mut self) -> Option<T> {
        match self.head {
            None => None,
            Some(ref mut head) if head.next.is_none() => {
                self.size -= 1;
                self.head.take().map(|node| node.data)
            }
            Some(ref mut head) => {
                let mut current = head;
                while current.next.as_ref().unwrap().next.is_some() {
                    current = current.next.as_mut().unwrap();
                }
                self.size -= 1;
                current.next.take().map(|node| node.data)
            }
        }
    }

    // Get the length of the list
    pub fn len(&self) -> usize {
        self.size
    }

    // Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    // Peek at the first element without removing it
    pub fn peek_front(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.data)
    }

    // Find an element in the list
    pub fn contains(&self, target: &T) -> bool
    where
        T: PartialEq
    {
        let mut current = &self.head;
        while let Some(node) = current {
            if node.data == *target {
                return true;
            }
            current = &node.next;
        }
        false
    }

    // Create an iterator over the list
    pub fn iter(&self) -> Iter<T> {
        Iter {
            current: self.head.as_deref(),
        }
    }
}

// Iterator implementation
pub struct Iter<'a, T> {
    current: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|node| {
            self.current = node.next.as_deref();
            &node.data
        })
    }
}

// Display implementation for pretty printing
impl<T: Display> Display for LinkedList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        let mut current = &self.head;
        let mut first = true;

        while let Some(node) = current {
            if !first {
                write!(f, " -> ")?;
            }
            write!(f, "{}", node.data)?;
            current = &node.next;
            first = false;
        }
        write!(f, "]")
    }
}

// Drop implementation for proper cleanup
impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_list() {
        let list: LinkedList<i32> = LinkedList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_push_front() {
        let mut list = LinkedList::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(list.len(), 3);
        assert_eq!(list.peek_front(), Some(&3));
    }

    #[test]
    fn test_pop_front() {
        let mut list = LinkedList::new();
        list.push_front(1);
        list.push_front(2);

        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn test_push_back() {
        let mut list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        assert_eq!(list.len(), 3);
        assert_eq!(list.peek_front(), Some(&1));
    }

    #[test]
    fn test_contains() {
        let mut list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        assert!(list.contains(&2));
        assert!(!list.contains(&4));
    }

    #[test]
    fn test_iterator() {
        let mut list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let collected: Vec<&i32> = list.iter().collect();
        assert_eq!(collected, vec![&1, &2, &3]);
    }
}

fn main() {
    // Example usage
    let mut list = LinkedList::new();

    // Add some elements
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);
    list.push_front(0);

    println!("List: {}", list);
    println!("Length: {}", list.len());

    // Iterate through the list
    print!("Elements: ");
    for item in list.iter() {
        print!("{} ", item);
    }
    println!();

    // Remove elements
    println!("Popped from front: {:?}", list.pop_front());
    println!("Popped from back: {:?}", list.pop_back());
    println!("List after popping: {}", list);

    // Check if element exists
    println!("Contains 2: {}", list.contains(&2));
    println!("Contains 5: {}", list.contains(&5));
}